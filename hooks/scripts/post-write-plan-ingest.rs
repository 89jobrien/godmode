#!/usr/bin/env rust-script
//! post-write-plan-ingest.rs — PostToolUse/Write hook (replaces post-write-plan-ingest.nu)
//! Detects plan files written by Claude and auto-ingests them into the task graph.
//! Always exits 0.
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde_json::Value;
use std::{
    fs,
    io::Read,
    path::Path,
    process::{self, Command},
};

fn git_root() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

fn main() {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    let input: Value = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => process::exit(0),
    };

    // Bail if not in a godmode repo
    let root = match git_root() {
        Some(r) => r,
        None => process::exit(0),
    };
    let task_file = format!("{root}/.ctx/godmode/tasks.yaml");
    let legacy_task_file = format!("{root}/.ctx/GODMODE.tasks.yaml");
    if !Path::new(&task_file).exists() && !Path::new(&legacy_task_file).exists() {
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

    // Extract file_path from tool_input
    let file_path = input
        .pointer("/tool_input/file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if file_path.is_empty() {
        process::exit(0);
    }

    // Check if this is a plan file
    let is_plan = file_path.ends_with(".plan.md")
        || (file_path.contains("_WORKING_DIR/") && file_path.ends_with(".md"));
    if !is_plan {
        process::exit(0);
    }

    let path = Path::new(file_path);
    if !path.exists() {
        process::exit(0);
    }

    // Check content contains plan task headings
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => process::exit(0),
    };
    if !content.contains("### Task") {
        process::exit(0);
    }

    // Run godmode plan ingest
    let result = Command::new("godmode")
        .args(["plan", "ingest", file_path])
        .output();

    match result {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stdout = stdout.trim();
            let basename = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.to_string());
            if stdout.is_empty() {
                println!("[godmode] Auto-ingested plan from {basename}");
            } else {
                println!("[godmode] Auto-ingested plan from {basename}:\n{stdout}");
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let basename = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.to_string());
            eprintln!(
                "[godmode] Plan ingest failed for {basename}: {}",
                stderr.trim()
            );
        }
        Err(_) => {}
    }
}
