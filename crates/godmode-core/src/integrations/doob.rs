//! Integration with `doob` — the todo/backlog CLI.

use std::path::Path;

use anyhow::{Context, Result};
use tracing::instrument;

use crate::detect;
use crate::integrations::subprocess;
use crate::model::Task;

// ---------------------------------------------------------------------------
// Pure logic — testable without shelling out
// ---------------------------------------------------------------------------

/// Parse raw JSON bytes from `doob todo list --json` into a Value.
#[instrument(name = "doob::parse_todo_list", skip(raw))]
pub fn parse_todo_list(raw: &[u8]) -> Result<serde_json::Value> {
    tracing::info!(integration = "doob", "parsing todo list JSON");
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
#[instrument(name = "doob::todo_list", fields(integration = "doob"))]
pub fn todo_list(project: &str) -> Result<serde_json::Value> {
    let raw = subprocess::run(
        "doob",
        &["todo", "list", "-p", project, "--json"],
        "doob not found on PATH",
    )?;
    parse_todo_list(raw.as_bytes())
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
#[instrument(name = "doob::todo_done_args")]
pub fn todo_done_args(uuid: &str) -> Vec<String> {
    tracing::debug!(integration = "doob", %uuid, "building todo_done args");
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
#[instrument(name = "doob::todo_done", fields(integration = "doob"))]
pub fn todo_done(uuid: &str) -> Result<()> {
    let args = todo_done_args(uuid);
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    subprocess::run("doob", &args_ref, "doob not found on PATH")?;
    Ok(())
}

/// Add a new todo to doob for a given project.
#[instrument(name = "doob::todo_add", fields(integration = "doob"))]
pub fn todo_add(project: &str, title: &str) -> Result<()> {
    let args = todo_add_args(project, title);
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    subprocess::run("doob", &args_ref, "doob not found on PATH")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn parse_todo_list_emits_trace_event() {
        let raw = br#"{"todos":[]}"#;
        let _ = parse_todo_list(raw).unwrap();
        assert!(
            logs_contain("doob"),
            "expected a tracing event containing 'doob'"
        );
    }

    #[test]
    #[traced_test]
    fn todo_done_args_emits_trace_event() {
        let args = todo_done_args("abc-123");
        assert!(!args.is_empty());
        assert!(
            logs_contain("doob"),
            "expected a tracing event containing 'doob'"
        );
    }
}
