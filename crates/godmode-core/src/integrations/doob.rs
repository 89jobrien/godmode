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

/// Sync a HANDOFF YAML file into doob's handoff_item table.
///
/// Doob derives the project name from the YAML file's parent directory,
/// so we copy to a temp dir named after the project before syncing.
/// After sync, stale items (in doob but not in the YAML) are marked done.
#[instrument(name = "doob::handoff_sync", fields(integration = "doob"))]
pub fn handoff_sync(yaml_path: &Path, project: &str, current_item_ids: &[String]) -> Result<()> {
    // Copy YAML into /tmp/<project>/ so doob picks up the correct project name
    let tmp_dir = std::env::temp_dir().join(project);
    let _ = std::fs::create_dir_all(&tmp_dir);
    let tmp_file = tmp_dir.join(
        yaml_path
            .file_name()
            .context("HANDOFF YAML has no filename")?,
    );
    std::fs::copy(yaml_path, &tmp_file)?;

    let tmp_str = tmp_file
        .to_str()
        .context("temp HANDOFF path is not valid UTF-8")?;
    subprocess::run(
        "doob",
        &["handoff", "sync", "--file", tmp_str],
        "doob not found on PATH",
    )?;

    // Clean up temp file
    let _ = std::fs::remove_file(&tmp_file);
    let _ = std::fs::remove_dir(&tmp_dir);

    // Mark stale items as done — items in doob for this project that are
    // no longer in the current YAML
    cleanup_stale_items(project, current_item_ids);

    Ok(())
}

/// Query doob for handoff items in this project and mark any that are not
/// in `current_ids` as done.
fn cleanup_stale_items(project: &str, current_ids: &[String]) {
    let out = subprocess::run(
        "doob",
        &["handoff", "list", "--project", project, "--json"],
        "doob not found",
    );
    let Ok(raw) = out else { return };
    let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) else {
        return;
    };

    for item in &items {
        let Some(handoff_id) = item.get("handoff_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
        // Skip items already done/parked
        if status == "done" || status == "parked" {
            continue;
        }
        // If this item is not in the current set, mark it done
        if !current_ids.iter().any(|id| id == handoff_id) {
            let _ = subprocess::run(
                "doob",
                &["handoff", "update-status", handoff_id, "done"],
                "doob not found",
            );
        }
    }
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
