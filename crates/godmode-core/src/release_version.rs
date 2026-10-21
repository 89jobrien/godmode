//! Version parsing, bump logic, and Cargo.toml cross-validation.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::integrations::subprocess::command_in;

#[derive(Debug, Deserialize)]
pub(crate) struct VersionBumpConfig {
    #[serde(default = "default_workspace_manifest")]
    pub workspace_manifest: String,
    pub files: Vec<FileTarget>,
}

fn default_workspace_manifest() -> String {
    "Cargo.toml".to_string()
}

#[derive(Debug, Deserialize)]
pub(crate) struct FileTarget {
    pub path: String,
    pub field: String,
}

// ── Config loading ──────────────────────────────────────────────────

pub(crate) fn config_path(root: &Path) -> PathBuf {
    root.join(".version-bump.json")
}

pub(crate) fn load_config(root: &Path) -> Result<VersionBumpConfig> {
    let path = config_path(root);
    let content =
        fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?;
    parse_json(&content, &path)
}

pub(crate) fn read_version_field(root: &Path, target: &FileTarget) -> Result<String> {
    let path = root.join(&target.path);
    let content =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let json: serde_json::Value = parse_json(&content, &path)?;
    json.get(&target.field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .with_context(|| format!("field '{}' not found in {}", target.field, path.display()))
}

pub(crate) fn write_version_field(root: &Path, target: &FileTarget, version: &str) -> Result<()> {
    let path = root.join(&target.path);
    let content =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let mut json: serde_json::Value = parse_json(&content, &path)?;
    let obj = json
        .as_object_mut()
        .with_context(|| format!("expected JSON object in {}", path.display()))?;
    obj.insert(
        target.field.clone(),
        serde_json::Value::String(version.to_string()),
    );
    let updated = serde_json::to_string_pretty(&json)?;
    fs::write(&path, updated + "\n")
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub(crate) fn parse_json<T: serde::de::DeserializeOwned>(content: &str, path: &Path) -> Result<T> {
    serde_json::from_str(content).with_context(|| format!("invalid JSON in {}", path.display()))
}

pub(crate) fn bump_patch(version: &str) -> Result<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        bail!("version '{version}' is not semver (expected MAJOR.MINOR.PATCH)");
    }
    let patch: u64 = parts[2]
        .parse()
        .with_context(|| format!("patch component '{}' is not a number", parts[2]))?;
    Ok(format!("{}.{}.{}", parts[0], parts[1], patch + 1))
}

// ── Cross-validation ────────────────────────────────────────────────

/// Extracts the version from a Cargo manifest's `[workspace.package]` section.
pub fn extract_cargo_workspace_version(content: &str) -> Option<String> {
    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
        } else if trimmed.starts_with('[') {
            in_workspace_package = false;
        } else if in_workspace_package && let Some(rest) = trimmed.strip_prefix("version") {
            let rest = rest.trim().strip_prefix('=')?;
            let rest = rest.trim().trim_matches('"');
            return Some(rest.to_string());
        }
    }
    None
}

/// Returns the nearest reachable Git tag without a leading `v`.
pub fn latest_tag_version(root: &Path) -> Option<String> {
    let out = command_in("git", &["describe", "--tags", "--abbrev=0"], root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Some(tag.strip_prefix('v').unwrap_or(&tag).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git_harness::{GitRepo, PinnedGitDir};

    /// Regression: `git describe` must read `root`, not the repository an
    /// ambient `GIT_DIR` points at.
    #[test]
    fn latest_tag_version_reads_the_given_root_under_a_pinned_git_dir() {
        let fixture = GitRepo::init();
        fixture.commit("fixture commit", None);
        fixture.tag("v1.0.0");

        let decoy = GitRepo::init();
        decoy.commit("decoy commit", None);
        decoy.tag("v9.9.9");

        let _pin = PinnedGitDir::pointing_at(&decoy);

        assert_eq!(latest_tag_version(fixture.path()).as_deref(), Some("1.0.0"));
    }
}
