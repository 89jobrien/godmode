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
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

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
/// Returns the path to the `.yaml` file.
pub fn find(root: &Path, name: &str) -> Result<PathBuf> {
    let filename = format!("{}.yaml", name);

    let local = local_dir(root).join(&filename);
    if local.exists() {
        return Ok(local);
    }

    if let Some(global) = global_dir() {
        let g = global.join(&filename);
        if g.exists() {
            return Ok(g);
        }
    }

    bail!(
        "template '{}' not found in templates/ or ~/.config/godmode/templates/",
        name
    )
}

/// List all templates in local and global dirs. Local entries take precedence
/// (duplicate names from global are omitted).
pub fn list(root: &Path) -> Result<Vec<TemplateEntry>> {
    let mut entries: Vec<TemplateEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Local first.
    let local = local_dir(root);
    if local.is_dir() {
        for entry in std::fs::read_dir(&local)
            .with_context(|| format!("reading {}", local.display()))?
            .flatten()
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(tmpl) = load_meta(&p)
            {
                seen.insert(tmpl.name.clone());
                entries.push(TemplateEntry {
                    meta: tmpl,
                    path: p,
                    source: TemplateSource::Local,
                });
            }
        }
    }

    // Global fallback — skip names already found locally.
    if let Some(global) = global_dir()
        && global.is_dir()
    {
        for entry in std::fs::read_dir(&global)
            .with_context(|| format!("reading {}", global.display()))?
            .flatten()
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(tmpl) = load_meta(&p)
                && !seen.contains(&tmpl.name)
            {
                entries.push(TemplateEntry {
                    meta: tmpl,
                    path: p,
                    source: TemplateSource::Global,
                });
            }
        }
    }

    Ok(entries)
}

// ── loading ────────────────────────────────────────────────────────────────

/// Load only metadata from a template file (no substitution).
fn load_meta(path: &Path) -> Result<TemplateMeta> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let t: RawTemplate =
        serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
    Ok(TemplateMeta {
        name: t.meta.name,
        description: t.meta.description,
    })
}

/// Load a template file, apply variable substitution, and return a resolved `Template`.
///
/// `vars` is a slice of `"key=value"` strings (same format as `--var` CLI flag).
pub fn load(path: &Path, vars: &[String]) -> Result<Template> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    // Parse var definitions first (pre-substitution) to validate required vars.
    let raw_tmpl: RawTemplate = serde_yaml::from_str(&raw)
        .with_context(|| format!("parsing template metadata from {}", path.display()))?;

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
            bail!(
                "template '{}' requires var '{}' — pass --var {}=<value>",
                raw_tmpl.meta.name,
                var_def.name,
                var_def.name
            );
        }
    }

    // Substitute vars into the raw YAML string, then re-parse.
    let substituted = substitute(&raw, &sub_map);
    let resolved: RawTemplate = serde_yaml::from_str(&substituted)
        .with_context(|| format!("parsing substituted template from {}", path.display()))?;

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

/// Parse a slice of `"key=value"` strings into a map.
fn parse_vars(vars: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for v in vars {
        let (k, val) = v
            .split_once('=')
            .with_context(|| format!("invalid --var '{}': expected key=value", v))?;
        map.insert(k.to_string(), val.to_string());
    }
    Ok(map)
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
    fn find_missing_template_errors() {
        let root = TempDir::new().unwrap();
        let err = find(root.path(), "nonexistent").unwrap_err();
        assert!(err.to_string().contains("not found"));
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
    }
}
