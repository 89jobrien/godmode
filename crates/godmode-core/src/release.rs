use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct VersionBumpConfig {
    files: Vec<FileTarget>,
}

#[derive(Debug, Deserialize)]
struct FileTarget {
    path: String,
    field: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ReleaseInfo {
    pub old_version: String,
    pub new_version: String,
    pub tag: String,
    pub pushed: bool,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Read the current version from `plugin.json` (first file in `.version-bump.json`).
pub fn current_version(root: &Path) -> Result<String> {
    let cfg = load_config(root)?;
    let target = cfg
        .files
        .first()
        .context("no files in .version-bump.json")?;
    read_version_field(root, target)
}

/// Increment the patch component across all files in `.version-bump.json`.
/// Pass `explicit` to set an exact version instead of auto-incrementing.
pub fn bump(root: &Path, explicit: Option<&str>) -> Result<ReleaseInfo> {
    let cfg = load_config(root)?;
    if cfg.files.is_empty() {
        bail!("no files listed in .version-bump.json");
    }

    let old_version = read_version_field(root, &cfg.files[0])?;
    let new_version = match explicit {
        Some(v) => v.to_string(),
        None => bump_patch(&old_version)?,
    };

    for target in &cfg.files {
        write_version_field(root, target, &new_version)?;
    }

    let tag = format!("v{new_version}");
    Ok(ReleaseInfo {
        old_version,
        new_version,
        tag,
        pushed: false,
    })
}

/// Create an annotated git tag for the current version.
pub fn tag(root: &Path) -> Result<String> {
    let version = current_version(root)?;
    let tag_name = format!("v{version}");
    let message = format!("release {tag_name}");

    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "tag",
            "-a",
            &tag_name,
            "-m",
            &message,
        ])
        .output()
        .context("failed to run git tag")?;

    if !out.status.success() {
        bail!("git tag failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    Ok(tag_name)
}

/// Push the current branch and version tag to origin.
pub fn push(root: &Path) -> Result<()> {
    let version = current_version(root)?;
    let tag_name = format!("v{version}");

    // Push branch
    let out = Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."), "push"])
        .output()
        .context("failed to run git push")?;
    if !out.status.success() {
        bail!("git push failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    // Push tag
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "push",
            "origin",
            &tag_name,
        ])
        .output()
        .context("failed to run git push origin <tag>")?;
    if !out.status.success() {
        bail!(
            "git push origin {tag_name} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn config_path(root: &Path) -> PathBuf {
    root.join(".version-bump.json")
}

fn load_config(root: &Path) -> Result<VersionBumpConfig> {
    let path = config_path(root);
    let content =
        fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn read_version_field(root: &Path, target: &FileTarget) -> Result<String> {
    let path = root.join(&target.path);
    let content =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    json.get(&target.field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .with_context(|| format!("field '{}' not found in {}", target.field, path.display()))
}

fn write_version_field(root: &Path, target: &FileTarget, version: &str) -> Result<()> {
    let path = root.join(&target.path);
    let content =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
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

fn bump_patch(version: &str) -> Result<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        bail!("version '{version}' is not semver (expected MAJOR.MINOR.PATCH)");
    }
    let patch: u64 = parts[2]
        .parse()
        .with_context(|| format!("patch component '{}' is not a number", parts[2]))?;
    Ok(format!("{}.{}.{}", parts[0], parts[1], patch + 1))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_fixture(version: &str) -> TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join(".claude-plugin")).unwrap();
        fs::write(
            root.join(".claude-plugin/plugin.json"),
            format!(r#"{{"name":"godmode","version":"{version}","author":{{"name":"Joe"}},"description":"x"}}"#),
        )
        .unwrap();

        fs::write(
            root.join(".version-bump.json"),
            r#"{"files":[{"path":".claude-plugin/plugin.json","field":"version"}],"audit":{"exclude":["target"]}}"#,
        )
        .unwrap();

        tmp
    }

    #[test]
    fn current_version_reads_plugin_json() {
        let tmp = make_fixture("1.2.3");
        assert_eq!(current_version(tmp.path()).unwrap(), "1.2.3");
    }

    #[test]
    fn bump_increments_patch() {
        let tmp = make_fixture("1.1.0");
        let info = bump(tmp.path(), None).unwrap();
        assert_eq!(info.old_version, "1.1.0");
        assert_eq!(info.new_version, "1.1.1");
        assert_eq!(info.tag, "v1.1.1");
        // Verify file was actually written
        assert_eq!(current_version(tmp.path()).unwrap(), "1.1.1");
    }

    #[test]
    fn bump_accepts_explicit_version() {
        let tmp = make_fixture("1.0.0");
        let info = bump(tmp.path(), Some("2.0.0")).unwrap();
        assert_eq!(info.new_version, "2.0.0");
        assert_eq!(current_version(tmp.path()).unwrap(), "2.0.0");
    }

    #[test]
    fn bump_patch_arithmetic() {
        assert_eq!(bump_patch("0.0.9").unwrap(), "0.0.10");
        assert_eq!(bump_patch("1.2.3").unwrap(), "1.2.4");
        assert!(bump_patch("1.0").is_err());
        assert!(bump_patch("bad").is_err());
    }

    #[test]
    fn bump_fails_on_missing_version_bump_json() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(bump(tmp.path(), None).is_err());
    }
}
