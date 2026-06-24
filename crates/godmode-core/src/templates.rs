//! Task template loading, variable substitution, and graph application.
//!
//! Templates live in `<repo-root>/templates/<name>.yaml` (local) or
//! `$HOME/.config/godmode/templates/<name>.yaml` (global fallback).
//!
//! Template YAML format:
//! ```yaml
//! meta:
//!   name: tdd-cycle
//!   description: "Red-green-refactor loop for one crate"
//!   vars:
//!     - name: crate
//!       required: true
//!     - name: prefix
//!       default: "t"
//!
//! tasks:
//!   - id: "{{prefix}}-red"
//!     title: "Write failing test for {{crate}}"
//!     crate_name: "{{crate}}"
//!     run: "cargo nextest run -p {{crate}}"
//! ```

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use miette::{Diagnostic, NamedSource, SourceSpan};
use serde::Deserialize;
use thiserror::Error;

use crate::graph;
use crate::model::{Task, TaskGraph};

// ── raw deserialization types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawVarDef {
    name: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    default: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawMeta {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    vars: Vec<RawVarDef>,
}

#[derive(Debug, Deserialize)]
struct RawTemplate {
    meta: RawMeta,
    #[serde(default)]
    tasks: Vec<Task>,
}

// ── public types ───────────────────────────────────────────────────────────

/// Resolved template metadata.
#[derive(Debug, Clone)]
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
}

/// A fully resolved template — all vars substituted, ready to apply.
#[derive(Debug)]
pub struct Template {
    pub meta: TemplateMeta,
    pub tasks: Vec<Task>,
}

/// Source of a discovered template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    Local,
    Global,
}

impl std::fmt::Display for TemplateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateSource::Local => write!(f, "local"),
            TemplateSource::Global => write!(f, "global"),
        }
    }
}

/// A discovered template entry (not yet loaded).
pub struct TemplateEntry {
    pub meta: TemplateMeta,
    pub path: PathBuf,
    pub source: TemplateSource,
}

impl TemplateEntry {
    fn new(meta: TemplateMeta, path: PathBuf, source: TemplateSource) -> Self {
        Self { meta, path, source }
    }
}

/// Errors emitted while resolving and loading task templates.
#[derive(Debug, Diagnostic, Error)]
pub enum TemplateError {
    #[error("template '{name}' not found")]
    #[diagnostic(
        code(godmode::template::not_found),
        help(
            "Create templates/{name}.yaml, templates/{name}.template.yaml, or the matching file under $HOME/.config/godmode/templates/."
        )
    )]
    NotFound { name: String },

    #[error("failed to read template {}", path.display())]
    #[diagnostic(code(godmode::template::read_failed))]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to read template directory {}", path.display())]
    #[diagnostic(code(godmode::template::read_dir_failed))]
    ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse template metadata from {}", path.display())]
    #[diagnostic(
        code(godmode::template::parse_metadata_failed),
        help("Expected a template with a top-level meta block and optional tasks list.")
    )]
    ParseMetadata {
        path: PathBuf,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("YAML parser stopped here")]
        span: Option<SourceSpan>,
        #[source]
        source: Box<serde_yaml::Error>,
    },

    #[error("failed to parse substituted template from {}", path.display())]
    #[diagnostic(
        code(godmode::template::parse_substituted_failed),
        help("Check substituted variable values for YAML-sensitive characters.")
    )]
    ParseSubstituted {
        path: PathBuf,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("YAML parser stopped here after variable substitution")]
        span: Option<SourceSpan>,
        #[source]
        source: Box<serde_yaml::Error>,
    },

    #[error("template '{template}' requires var '{var}'")]
    #[diagnostic(
        code(godmode::template::missing_var),
        help("Pass --var {var}=<value> when applying this template.")
    )]
    MissingVar { template: String, var: String },

    #[error("invalid --var '{value}': expected key=value")]
    #[diagnostic(
        code(godmode::template::invalid_var),
        help("Use --var name=value. Omit spaces around '='.")
    )]
    InvalidVar { value: String },
}

type TemplateResult<T> = std::result::Result<T, TemplateError>;

// ── resolution ─────────────────────────────────────────────────────────────

/// Local templates directory relative to repo root.
fn local_dir(root: &Path) -> PathBuf {
    root.join("templates")
}

