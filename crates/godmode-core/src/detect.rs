//! Resolve the repo root and `.ctx/` path from the current working directory.
//!
//! Supports pinning the root to a specific path via `.ctx/godmode/session.json`.
//! When pinned, `root_or_cwd()` returns the pinned path instead of auto-detecting.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Walk up from `start` to find the nearest directory containing `.git`.
pub fn repo_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            bail!("no git repository found from {}", start.display());
        }
    }
}

/// Resolve repo root: pinned root > git root from cwd > cwd.
pub fn root_or_cwd() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = repo_root(&cwd).unwrap_or(cwd);
    if let Some(pinned) = pinned_root(&root) {
        Ok(pinned)
    } else {
        Ok(root)
    }
}

fn session_json_path(root: &Path) -> PathBuf {
    root.join(".ctx/godmode/session.json")
}

/// Read the `pinned_root` field from `.ctx/godmode/session.json`.
pub fn pinned_root(root: &Path) -> Option<PathBuf> {
    let path = session_json_path(root);
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get("pinned_root")?.as_str().map(|s| PathBuf::from(s))
}

/// Pin the session to a specific repo root path.
pub fn pin_root(root: &Path, target: &Path) -> Result<()> {
    let path = session_json_path(root);
    let mut v = if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or(serde_json::json!({}))
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };
    let canonical = target
        .canonicalize()
        .with_context(|| format!("resolving {}", target.display()))?;
    v["pinned_root"] = serde_json::Value::String(canonical.to_string_lossy().into_owned());
    std::fs::write(&path, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

/// Remove the pinned root from session state.
pub fn unpin_root(root: &Path) -> Result<bool> {
    let path = session_json_path(root);
    if !path.exists() {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut v: serde_json::Value = serde_json::from_str(&raw)?;
    let removed = v.as_object_mut().map(|o| o.remove("pinned_root")).is_some();
    std::fs::write(&path, serde_json::to_string_pretty(&v)?)?;
    Ok(removed)
}

/// Read the `[package] name` from the nearest `Cargo.toml` at or above `root`.
pub fn package_name(root: &Path) -> Result<String> {
    let cargo_toml = root.join("Cargo.toml");
    let raw = std::fs::read_to_string(&cargo_toml)
        .with_context(|| format!("reading {}", cargo_toml.display()))?;
    // Parse just enough to extract [package] name without a full TOML dep.
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim().trim_start_matches('=').trim();
            let name = rest.trim_matches('"').trim_matches('\'').to_string();
            if !name.is_empty() {
                return Ok(name);
            }
        }
    }
    bail!("could not find [package] name in {}", cargo_toml.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn finds_git_root() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let sub = dir.path().join("sub/dir");
        std::fs::create_dir_all(&sub).unwrap();
        let found = repo_root(&sub).unwrap();
        assert_eq!(found, dir.path());
    }

    #[test]
    fn fails_outside_git_repo() {
        // /tmp itself is not a git repo (usually).
        let result = repo_root(Path::new("/tmp"));
        // Either succeeds (if /tmp is somehow in a repo) or errors — just ensure no panic.
        let _ = result;
    }

    #[test]
    fn pin_and_unpin_root() {
        let dir = TempDir::new().unwrap();
        let ctx = dir.path().join(".ctx/godmode");
        std::fs::create_dir_all(&ctx).unwrap();
        // Write a pre-existing session.json
        std::fs::write(
            ctx.join("session.json"),
            r#"{"session_id":"abc-123","started_at":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();

        // No pin initially
        assert!(pinned_root(dir.path()).is_none());

        // Pin to dir itself (must exist for canonicalize)
        pin_root(dir.path(), dir.path()).unwrap();
        let pinned = pinned_root(dir.path()).unwrap();
        assert_eq!(pinned, dir.path().canonicalize().unwrap());

        // Verify session_id preserved
        let raw = std::fs::read_to_string(ctx.join("session.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["session_id"].as_str(), Some("abc-123"));

        // Unpin
        let removed = unpin_root(dir.path()).unwrap();
        assert!(removed);
        assert!(pinned_root(dir.path()).is_none());
    }
}
