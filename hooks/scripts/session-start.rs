#!/usr/bin/env rust-script
//! session-start.rs — SessionStart hook (replaces session-start.nu)
//! Runs `godmode handon` when .ctx/GODMODE.tasks.yaml exists in the repo root.
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
    path::PathBuf,
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

fn main() {
    // Read and parse stdin (hook input JSON) — ignore parse failures
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    let _input: Value = serde_json::from_str(&buf).unwrap_or(Value::Null);

    let root = match git_root() {
        Some(r) => r,
        None => process::exit(0),
    };

    // Fire trace event unconditionally
    let plugin_root = std::env::var("CLAUDE_PLUGIN_ROOT").unwrap_or_default();
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

    // Check for task file
    let task_file = root.join(".ctx/GODMODE.tasks.yaml");
    if !task_file.exists() {
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