/// Global templates directory: `$HOME/.config/godmode/templates/`.
fn global_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("godmode")
            .join("templates")
    })
}

/// Locate a template file by name. Checks local dir first, then global.
/// Returns the path to the `.yaml` or `.template.yaml` file.
pub fn find(root: &Path, name: &str) -> TemplateResult<PathBuf> {
    for filename in candidate_filenames(name) {
        let local = local_dir(root).join(&filename);
        if local.exists() {
            return Ok(local);
        }
    }

    if let Some(global) = global_dir() {
        for filename in candidate_filenames(name) {
            let g = global.join(&filename);
            if g.exists() {
                return Ok(g);
            }
        }
    }

    Err(TemplateError::NotFound {
        name: name.to_string(),
    })
}

/// List all templates in local and global dirs. Local entries take precedence
/// (duplicate names from global are omitted).
pub fn list(root: &Path) -> TemplateResult<Vec<TemplateEntry>> {
    let mut entries: Vec<TemplateEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Local first.
    let local = local_dir(root);
    if local.is_dir() {
        for entry in read_dir(&local)?.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(tmpl) = load_meta(&p)
            {
                seen.insert(tmpl.name.clone());
                entries.push(TemplateEntry::new(tmpl, p, TemplateSource::Local));
            }
        }
    }

    // Global fallback — skip names already found locally.
    if let Some(global) = global_dir()
        && global.is_dir()
    {
        for entry in read_dir(&global)?.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(tmpl) = load_meta(&p)
                && !seen.contains(&tmpl.name)
            {
                entries.push(TemplateEntry::new(tmpl, p, TemplateSource::Global));
            }
        }
    }

    Ok(entries)
}

// ── loading ────────────────────────────────────────────────────────────────

/// Load only metadata from a template file (no substitution).
fn load_meta(path: &Path) -> TemplateResult<TemplateMeta> {
    let raw = read_template(path)?;
    let t: RawTemplate = serde_yaml::from_str(&raw)
        .map_err(|source| parse_error(path, &raw, source, ParsePhase::Metadata))?;
    Ok(TemplateMeta {
        name: t.meta.name,
        description: t.meta.description,
    })
}

/// Load a template file, apply variable substitution, and return a resolved `Template`.
///
/// `vars` is a slice of `"key=value"` strings (same format as `--var` CLI flag).
pub fn load(path: &Path, vars: &[String]) -> TemplateResult<Template> {
    let raw = read_template(path)?;

    // Parse var definitions first (pre-substitution) to validate required vars.
    let raw_tmpl: RawTemplate = serde_yaml::from_str(&raw)
        .map_err(|source| parse_error(path, &raw, source, ParsePhase::Metadata))?;

    // Build substitution map from supplied vars.
    let supplied = parse_vars(vars)?;

    // Apply defaults for missing vars, error on missing required vars.
    let mut sub_map: HashMap<String, String> = HashMap::new();
    for var_def in &raw_tmpl.meta.vars {
        if let Some(val) = supplied.get(&var_def.name) {
            sub_map.insert(var_def.name.clone(), val.clone());
        } else if let Some(default) = &var_def.default {
            sub_map.insert(var_def.name.clone(), default.clone());
        } else if var_def.required {
            return Err(TemplateError::MissingVar {
                template: raw_tmpl.meta.name.clone(),
                var: var_def.name.clone(),
            });
        }
    }

    // Substitute vars into the raw YAML string, then re-parse.
    let substituted = substitute(&raw, &sub_map);
    let resolved: RawTemplate = serde_yaml::from_str(&substituted)
        .map_err(|source| parse_error(path, &substituted, source, ParsePhase::Substituted))?;

    Ok(Template {
        meta: TemplateMeta {
            name: resolved.meta.name,
            description: resolved.meta.description,
        },
        tasks: resolved.tasks,
    })
}

// ── application ────────────────────────────────────────────────────────────

/// Apply a resolved template into a task graph.
///
/// Idempotent — tasks whose IDs already exist are skipped. Returns `(applied, skipped)`.
pub fn apply(graph: &mut TaskGraph, template: Template) -> Result<(usize, usize)> {
    let mut applied = 0usize;
    let mut skipped = 0usize;
    for task in template.tasks {
        match graph::add(graph, task) {
            Ok(()) => applied += 1,
            Err(e) if e.to_string().contains("already exists") => skipped += 1,
            Err(e) => return Err(e),
        }
    }
    Ok((applied, skipped))
}

