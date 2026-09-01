//! GitHub issue import: fetch open issues and convert to task graph entries.

use anyhow::{Context, Result, bail};

use crate::integrations::subprocess;
use crate::model::Task;

/// Parse raw JSON bytes from `gh issue list --json number,title,body,labels`.
pub fn parse_issue_list(raw: &[u8]) -> Result<serde_json::Value> {
    serde_json::from_slice(raw).context("gh issue list: invalid JSON")
}

/// Convert a parsed issue list JSON array into `Task` values for the task graph.
pub fn issues_to_tasks(value: &serde_json::Value, label: Option<&str>) -> Vec<Task> {
    let issues = match value.as_array() {
        Some(arr) => arr,
        None => return vec![],
    };
    issues
        .iter()
        .filter(|issue| {
            if let Some(filter) = label {
                issue
                    .get("labels")
                    .and_then(|ls| ls.as_array())
                    .map(|ls| {
                        ls.iter()
                            .any(|l| l.get("name").and_then(|n| n.as_str()) == Some(filter))
                    })
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .filter_map(|issue| {
            let number = issue.get("number")?.as_u64()?;
            let title = issue.get("title")?.as_str()?;
            let id = format!("gh-{}", number);
            let mut task = Task::new(id, title);
            if let Some(body) = issue.get("body").and_then(|b| b.as_str())
                && !body.is_empty()
            {
                task.notes = body.to_string();
            }
            Some(task)
        })
        .collect()
}

/// Fetch open issues via `gh`. Degrades gracefully if `gh` is not on PATH.
pub fn pull_issues(repo: Option<&str>, label: Option<&str>) -> Result<Vec<Task>> {
    let mut args = vec![
        "issue",
        "list",
        "--state",
        "open",
        "--json",
        "number,title,body,labels",
    ];
    let repo_flag;
    if let Some(r) = repo {
        repo_flag = r.to_string();
        args.push("--repo");
        args.push(&repo_flag);
    }
    let label_flag;
    if let Some(l) = label {
        label_flag = l.to_string();
        args.push("--label");
        args.push(&label_flag);
    }
    let raw = subprocess::run("gh", &args, "install the GitHub CLI to use --github")?;
    let value = parse_issue_list(raw.as_bytes())?;
    Ok(issues_to_tasks(&value, label))
}

/// Close an issue through `gh` and add a comment identifying the implementing commit.
pub fn issue_close(number: u64, repo: Option<&str>, commit_sha: &str) -> Result<()> {
    let number_str = number.to_string();
    let comment = format!("Implemented in {}.", commit_sha);
    let mut args = vec!["issue", "close", &number_str];
    let repo_flag;
    if let Some(r) = repo {
        repo_flag = r.to_string();
        args.push("--repo");
        args.push(&repo_flag);
    }
    args.push("--comment");
    args.push(&comment);
    // issue close doesn't produce meaningful stdout; use raw Command for status check
    let out = std::process::Command::new("gh")
        .args(&args)
        .output()
        .context("gh issue close failed")?;
    if !out.status.success() {
        bail!("gh issue close exited with {}", out.status);
    }
    Ok(())
}
