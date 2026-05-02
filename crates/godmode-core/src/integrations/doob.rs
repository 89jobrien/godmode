//! Integration with `doob` — the todo/backlog CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::detect;

// ---------------------------------------------------------------------------
// Pure logic — testable without shelling out
// ---------------------------------------------------------------------------

/// Parse raw JSON bytes from `doob todo list --json` into a Value.
pub fn parse_todo_list(raw: &[u8]) -> Result<serde_json::Value> {
    serde_json::from_slice(raw).context("doob todo list: invalid JSON")
}

/// Return the first pending todo from a parsed `doob todo list` response.
pub fn find_next_pending(value: &serde_json::Value) -> Option<serde_json::Value> {
    value
        .get("todos")
        .and_then(|t| t.as_array())
        .and_then(|todos| {
            todos
                .iter()
                .find(|t| t.get("status").and_then(|s| s.as_str()) == Some("pending"))
                .cloned()
        })
}

// ---------------------------------------------------------------------------
// Shell-out layer
// ---------------------------------------------------------------------------

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
    parse_todo_list(&out.stdout)
}

/// Return the highest-priority pending todo for a project, or None if empty.
pub fn todo_next(project: &str) -> Result<Option<serde_json::Value>> {
    let v = todo_list(project)?;
    Ok(find_next_pending(&v))
}

/// Detect project name from nearest Cargo.toml and call `todo_next`.
pub fn todo_next_for_root(root: &Path) -> Result<Option<serde_json::Value>> {
    let project = detect::package_name(root)?;
    todo_next(&project)
}
