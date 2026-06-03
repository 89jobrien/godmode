use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Result of running a single migration script.
#[derive(Debug)]
pub struct MigrationResult {
    pub name: String,
    pub ok: bool,
    pub output: String,
}

/// Run all `hooks/migrations/*.nu` scripts in sorted order.
///
/// Each script is expected to be idempotent. Individual failures are recorded
/// but do not prevent subsequent migrations from running.
pub fn run_migrations(root: &Path) -> Result<Vec<MigrationResult>> {
    let migrations_dir = root.join("hooks").join("migrations");
    if !migrations_dir.exists() {
        return Ok(vec![]);
    }

    let mut scripts: Vec<std::path::PathBuf> = std::fs::read_dir(&migrations_dir)
        .with_context(|| format!("cannot read {}", migrations_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "nu").unwrap_or(false))
        .collect();

    scripts.sort();

    let mut results = Vec::new();
    for script in &scripts {
        let name = script
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| script.display().to_string());

        let out = Command::new("nu").arg(script).current_dir(root).output();

        let result = match out {
            Ok(o) => {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                MigrationResult {
                    name,
                    ok: o.status.success(),
                    output: combined.trim().to_string(),
                }
            }
            Err(e) => MigrationResult {
                name,
                ok: false,
                output: format!("failed to launch nu: {}", e),
            },
        };
        results.push(result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_migrations_dir_returns_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let results = run_migrations(dir.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn empty_migrations_dir_returns_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("hooks/migrations")).unwrap();
        let results = run_migrations(dir.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn non_nu_files_are_ignored() {
        let dir = tempfile::TempDir::new().unwrap();
        let mig_dir = dir.path().join("hooks/migrations");
        std::fs::create_dir_all(&mig_dir).unwrap();
        std::fs::write(mig_dir.join("readme.md"), "not a script").unwrap();
        std::fs::write(mig_dir.join("script.sh"), "#!/bin/sh").unwrap();
        let results = run_migrations(dir.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn scripts_run_in_sorted_order() {
        let dir = tempfile::TempDir::new().unwrap();
        let mig_dir = dir.path().join("hooks/migrations");
        std::fs::create_dir_all(&mig_dir).unwrap();
        // Create two nu scripts that write to a shared file in order.
        // Even if nu isn't available, we verify the names are sorted.
        std::fs::write(mig_dir.join("002-second.nu"), "echo second").unwrap();
        std::fs::write(mig_dir.join("001-first.nu"), "echo first").unwrap();
        let results = run_migrations(dir.path()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "001-first.nu");
        assert_eq!(results[1].name, "002-second.nu");
    }
}