// ── internal helpers ───────────────────────────────────────────────────────

fn candidate_filenames(name: &str) -> [String; 2] {
    [format!("{name}.yaml"), format!("{name}.template.yaml")]
}

/// Parse a slice of `"key=value"` strings into a map.
fn parse_vars(vars: &[String]) -> TemplateResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for v in vars {
        let (k, val) = v
            .split_once('=')
            .ok_or_else(|| TemplateError::InvalidVar { value: v.clone() })?;
        map.insert(k.to_string(), val.to_string());
    }
    Ok(map)
}

fn read_dir(path: &Path) -> TemplateResult<std::fs::ReadDir> {
    std::fs::read_dir(path).map_err(|source| TemplateError::ReadDir {
        path: path.to_path_buf(),
        source,
    })
}

fn read_template(path: &Path) -> TemplateResult<String> {
    std::fs::read_to_string(path).map_err(|source| TemplateError::Read {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Clone, Copy)]
enum ParsePhase {
    Metadata,
    Substituted,
}

fn parse_error(
    path: &Path,
    raw: &str,
    source: serde_yaml::Error,
    phase: ParsePhase,
) -> TemplateError {
    let span = source
        .location()
        .and_then(|location| source_span_for_location(raw, location.line(), location.column()));
    let src = Arc::new(NamedSource::new(
        path.display().to_string(),
        raw.to_string(),
    ));
    match phase {
        ParsePhase::Metadata => TemplateError::ParseMetadata {
            path: path.to_path_buf(),
            src,
            span,
            source: Box::new(source),
        },
        ParsePhase::Substituted => TemplateError::ParseSubstituted {
            path: path.to_path_buf(),
            src,
            span,
            source: Box::new(source),
        },
    }
}

fn source_span_for_location(raw: &str, line: usize, column: usize) -> Option<SourceSpan> {
    let line = line.checked_sub(1)?;
    let column = column.checked_sub(1)?;
    let mut offset = 0usize;
    for (idx, text) in raw.split_inclusive('\n').enumerate() {
        if idx == line {
            let column_offset = text
                .char_indices()
                .nth(column)
                .map(|(byte_idx, _)| byte_idx)
                .unwrap_or_else(|| text.trim_end_matches('\n').len());
            return Some((offset + column_offset, 1).into());
        }
        offset += text.len();
    }
    None
}

/// Replace all `{{key}}` occurrences in `raw` with values from `vars`.
fn substitute(raw: &str, vars: &HashMap<String, String>) -> String {
    let mut result = raw.to_string();
    for (k, v) in vars {
        let placeholder = format!("{{{{{}}}}}", k);
        result = result.replace(&placeholder, v);
    }
    result
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_template(dir: &Path, name: &str, content: &str) -> PathBuf {
        let p = dir.join(format!("{}.yaml", name));
        std::fs::write(&p, content).unwrap();
        p
    }

    const BASIC_TEMPLATE: &str = r#"
meta:
  name: tdd-cycle
  description: "TDD loop"
  vars:
    - name: crate
      required: true
    - name: prefix
      default: "t"

tasks:
  - id: "{{prefix}}-red"
    title: "Write failing test for {{crate}}"
    status: pending
    depends_on: []
    notes: ""
    crate_name: "{{crate}}"
  - id: "{{prefix}}-green"
    title: "Implement minimum code"
    status: pending
    depends_on: ["{{prefix}}-red"]
    notes: ""
"#;

    #[test]
    fn substitute_replaces_vars() {
        let mut vars = HashMap::new();
        vars.insert("crate".to_string(), "foo".to_string());
        vars.insert("prefix".to_string(), "my".to_string());
        let result = substitute("id: {{prefix}}-{{crate}}", &vars);
        assert!(result.contains("my-foo"));
    }

    #[test]
    fn substitute_unknown_var_is_noop() {
        let vars = HashMap::new();
        let result = substitute("id: {{unknown}}", &vars);
        assert_eq!(result, "id: {{unknown}}");
    }

    #[test]
    fn load_happy_path() {
        let dir = TempDir::new().unwrap();
        let path = write_template(dir.path(), "tdd-cycle", BASIC_TEMPLATE);
        let tmpl = load(&path, &["crate=godmode-core".to_string()]).unwrap();
        assert_eq!(tmpl.tasks.len(), 2);
        assert_eq!(tmpl.tasks[0].id, "t-red");
        assert_eq!(tmpl.tasks[0].crate_name.as_deref(), Some("godmode-core"));
        assert_eq!(tmpl.tasks[1].depends_on, vec!["t-red"]);
    }

    #[test]
    fn load_missing_required_var_errors() {
        let dir = TempDir::new().unwrap();
        let path = write_template(dir.path(), "tdd-cycle", BASIC_TEMPLATE);
        let err = load(&path, &[]).unwrap_err();
        assert!(err.to_string().contains("requires var 'crate'"));
        assert!(matches!(
            err,
            TemplateError::MissingVar {
                ref template,
                ref var
            } if template == "tdd-cycle" && var == "crate"
        ));
    }

    #[test]
    fn load_uses_default_for_optional_var() {
        let dir = TempDir::new().unwrap();
        let path = write_template(dir.path(), "tdd-cycle", BASIC_TEMPLATE);
        let tmpl = load(&path, &["crate=foo".to_string()]).unwrap();
        // prefix defaults to "t"
        assert_eq!(tmpl.tasks[0].id, "t-red");
    }

    #[test]
    fn apply_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = write_template(dir.path(), "tdd-cycle", BASIC_TEMPLATE);

        let mut g = TaskGraph::default();

        let tmpl1 = load(&path, &["crate=foo".to_string()]).unwrap();
        let (applied, skipped) = apply(&mut g, tmpl1).unwrap();
        assert_eq!(applied, 2);
        assert_eq!(skipped, 0);

        let tmpl2 = load(&path, &["crate=foo".to_string()]).unwrap();
        let (applied2, skipped2) = apply(&mut g, tmpl2).unwrap();
        assert_eq!(applied2, 0);
        assert_eq!(skipped2, 2);
    }

    #[test]
    fn find_prefers_local_over_global() {
        let root = TempDir::new().unwrap();
        std::fs::create_dir_all(root.path().join("templates")).unwrap();
        write_template(&root.path().join("templates"), "my-tmpl", BASIC_TEMPLATE);
        let found = find(root.path(), "my-tmpl").unwrap();
        assert!(found.starts_with(root.path()));
    }

    #[test]
    fn find_resolves_template_yaml_suffix() {
        let root = TempDir::new().unwrap();
        std::fs::create_dir_all(root.path().join("templates")).unwrap();
        let expected = write_template(
            &root.path().join("templates"),
            "AICHAT-SYSTEM.template",
            BASIC_TEMPLATE,
        );
        let found = find(root.path(), "AICHAT-SYSTEM").unwrap();
        assert_eq!(found, expected);
    }

    #[test]
    fn find_missing_template_errors() {
        let root = TempDir::new().unwrap();
        let err = find(root.path(), "nonexistent").unwrap_err();
        assert!(err.to_string().contains("not found"));
        assert!(matches!(
            err,
            TemplateError::NotFound { ref name } if name == "nonexistent"
        ));
    }

    #[test]
    fn parse_vars_valid() {
        let vars = vec!["crate=foo".to_string(), "prefix=my".to_string()];
        let map = parse_vars(&vars).unwrap();
        assert_eq!(map["crate"], "foo");
        assert_eq!(map["prefix"], "my");
    }

    #[test]
    fn parse_vars_invalid_format_errors() {
        let vars = vec!["badformat".to_string()];
        let err = parse_vars(&vars).unwrap_err();
        assert!(err.to_string().contains("key=value"));
        assert!(matches!(
            err,
            TemplateError::InvalidVar { ref value } if value == "badformat"
        ));
    }

    #[test]
    fn load_parse_error_includes_source_span() {
        let dir = TempDir::new().unwrap();
        let path = write_template(
            dir.path(),
            "broken",
            r#"
meta:
  name: broken
tasks:
  - id: [
"#,
        );
        let err = load(&path, &[]).unwrap_err();
        assert!(matches!(
            err,
            TemplateError::ParseMetadata { span: Some(_), .. }
        ));
    }
}
