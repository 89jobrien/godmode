//! task-driven-development — PreToolUse/Bash hook.
//! Warns if running cargo build/run without test changes in working tree.

use std::path::Path;

/// Run the task-driven-development hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str) -> String {
    let is_build = command.contains("cargo build") || command.contains("cargo run");
    if !is_build {
        return String::new();
    }

    // Exclude test/check/clippy variants
    if command.contains("cargo test")
        || command.contains("cargo nextest")
        || command.contains("cargo check")
        || command.contains("cargo clippy")
    {
        return String::new();
    }

    // Check for test-related changes in working tree
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &root.display().to_string(),
            "diff",
            "HEAD",
            "--name-only",
        ])
        .output();

    let has_test_changes = match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .any(|f| f.contains("test") || f.ends_with("_test.rs"))
        }
        _ => return String::new(),
    };

    if has_test_changes {
        String::new()
    } else {
        "[godmode:tdd] Building without tests — write a failing test first (see tdd-tasks.yaml)"
            .to_string()
    }
}
