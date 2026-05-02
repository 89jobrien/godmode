//! Integration with `doob` — the todo/backlog CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::detect;

/// Call `doob todo list -p <project> --json` and return the parsed JSON value.
pub fn todo_list(project: &str) -> Result<serde_json::Value> {
    let out = Command::new("doob")
        .args(["todo", "list", "-p", project, "--json"])
        .output()
        .context("doob not found on PATH")?;
    if !out.status.success() {
        bail!(
            "doob todo list failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).context("doob todo list: invalid JSON")?;
    Ok(v)
}

/// Return the highest-priority pending todo for a project, or None if empty.
pub fn todo_next(project: &str) -> Result<Option<serde_json::Value>> {
    let v = todo_list(project)?;
    let todos = v
        .get("todos")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();
    // doob returns todos already sorted by priority descending; take first pending.
    let next = todos
        .into_iter()
        .find(|t| t.get("status").and_then(|s| s.as_str()) == Some("pending"));
    Ok(next)
}

/// Detect project name from nearest Cargo.toml and call `todo_next`.
pub fn todo_next_for_root(root: &Path) -> Result<Option<serde_json::Value>> {
    let project = detect::package_name(root)?;
    todo_next(&project)
}
