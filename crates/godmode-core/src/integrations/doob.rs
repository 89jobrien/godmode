//! Integration with `doob` — the todo/backlog CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::detect;
use crate::model::Task;

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

// ---------------------------------------------------------------------------
// Write — pure arg builders (testable without shelling out)
// ---------------------------------------------------------------------------

/// Build argv for `doob todo complete <uuid>`.
pub fn todo_done_args(uuid: &str) -> Vec<String> {
    vec!["todo".into(), "complete".into(), uuid.into()]
}

/// Build argv for `doob todo add -p <project> <title>`.
pub fn todo_add_args(project: &str, title: &str) -> Vec<String> {
    vec![
        "todo".into(),
        "add".into(),
        "-p".into(),
        project.into(),
        title.into(),
    ]
}

/// Convert pending doob todos into `Task` values for import into the task graph.
///
/// Completed todos are skipped. The doob UUID is stored in `task.notes` as `doob:<uuid>`
/// so it can be resolved later for sync-back.
pub fn todos_to_tasks(value: &serde_json::Value) -> Vec<Task> {
    value
        .get("todos")
        .and_then(|t| t.as_array())
        .map(|todos| {
            todos
                .iter()
                .filter(|t| t.get("status").and_then(|s| s.as_str()) == Some("pending"))
                .filter_map(|t| {
                    let id = t.get("id")?.as_str()?;
                    let title = t.get("content")?.as_str()?;
                    let mut task = Task::new(format!("doob-{}", &id[..8.min(id.len())]), title);
                    task.notes = format!("doob:{id}");
                    Some(task)
                })
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Write — shell-out layer
// ---------------------------------------------------------------------------

/// Mark a doob todo as complete by UUID.
pub fn todo_done(uuid: &str) -> Result<()> {
    let args = todo_done_args(uuid);
    let out = Command::new("doob")
        .args(&args)
        .output()
        .context("doob not found on PATH")?;
    if !out.status.success() {
        bail!(
            "doob todo complete failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Add a new todo to doob for a given project.
pub fn todo_add(project: &str, title: &str) -> Result<()> {
    let args = todo_add_args(project, title);
    let out = Command::new("doob")
        .args(&args)
        .output()
        .context("doob not found on PATH")?;
    if !out.status.success() {
        bail!(
            "doob todo add failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}
