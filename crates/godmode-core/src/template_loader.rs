//! Template path resolution (local/global) and YAML deserialization.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use miette::{NamedSource, SourceSpan};

use crate::templates::{
    ParsePhase, RawTemplate, TemplateEntry, TemplateError, TemplateMeta, TemplateResult,
    TemplateSource,
};

// ── Resolution ──────────────────────────────────────────────────────

/// Local templates directory relative to repo root.
pub fn local_dir(root: &Path) -> PathBuf {
    root.join("templates")
}

/// Global templates directory: `$HOME/.config/godmode/templates/`.
pub fn global_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("godmode")
            .join("templates")
    })
}

pub(crate) fn candidate_filenames(name: &str) -> [String; 2] {
    [format!("{name}.yaml"), format!("{name}.template.yaml")]
}

/// Locate a template file by name. Checks local dir first, then global.
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

/// List all templates in local and global dirs.
pub fn list(root: &Path) -> TemplateResult<Vec<TemplateEntry>> {
    let mut entries: Vec<TemplateEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

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

// ── Loading ─────────────────────────────────────────────────────────

/// Load only metadata from a template file (no substitution).
pub(crate) fn load_meta(path: &Path) -> TemplateResult<TemplateMeta> {
    let raw = read_template(path)?;
    let t: RawTemplate = serde_yaml::from_str(&raw)
        .map_err(|source| parse_error(path, &raw, source, ParsePhase::Metadata))?;
    Ok(TemplateMeta {
        name: t.meta.name,
        description: t.meta.description,
    })
}

pub(crate) fn read_dir(path: &Path) -> TemplateResult<std::fs::ReadDir> {
    std::fs::read_dir(path).map_err(|source| TemplateError::ReadDir {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn read_template(path: &Path) -> TemplateResult<String> {
    std::fs::read_to_string(path).map_err(|source| TemplateError::Read {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn parse_error(
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
