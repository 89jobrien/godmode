//! Configuration for godmode — loaded from `.godmode.toml` (repo-local)
//! or `~/.config/godmode/config.toml` (global fallback).

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Top-level godmode configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Override the auto-detected project name.
    pub project_name: Option<String>,

    /// Integration toggles.
    pub integrations: Integrations,

    /// Handoff output settings.
    pub handoff: Handoff,
}

/// Which external tools godmode will call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Integrations {
    /// Call `doob` for todo sync and handoff item upsert.
    pub doob: bool,
    /// Call `hj` for handoff lifecycle events.
    pub hj: bool,
    /// Append crux trace events to session JSONL.
    pub crux: bool,
    /// Validate task run commands via `rx`.
    pub rx: bool,
}

impl Default for Integrations {
    fn default() -> Self {
        Self {
            doob: true,
            hj: true,
            crux: true,
            rx: true,
        }
    }
}

/// Handoff YAML output settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Handoff {
    /// Write HANDOFF YAML on `godmode handoff`.
    pub enabled: bool,
    /// Sync HANDOFF YAML to doob after writing.
    pub doob_sync: bool,
    /// Maximum number of commit SHAs to include per log entry.
    pub max_commits: usize,
}

impl Default for Handoff {
    fn default() -> Self {
        Self {
            enabled: true,
            doob_sync: true,
            max_commits: 10,
        }
    }
}

impl Config {
    /// Load config: repo-local `.godmode.toml` wins, then global
    /// `~/.config/godmode/config.toml`, then defaults.
    pub fn load(root: &Path) -> Config {
        // Try repo-local first
        let local = root.join(".godmode.toml");
        if let Ok(cfg) = Self::from_file(&local) {
            return cfg;
        }
        // Try global
        if let Ok(cfg) = Self::from_file(&global_config_path()) {
            return cfg;
        }
        Config::default()
    }

    fn from_file(path: &Path) -> Result<Config> {
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    /// Resolve the project name: config override > Cargo.toml > git remote > dir name.
    pub fn project_name(&self, root: &Path) -> String {
        if let Some(ref name) = self.project_name {
            return name.clone();
        }
        if let Ok(name) = crate::detect::package_name(root) {
            return name;
        }
        if let Some(name) = project_name_from_git(root) {
            return name;
        }
        root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

/// Extract project name from `git remote get-url origin` — takes the repo
/// basename, strips `.git` suffix.
fn project_name_from_git(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["-C", root.to_str()?, "remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Handle both SSH (git@...:user/repo.git) and HTTPS (.../repo.git)
    let name = url
        .rsplit('/')
        .next()
        .or_else(|| url.rsplit(':').next())?
        .trim_end_matches(".git")
        .to_string();
    if name.is_empty() { None } else { Some(name) }
}

fn global_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("godmode")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn defaults_are_all_enabled() {
        let cfg = Config::default();
        assert!(cfg.integrations.doob);
        assert!(cfg.integrations.hj);
        assert!(cfg.handoff.enabled);
        assert!(cfg.handoff.doob_sync);
        assert_eq!(cfg.handoff.max_commits, 10);
    }

    #[test]
    fn loads_from_toml() {
        let dir = TempDir::new().unwrap();
        let toml = r#"
project_name = "myproject"

[integrations]
hj = false

[handoff]
max_commits = 5
"#;
        std::fs::write(dir.path().join(".godmode.toml"), toml).unwrap();
        let cfg = Config::load(dir.path());
        assert_eq!(cfg.project_name.as_deref(), Some("myproject"));
        assert!(!cfg.integrations.hj);
        assert!(cfg.integrations.doob); // default preserved
        assert_eq!(cfg.handoff.max_commits, 5);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let cfg = Config::load(Path::new("/tmp/nonexistent"));
        assert!(cfg.project_name.is_none());
        assert!(cfg.integrations.doob);
    }

    #[test]
    fn project_name_fallback_to_dirname() {
        let cfg = Config::default();
        let name = cfg.project_name(Path::new("/tmp/my-project"));
        assert_eq!(name, "my-project");
    }
}
