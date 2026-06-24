use std::collections::BTreeMap;
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
    // Pre-flight: warn on version drift before bumping
    let drift = validate_versions(root).unwrap_or_default();
    if !drift.is_empty() {
        eprintln!("Warning: version drift detected before bump:");
        for w in &drift {
            eprintln!("  - {w}");
        }
    }

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
// Changelog
// ---------------------------------------------------------------------------

/// One versioned section in CHANGELOG.md.
#[derive(Debug)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub sections: BTreeMap<String, Vec<String>>,
}

/// Recognised conventional-commit type prefixes → section heading.
fn commit_type_heading(prefix: &str) -> &'static str {
    match prefix {
        "feat" => "Features",
        "fix" => "Bug Fixes",
        "chore" => "Chores",
        "docs" => "Documentation",
        "refactor" => "Refactoring",
        "test" => "Tests",
        _ => "Other",
    }
}

/// Classify a commit subject line into a (heading, message) pair.
fn classify(subject: &str) -> (&'static str, String) {
    // Match `type:` or `type(scope):` prefix
    if let Some(colon) = subject.find(':') {
        let prefix = subject[..colon].split('(').next().unwrap_or("").trim();
        let msg = subject[colon + 1..].trim().to_string();
        let heading = commit_type_heading(prefix);
        if heading != "Other" || prefix.chars().all(|c| c.is_alphabetic()) {
            return (heading, msg);
        }
    }
    ("Other", subject.to_string())
}

/// Generate a changelog entry from commits since the latest tag.
pub fn generate_changelog(root: &Path) -> Result<ChangelogEntry> {
    let root_str = root.to_str().unwrap_or(".");

    // Get latest tag
    let tag_out = Command::new("git")
        .args(["-C", root_str, "describe", "--tags", "--abbrev=0"])
        .output()
        .context("failed to run git describe")?;
    let since = if tag_out.status.success() {
        let tag = String::from_utf8_lossy(&tag_out.stdout).trim().to_string();
        format!("{}..HEAD", tag)
    } else {
        // No tags yet — use all commits
        "HEAD".to_string()
    };

    // Get commit subjects since tag
    let log_out = Command::new("git")
        .args(["-C", root_str, "log", &since, "--format=%s"])
        .output()
        .context("failed to run git log")?;
    if !log_out.status.success() {
        bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&log_out.stderr)
        );
    }

    let mut sections: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in String::from_utf8_lossy(&log_out.stdout).lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (heading, msg) = classify(line);
        sections.entry(heading.to_string()).or_default().push(msg);
    }

    let version = current_version(root)?;
    let date = Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(ChangelogEntry {
        version,
        date,
        sections,
    })
}

/// Prepend a new section to CHANGELOG.md (creates the file if absent).
pub fn write_changelog(root: &Path, entry: &ChangelogEntry) -> Result<()> {
    let path = root.join("CHANGELOG.md");
    let existing = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut new_section = format!("## [{}] - {}\n", entry.version, entry.date);

    // Ordered headings for deterministic output
    let order = [
        "Features",
        "Bug Fixes",
        "Refactoring",
        "Tests",
        "Documentation",
        "Chores",
        "Other",
    ];
    for heading in order {
        if let Some(items) = entry.sections.get(heading) {
            new_section.push_str(&format!("\n### {}\n", heading));
            for item in items {
                new_section.push_str(&format!("- {}\n", item));
            }
        }
    }
    // Any headings not in the ordered list
    for (heading, items) in &entry.sections {
        if !order.contains(&heading.as_str()) {
            new_section.push_str(&format!("\n### {}\n", heading));
            for item in items {
                new_section.push_str(&format!("- {}\n", item));
            }
        }
    }

    let content = if existing.is_empty() {
        format!("# Changelog\n\n{}\n", new_section)
    } else {
        // Prepend after first line if it's a `# Changelog` heading, else just prepend
        if existing.starts_with("# Changelog") {
            let rest = existing
                .split_once('\n')
                .map(|x| x.1)
                .unwrap_or("")
                .trim_start_matches('\n');
            format!("# Changelog\n\n{}\n{}", new_section, rest)
        } else {
            format!("{}\n{}", new_section, existing)
        }
    };

    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Version cross-validation
// ---------------------------------------------------------------------------

/// Cross-check plugin.json version, Cargo.toml workspace version, and latest git tag.
/// Returns a list of warnings (empty = all consistent).
pub fn validate_versions(root: &Path) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // 1. Plugin version from .version-bump.json
    let plugin_version = current_version(root)?;

    // 2. Cargo.toml workspace version
    let cargo_toml = root.join("Cargo.toml");
    let cargo_version = if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        extract_cargo_workspace_version(&content)
    } else {
        None
    };

    // 3. Latest git tag
    let tag_version = latest_tag_version(root);

    // Cross-check plugin vs Cargo
    if let Some(ref cv) = cargo_version
        && *cv != plugin_version
    {
        warnings.push(format!(
            "version mismatch: plugin.json={plugin_version}, Cargo.toml={cv}"
        ));
    }

    // Cross-check plugin vs git tag
    if let Some(ref tv) = tag_version
        && *tv != plugin_version
    {
        warnings.push(format!(
            "version mismatch: plugin.json={plugin_version}, latest tag=v{tv}"
        ));
    }

    // Cross-check Cargo vs git tag
    if let Some(ref cv) = cargo_version
        && let Some(ref tv) = tag_version
        && cv != tv
    {
        warnings.push(format!(
            "version mismatch: Cargo.toml={cv}, latest tag=v{tv}"
        ));
    }

    Ok(warnings)
}

fn extract_cargo_workspace_version(content: &str) -> Option<String> {
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

fn latest_tag_version(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "describe",
            "--tags",
            "--abbrev=0",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Some(tag.strip_prefix('v').unwrap_or(&tag).to_string())
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
    parse_json(&content, &path)
}

fn read_version_field(root: &Path, target: &FileTarget) -> Result<String> {
    let path = root.join(&target.path);
    let content =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let json: serde_json::Value = parse_json(&content, &path)?;
    json.get(&target.field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .with_context(|| format!("field '{}' not found in {}", target.field, path.display()))
}

fn write_version_field(root: &Path, target: &FileTarget, version: &str) -> Result<()> {
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

fn parse_json<T: serde::de::DeserializeOwned>(content: &str, path: &Path) -> Result<T> {
    serde_json::from_str(content).with_context(|| format!("invalid JSON in {}", path.display()))
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

    #[test]
    fn extract_cargo_workspace_version_parses() {
        let toml = r#"
[workspace]
members = ["crates/*"]

[workspace.package]
version = "0.6.0"
edition = "2024"
"#;
        assert_eq!(
            extract_cargo_workspace_version(toml),
            Some("0.6.0".to_string())
        );
    }

    #[test]
    fn extract_cargo_workspace_version_returns_none_if_missing() {
        let toml = "[workspace]\nmembers = [\"crates/*\"]\n";
        assert_eq!(extract_cargo_workspace_version(toml), None);
    }

    #[test]
    fn validate_versions_reports_no_warnings_when_consistent() {
        let tmp = make_fixture("1.0.0");
        // No git repo, so tag check returns None — only plugin version is checked
        let warnings = validate_versions(tmp.path()).unwrap();
        // No Cargo.toml → no cross-check, no git tag → no cross-check
        assert!(warnings.is_empty());
    }
}
