#!/usr/bin/env rust-script
//! session-start.rs — SessionStart hook (replaces session-start.nu)
//! Runs `godmode handon` when .ctx/godmode/tasks.yaml exists in the repo root.
//! No-ops silently in non-godmode repos. Always exits 0.
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde_json::Value;
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{self, Command},
};

fn git_root() -> Option<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    Some(PathBuf::from(s.trim()))
}

/// `version = "X.Y.Z"` from the `[workspace.package]` table of a Cargo.toml.
fn workspace_version(manifest: &Path) -> Option<String> {
    let text = std::fs::read_to_string(manifest).ok()?;
    let mut in_package_table = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package_table = line.starts_with("[workspace.package]");
            continue;
        }
        if in_package_table {
            if let Some(rest) = line.strip_prefix("version") {
                let value = rest.trim_start().strip_prefix('=')?.trim();
                return Some(value.trim_matches(['"', '\'']).to_string());
            }
        }
    }
    None
}

/// Version reported by the `godmode` binary actually on PATH.
fn installed_version() -> Option<String> {
    let out = Command::new("godmode").arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    // "godmode 0.8.1" -> "0.8.1"
    let text = String::from_utf8(out.stdout).ok()?;
    text.split_whitespace().nth(1).map(str::to_string)
}

/// Warn — once, on one line — when the installed CLI predates the plugin source.
///
/// A stale binary is the failure mode that wastes the most time: `review`,
/// `skill`, and every skill that quotes a subcommand read from the binary, so a
/// drifted install makes the plugin audit itself against a CLI that no longer
/// matches its own docs. Rebuilding here is deliberately out of scope: SessionStart
/// hooks get 15 seconds and a release build takes minutes.
fn warn_on_version_drift(plugin_root: &Path) {
    let manifest = plugin_root.join("Cargo.toml");
    let Some(want) = workspace_version(&manifest) else {
        return;
    };
    let Some(have) = installed_version() else {
        return;
    };
    if want == have {
        return;
    }
    println!(
        "[godmode] installed CLI is {have}, plugin source is {want} — run `cargo xtask install` to refresh"
    );
}

fn main() {
    // Read and parse stdin (hook input JSON) — ignore parse failures
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    let _input: Value = serde_json::from_str(&buf).unwrap_or(Value::Null);

    let root = match git_root() {
        Some(r) => r,
        None => process::exit(0),
    };

    let plugin_root = std::env::var("CLAUDE_PLUGIN_ROOT").unwrap_or_default();

    // Only meaningful when running from a godmode source checkout; no-ops
    // everywhere else.
    if !plugin_root.is_empty() {
        warn_on_version_drift(Path::new(&plugin_root));
    }

    // Fire trace event unconditionally
    if !plugin_root.is_empty() {
        let trace_script = PathBuf::from(&plugin_root)
            .join("hooks/scripts/godmode-trace.rs");
        if trace_script.exists() {
            let _ = Command::new("rust-script")
                .arg(&trace_script)
                .arg("start")
                .arg(&root)
                .output();
        }
    }

    // Check for task file (new path, with legacy fallback)
    let task_file = root.join(".ctx/godmode/tasks.yaml");
    let legacy_task_file = root.join(".ctx/GODMODE.tasks.yaml");
    if !task_file.exists() && !legacy_task_file.exists() {
        process::exit(0);
    }

    // Check godmode is on PATH
    let godmode_ok = Command::new("which")
        .arg("godmode")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !godmode_ok {
        process::exit(0);
    }

    let _ = Command::new("godmode").arg("handon").output();
}
