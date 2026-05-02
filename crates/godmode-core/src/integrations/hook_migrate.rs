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
