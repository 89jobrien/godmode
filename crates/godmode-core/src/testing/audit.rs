//! Compile-time and runtime auditing utilities.
//!
//! - [`assert_implements!`] — zero-cost compile-time trait-bound check.
//! - [`DepAudit`] — runtime check that `Cargo.toml` deps match an allowlist.
//! - [`SnapshotAudit`] — golden-file key/value comparison.

/// Assert at compile time that `$type` satisfies all listed trait bounds.
///
/// Zero-cost: expands to a never-called function that forces the compiler
/// to verify all bounds.
///
/// ```rust
/// use godmode_core::testing::audit::assert_implements;
///
/// #[derive(Debug, Clone)]
/// struct Foo;
///
/// assert_implements!(Foo: std::fmt::Debug + Clone + Send + Sync);
/// ```
#[macro_export]
macro_rules! assert_implements {
    ($type:ty : $($bound:tt)+) => {
        const _: fn() = || {
            fn _check<T: $($bound)+>() {}
            _check::<$type>();
        };
    };
}

pub use assert_implements;

// ---------------------------------------------------------------------------
// DepAudit
// ---------------------------------------------------------------------------

/// Runtime check that a `Cargo.toml`'s `[dependencies]` section contains only
/// names from a given allowlist.
pub struct DepAudit {
    cargo_toml_path: std::path::PathBuf,
    allowed: std::collections::HashSet<String>,
}

impl DepAudit {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            cargo_toml_path: path.into(),
            allowed: std::collections::HashSet::new(),
        }
    }

    /// Add a dependency name to the allowlist (builder style).
    pub fn allow(mut self, dep: impl Into<String>) -> Self {
        self.allowed.insert(dep.into());
        self
    }

    /// Panic if any `[dependencies]` name is not in the allowlist.
    ///
    /// Only top-level `[dependencies]` keys are checked — dev/build deps
    /// are ignored.
    pub fn assert_no_unlisted(&self) {
        let bytes = match std::fs::read(&self.cargo_toml_path) {
            Ok(b) => b,
            Err(e) => panic!("DepAudit: cannot read {:?}: {}", self.cargo_toml_path, e),
        };
        let content = String::from_utf8_lossy(&bytes);

        let mut in_deps = false;
        let mut unlisted: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('[') {
                in_deps = trimmed == "[dependencies]";
                continue;
            }

            if !in_deps || trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(name) = trimmed.split(['=', '.']).next() {
                let name = name.trim().to_owned();
                if !name.is_empty() && !self.allowed.contains(&name) {
                    unlisted.push(name);
                }
            }
        }

        if !unlisted.is_empty() {
            panic!(
                "DepAudit: unlisted dependencies in {:?}:\n  {}",
                self.cargo_toml_path,
                unlisted.join(", ")
            );
        }
    }
}

// ---------------------------------------------------------------------------
// SnapshotAudit
// ---------------------------------------------------------------------------

use anyhow::{Context, Result};
use std::collections::BTreeMap;

/// Accumulate string key/value observations and compare against a golden file.
///
/// On first run (or when `UPDATE_SNAPSHOTS=1` is set), the snapshot is
/// written. On subsequent runs it is compared — mismatches cause an error.
pub struct SnapshotAudit {
    snapshot_path: std::path::PathBuf,
    entries: BTreeMap<String, String>,
    update_env_var: String,
}

impl SnapshotAudit {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            snapshot_path: path.into(),
            entries: BTreeMap::new(),
            update_env_var: "UPDATE_SNAPSHOTS".to_owned(),
        }
    }

    /// Override the env var name that triggers snapshot update
    /// (default: `UPDATE_SNAPSHOTS`).
    pub fn update_env(mut self, var: impl Into<String>) -> Self {
        self.update_env_var = var.into();
        self
    }

    pub fn record(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    /// Compare recorded observations against the snapshot file, or write it
    /// if it doesn't exist or the update env var is set to `"1"`.
    pub fn assert_snapshot(&self) -> Result<()> {
        let current = self.render();
        let update = std::env::var(&self.update_env_var).as_deref() == Ok("1");

        if update || !self.snapshot_path.exists() {
            if let Some(parent) = self.snapshot_path.parent() {
                std::fs::create_dir_all(parent).context("failed to create snapshot directory")?;
            }
            std::fs::write(&self.snapshot_path, &current).context("failed to write snapshot")?;
            return Ok(());
        }

        let stored =
            std::fs::read_to_string(&self.snapshot_path).context("failed to read snapshot file")?;

        if current != stored {
            anyhow::bail!(
                "Snapshot mismatch at {:?}.\n\
                 Set {}=1 to accept new snapshot.\n\
                 --- stored ---\n{}\n--- current ---\n{}",
                self.snapshot_path,
                self.update_env_var,
                stored,
                current
            );
        }

        Ok(())
    }

    fn render(&self) -> String {
        self.entries
            .iter()
            .map(|(k, v)| format!("{k} = {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct ExampleType;

    assert_implements!(ExampleType: std::fmt::Debug + Clone + Send + Sync);

    #[test]
    fn dep_audit_passes_when_all_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        std::fs::write(
            &path,
            "[package]\nname = \"test\"\n\n[dependencies]\n\
             anyhow = \"1\"\ntokio = \"1\"\n",
        )
        .unwrap();

        DepAudit::new(&path)
            .allow("anyhow")
            .allow("tokio")
            .assert_no_unlisted();
    }

    #[test]
    #[should_panic(expected = "unlisted dependencies")]
    fn dep_audit_fails_on_unlisted_dep() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        std::fs::write(
            &path,
            "[package]\nname = \"test\"\n\n[dependencies]\n\
             anyhow = \"1\"\nsecret-dep = \"0.1\"\n",
        )
        .unwrap();

        DepAudit::new(&path).allow("anyhow").assert_no_unlisted();
    }

    #[test]
    fn snapshot_audit_writes_and_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snap.txt");

        let mut audit = SnapshotAudit::new(&path);
        audit.record("field_a", "present");
        audit.record("field_b", "absent");
        audit.assert_snapshot().unwrap();
        audit.assert_snapshot().unwrap();
    }

    #[test]
    fn snapshot_audit_detects_divergence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snap.txt");

        let mut audit = SnapshotAudit::new(&path);
        audit.record("key", "v1");
        audit.assert_snapshot().unwrap();

        audit.record("key", "v2");
        let result = audit.assert_snapshot();
        assert!(result.is_err());
    }
}
