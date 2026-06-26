//! Release orchestration: version bumping, tagging, pushing, and changelog generation.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

#[path = "release_tag.rs"]
pub mod release_tag;
#[path = "release_version.rs"]
pub mod release_version;

pub use release_tag::{push, tag};
pub use release_version::{extract_cargo_workspace_version, latest_tag_version};

use release_version::{bump_patch, load_config, read_version_field, write_version_field};

// ── Public types ────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct ReleaseInfo {
    pub old_version: String,
    pub new_version: String,
    pub tag: String,
    pub pushed: bool,
}

// ── Public API ──────────────────────────────────────────────────────

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

// ── Changelog ───────────────────────────────────────────────────────

/// One versioned section in CHANGELOG.md.
#[derive(Debug)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub sections: BTreeMap<String, Vec<String>>,
}

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

fn classify(subject: &str) -> (&'static str, String) {
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

    let tag_out = Command::new("git")
        .args(["-C", root_str, "describe", "--tags", "--abbrev=0"])
        .output()
        .context("failed to run git describe")?;
    let since = if tag_out.status.success() {
        let tag = String::from_utf8_lossy(&tag_out.stdout).trim().to_string();
        format!("{}..HEAD", tag)
    } else {
        "HEAD".to_string()
    };

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

// ── Version cross-validation ────────────────────────────────────────

/// Cross-check plugin.json version, Cargo.toml workspace version, and latest git tag.
pub fn validate_versions(root: &Path) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    let plugin_version = current_version(root)?;

    let cargo_toml = root.join("Cargo.toml");
    let cargo_version = if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        extract_cargo_workspace_version(&content)
    } else {
        None
    };

    let tag_version = latest_tag_version(root);

    if let Some(ref cv) = cargo_version
        && *cv != plugin_version
    {
        warnings.push(format!(
            "version mismatch: plugin.json={plugin_version}, Cargo.toml={cv}"
        ));
    }

    if let Some(ref tv) = tag_version
        && *tv != plugin_version
    {
        warnings.push(format!(
            "version mismatch: plugin.json={plugin_version}, latest tag=v{tv}"
        ));
    }

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

// ── Tests ───────────────────────────────────────────────────────────

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
        use release_version::bump_patch;
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
        let warnings = validate_versions(tmp.path()).unwrap();
        assert!(warnings.is_empty());
    }
}
