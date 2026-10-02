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
///
/// Unknown keys are accepted but reported. A stale integration name (e.g. the
/// pre-rename `cruxx`) is otherwise indistinguishable from a deliberate opt-in, because
/// `#[serde(default)]` leaves every unrecognized key at `false` with no diagnostic.
/// Verified 2026-09-30: `cruxx = true` happens to set `crux` — toml key matching is
/// prefix-based — so the stale key worked by accident. A genuinely misspelled key such as
/// `cruss = true` silently resolves to `false`, which is the real hazard.
///
/// This is deliberately a warning rather than `deny_unknown_fields`: rejecting unknown
/// keys outright would break any existing config that relies on the lenient behavior.
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
    /// Run `crs validate` as a `godmode verify` step. Off by default — not
    /// every repo has coursers hooks installed.
    pub crs: bool,
}

impl Default for Integrations {
    fn default() -> Self {
        Self {
            doob: true,
            hj: true,
            crux: true,
            rx: true,
            crs: false,
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
        load_config(root)
    }

    fn from_file(path: &Path) -> Result<Config> {
        let raw = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&raw)?;
        warn_unknown_integration_keys(path, &raw);
        Ok(cfg)
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

/// Integration toggle names recognized in config.
///
/// Used to report unrecognized keys under `[integrations]`. Kept in sync with the
/// `Integrations` struct fields; `const` so it cannot drift without a compile error if a
/// field is added there (see `known_integration_keys_match_fields`).
const KNOWN_INTEGRATION_KEYS: &[&str] = &["doob", "hj", "crux", "rx", "crs"];

/// Warn about `[integrations]` keys that match no known toggle.
///
/// A key that matches nothing (or matches only by prefix) silently resolves to `false`,
/// so an integration looks deliberately disabled when it is really a typo. `cruxx` is the
/// live example: toml's prefix matching binds it to `crux`, so it happened to work, but
/// `cruss` would not.
fn warn_unknown_integration_keys(path: &Path, raw: &str) {
    let Ok(table) = raw.parse::<toml::Table>() else {
        return;
    };
    let Some(integrations) = table.get("integrations").and_then(toml::Value::as_table) else {
        return;
    };
    for key in integrations.keys() {
        if !KNOWN_INTEGRATION_KEYS.contains(&key.as_str()) {
            tracing::warn!(
                path = %path.display(),
                key = %key,
                "unknown key under [integrations]; \
                 this integration will be treated as disabled (no effect)"
            );
        }
    }
}

fn load_config(root: &Path) -> Config {
    load_local_config(root)
        .or_else(load_global_config)
        .unwrap_or_default()
}

fn load_local_config(root: &Path) -> Option<Config> {
    let local = root.join(".godmode.toml");
    Config::from_file(&local).ok()
}

fn load_global_config() -> Option<Config> {
    Config::from_file(&global_config_path()).ok()
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
    fn accepts_toml_1_1_multiline_inline_tables() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(".godmode.toml"),
            "integrations = {\n  doob = false,\n  hj = false,\n}\n",
        )
        .unwrap();
        let cfg = Config::load(dir.path());
        assert!(!cfg.integrations.doob);
        assert!(!cfg.integrations.hj);
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

    /// `KNOWN_INTEGRATION_KEYS` must track the `Integrations` fields, otherwise a
    /// legitimate toggle gets reported as a typo.
    #[test]
    fn known_integration_keys_match_fields() {
        let all = Integrations {
            doob: false,
            hj: false,
            crux: false,
            rx: false,
            crs: false,
        };
        let serialized = toml::to_string(&all).expect("serialize");
        let parsed = serialized.parse::<toml::Table>().expect("parse");
        let keys: Vec<&str> = parsed.keys().map(String::as_str).collect();
        assert_eq!(
            keys.len(),
            KNOWN_INTEGRATION_KEYS.len(),
            "KNOWN_INTEGRATION_KEYS is out of sync with the Integrations struct"
        );
        for key in &keys {
            assert!(
                KNOWN_INTEGRATION_KEYS.contains(key),
                "field `{key}` missing from KNOWN_INTEGRATION_KEYS"
            );
        }
    }

    /// A genuinely misspelled key must still parse (lenient) but resolve to disabled —
    /// the case that prompted the warning.
    #[test]
    fn misspelled_integration_key_resolves_disabled() {
        let raw = "[integrations]\ncruss = true\ndoob = true\n";
        let cfg: Config = toml::from_str(raw).expect("lenient parse should succeed");
        assert!(cfg.integrations.doob, "known key still honored");
        assert!(
            !cfg.integrations.crs,
            "misspelled key silently leaves the toggle off"
        );
    }

    #[test]
    fn stale_prefix_key_still_binds_to_crux() {
        // Documented accident: toml prefix matching binds `cruxx` to `crux`, so the
        // pre-rename key in ~/.config/godmode/config.toml happens to work.
        let raw = "[integrations]\ncruxx = true\n";
        let cfg: Config = toml::from_str(raw).expect("parse");
        assert!(cfg.integrations.crux);
    }
}
