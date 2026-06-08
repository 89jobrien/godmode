//! todo-issue-sync — PostToolUse/Bash hook.
//! After `gh issue create`, extracts the issue URL and adds it as a task.

use std::path::Path;

use serde_json::Value;

use crate::graph;
use crate::model::{Status, Task};

/// Run the todo-issue-sync hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str, tool_response: &Value) -> String {
    if !command.contains("gh issue create") {
        return String::new();
    }

    // Extract stdout from tool_response
    let stdout = tool_response
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Find URL in output
    let url = stdout
        .lines()
        .find(|l| l.starts_with("https://"))
        .unwrap_or("");

    if url.is_empty() {
        return "[godmode:todo-issue-sync] Issue created — run `godmode task pull --github` to sync.".to_string();
    }

    // Extract issue number from URL (last path segment)
    let issue_num = url.split('/').next_back().unwrap_or("0").trim();
    let task_id = format!("gh-{issue_num}");

    // Extract title from --title flag in command
    let title = extract_title(command).unwrap_or_else(|| format!("GitHub issue #{issue_num}"));

    // Add to task graph directly
    let mut g = match graph::load(root) {
        Ok(g) => g,
        Err(_) => {
            return format!("[godmode:todo-issue-sync] Issue created: {url}");
        }
    };

    let task = Task {
        id: task_id.clone(),
        title,
        status: Status::Pending,
        depends_on: vec![],
        notes: url.to_string(),
        crate_name: None,
        commit: None,
        completed: None,
        run: None,
        started_at: None,
        completed_at: None,
        priority: Default::default(),
        tags: vec![],
    };

    // Skip if already exists
    if g.tasks.iter().any(|t| t.id == task_id) {
        return format!(
            "[godmode:todo-issue-sync] Issue created: {url} (task {task_id} already exists)"
        );
    }

    g.tasks.push(task);
    if graph::save(root, &g).is_ok() {
        format!(
            "[godmode:todo-issue-sync] Issue created: {url}\n[godmode:todo-issue-sync] Task {task_id} added to graph."
        )
    } else {
        format!("[godmode:todo-issue-sync] Issue created: {url}")
    }
}

fn extract_title(cmd: &str) -> Option<String> {
    // Match --title "..." or --title '...'
    let patterns = ["--title \"", "--title '"];
    for pat in patterns {
        if let Some(start) = cmd.find(pat) {
            let rest = &cmd[start + pat.len()..];
            let delim = if pat.ends_with('"') { '"' } else { '\'' };
            if let Some(end) = rest.find(delim) {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}
