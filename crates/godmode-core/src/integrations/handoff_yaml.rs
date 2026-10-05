//! Native HANDOFF YAML writer — populates `.ctx/HANDOFF.<project>.<id>.yaml`
//! from godmode's task graph state.

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::model::{Status, Task};

/// A single handoff item, matching the minibox/atelier HANDOFF schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffItem {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doob_uuid: Option<String>,
    pub name: String,
    pub priority: String,
    pub status: String,
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<HandoffExtra>,
}

impl HandoffItem {
    fn running(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            doob_uuid: None,
            name: slug(&task.title),
            priority: priority_to_handoff(&task.priority),
            status: "open".into(),
            title: task.title.clone(),
            description: if task.notes.is_empty() {
                format!("Task {} in progress", task.id)
            } else {
                task.notes.clone()
            },
            files: vec![],
            completed: None,
            extra: vec![],
        }
    }

    fn pending(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            doob_uuid: None,
            name: slug(&task.title),
            priority: priority_to_handoff(&task.priority),
            status: "pending".into(),
            title: task.title.clone(),
            description: if task.notes.is_empty() {
                format!("Queued work, not yet started ({})", task.id)
            } else {
                task.notes.clone()
            },
            files: vec![],
            completed: None,
            extra: vec![],
        }
    }

    fn blocked(task: &Task) -> Self {
        let reason = if task.notes.is_empty() {
            "blocked (no reason recorded)".into()
        } else {
            task.notes.clone()
        };
        Self {
            id: task.id.clone(),
            doob_uuid: None,
            name: slug(&task.title),
            priority: priority_to_handoff(&task.priority),
            status: "blocked".into(),
            title: task.title.clone(),
            description: reason,
            files: vec![],
            completed: None,
            extra: vec![HandoffExtra {
                date: Some(Utc::now().format("%Y-%m-%d").to_string()),
                kind: Some("blocker".into()),
                note: Some(if task.notes.is_empty() {
                    "no reason recorded".into()
                } else {
                    task.notes.clone()
                }),
            }],
        }
    }
}

/// Extra metadata on a handoff item (notes, blockers, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffExtra {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A single log entry in the HANDOFF file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffLog {
    pub date: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<u32>,
}

/// The full HANDOFF YAML document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffFile {
    pub project: String,
    pub id: String,
    pub updated: NaiveDate,
    #[serde(default)]
    pub items: Vec<HandoffItem>,
    #[serde(default)]
    pub log: Vec<HandoffLog>,
}

/// Upper bound on handoff items, so a large backlog cannot turn the snapshot into a wall of rows.
///
/// Ordering in [`items_from_tasks`] puts blocked and running ahead of pending, so truncation only
/// ever discards queued work.
const MAX_HANDOFF_ITEMS: usize = 50;

/// Order two task identifiers so a numeric tail sorts numerically.
///
/// Task identifiers in a generated chain end in a plain integer, so a byte comparison puts `t10`
/// before `t6` and a snapshot listing `t10, t6, t7` misrepresents which task is next.
fn natural_order(left: &str, right: &str) -> std::cmp::Ordering {
    let mut left_parts = natural_parts(left);
    let mut right_parts = natural_parts(right);
    loop {
        match (left_parts.next(), right_parts.next()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(left_part), Some(right_part)) => match left_part.cmp(&right_part) {
                std::cmp::Ordering::Equal => {}
                other => return other,
            },
        }
    }
}

/// Splits an identifier into alternating numeric and literal runs.
///
/// A run is `(kind, number, literal)`: numeric runs carry a parsed `u64` so they compare as numbers,
/// and always sort before literal runs at the same position. Runs longer than `u64` saturate, which
/// only affects identifiers far longer than any this project mints.
fn natural_parts(value: &str) -> impl Iterator<Item = (u8, u64, String)> + '_ {
    let mut rest = value;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let start = rest;
        let numeric = start.as_bytes()[0].is_ascii_digit();
        let split = start
            .find(|character: char| character.is_ascii_digit() != numeric)
            .unwrap_or(start.len());
        let (head, tail) = start.split_at(split);
        rest = tail;
        if numeric {
            Some((0, head.parse::<u64>().unwrap_or(u64::MAX), String::new()))
        } else {
            Some((1, 0, head.to_owned()))
        }
    })
}

/// Build handoff items from task graph state.
///
/// Running and blocked are the needs-attention-now set. Pending is included too, because a task that
/// has not been started is exactly what someone picking this repository up cold cannot otherwise
/// discover — dropping it alongside `Done` made the snapshot read "No outstanding items" while a
/// multi-step chain sat queued.
///
/// Only `Done` is excluded. Items are ordered blocked, then running, then pending, and within a
/// group by priority, so the [`MAX_HANDOFF_ITEMS`] cap discards queued work before in-flight work.
pub fn items_from_tasks(tasks: &[Task]) -> Vec<HandoffItem> {
    let mut ranked: Vec<(u8, HandoffItem)> = Vec::new();

    for task in tasks {
        let (rank, item) = match task.status {
            Status::Blocked => (0, HandoffItem::blocked(task)),
            Status::Running => (1, HandoffItem::running(task)),
            Status::Pending => (2, HandoffItem::pending(task)),
            Status::Done => continue,
        };
        ranked.push((rank, item));
    }

    ranked.sort_by(|(left_rank, left), (right_rank, right)| {
        left_rank
            .cmp(right_rank)
            .then_with(|| left.priority.cmp(&right.priority))
            .then_with(|| natural_order(&left.id, &right.id))
    });
    ranked
        .into_iter()
        .map(|(_, item)| item)
        .take(MAX_HANDOFF_ITEMS)
        .collect()
}

/// Collect recent commits (since last tag or last 20).
pub fn recent_commits(root: &Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "log",
            "--oneline",
            "-20",
            "--format=%H",
        ])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        _ => vec![],
    }
}

/// Write the HANDOFF YAML file, merging with any existing content.
/// Returns `(path, item_ids)` on success.
///
/// Items are derived from task state only. Working-tree dirt is deliberately excluded: it is a
/// live query, not durable state, and embedding it in a tracked snapshot makes the file churn
/// every time the tree changes. `handoff`'s stdout still reports it.
pub fn write_handoff(
    root: &Path,
    tasks: &[Task],
    summary_text: &str,
    cfg: &Config,
) -> Result<(PathBuf, Vec<String>)> {
    let project = cfg.project_name(root);
    let ctx_dir = root.join(".ctx");
    let _ = std::fs::create_dir_all(&ctx_dir);
    let path = ctx_dir.join(format!("HANDOFF.{project}.{project}.yaml"));

    let today = Utc::now().date_naive();

    // Build items from current state
    let items = items_from_tasks(tasks);

    // Build log entry
    let commits = recent_commits(root);
    let log_entry = HandoffLog {
        date: Utc::now().format("%Y%m%d.%H%M%S").to_string(),
        summary: summary_text.to_string(),
        commits: commits.into_iter().take(cfg.handoff.max_commits).collect(),
        session: None,
    };

    // Read existing file or create new
    let base = HandoffFile {
        project: project.clone(),
        id: project.clone(),
        updated: today,
        items: vec![],
        log: vec![],
    };
    let mut handoff = if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        serde_yaml::from_str::<HandoffFile>(&raw).unwrap_or(base)
    } else {
        base
    };

    // Replace items (current state is source of truth)
    handoff.items = items;
    handoff.updated = today;

    // Deduplicate: skip if the most recent log entry has the same summary and
    // was written on the same calendar day (first 8 chars of the date stamp).
    let today_prefix = Utc::now().format("%Y%m%d").to_string();
    let is_duplicate = handoff.log.first().is_some_and(|last| {
        last.summary == log_entry.summary && last.date.starts_with(&today_prefix)
    });
    if !is_duplicate {
        handoff.log.insert(0, log_entry);
    }

    let item_ids: Vec<String> = handoff.items.iter().map(|i| i.id.clone()).collect();
    let yaml = serde_yaml::to_string(&handoff)?;
    std::fs::write(&path, yaml)?;

    // Render HANDOFF.md from the YAML data
    let _ = write_handoff_md(root, &handoff);

    Ok((path, item_ids))
}

/// Render `HANDOFF.md` from the handoff file as a human-readable summary.
///
/// The file is written to the repository root so it is tracked alongside the code it describes.
/// Unlike the YAML record it is derived from, it is a point-in-time snapshot for a human or an
/// agent picking the repo up cold — it is safe to regenerate at any time.
pub fn write_handoff_md(root: &Path, handoff: &HandoffFile) -> Result<()> {
    let path = root.join("HANDOFF.md");
    let mut md = format!("# Handoff — {} ({})\n\n", handoff.project, handoff.updated);

    // Items table
    if handoff.items.is_empty() {
        md.push_str("No outstanding items.\n");
    } else {
        md.push_str("| ID | P | Status | Title |\n|---|---|---|---|\n");
        for item in &handoff.items {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                item.id, item.priority, item.status, item.title
            ));
        }
    }

    // Recent log
    if !handoff.log.is_empty() {
        md.push_str("\n## Log\n\n");
        for entry in handoff.log.iter().take(5) {
            let commits = if entry.commits.is_empty() {
                String::new()
            } else {
                format!(
                    " [{}]",
                    entry
                        .commits
                        .iter()
                        .map(|c| &c[..7.min(c.len())])
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            md.push_str(&format!("- {}: {}{}\n", entry.date, entry.summary, commits));
        }
    }

    std::fs::write(&path, md)?;
    Ok(())
}

fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn priority_to_handoff(p: &crate::model::Priority) -> String {
    match p {
        crate::model::Priority::High => "P1".into(),
        crate::model::Priority::Normal => "P2".into(),
        crate::model::Priority::Low => "P3".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Priority, Task};

    #[test]
    fn items_from_running_task() {
        let mut t = Task::new("t1", "Fix the bug");
        t.status = Status::Running;
        t.priority = Priority::High;
        let items = items_from_tasks(&[t]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "open");
        assert_eq!(items[0].priority, "P1");
    }

    #[test]
    fn items_from_blocked_task() {
        let mut t = Task::new("t2", "Blocked thing");
        t.status = Status::Blocked;
        t.notes = "waiting on upstream".into();
        let items = items_from_tasks(&[t]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "blocked");
        assert_eq!(items[0].extra.len(), 1);
        assert_eq!(
            items[0].extra[0].note.as_deref(),
            Some("waiting on upstream")
        );
    }

    #[test]
    fn skips_completed_tasks() {
        let mut t1 = Task::new("t1", "Done");
        t1.status = Status::Done;
        let mut t2 = Task::new("t2", "Also done");
        t2.status = Status::Done;
        assert!(items_from_tasks(&[t1, t2]).is_empty());
    }

    #[test]
    fn numeric_task_ids_sort_naturally() {
        let mut ids = vec![
            "t10".to_owned(),
            "t6".to_owned(),
            "t2".to_owned(),
            "t10a".to_owned(),
        ];
        ids.sort_by(|left, right| natural_order(left, right));
        assert_eq!(ids, vec!["t2", "t6", "t10", "t10a"]);
    }

    #[test]
    fn slug_generation() {
        assert_eq!(slug("Fix the Bug"), "fix-the-bug");
        assert_eq!(slug("  spaces  "), "spaces");
    }

    #[test]
    fn write_handoff_deduplicates_same_summary_same_day() {
        use crate::config::Config;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();
        // init git so recent_commits doesn't fail
        let _ = std::process::Command::new("git")
            .args(["init", root.to_str().unwrap()])
            .output();

        let cfg = Config::default();
        let summary = "done=1 running=0 pending=0 blocked=0";

        write_handoff(root, &[], summary, &cfg).unwrap();
        write_handoff(root, &[], summary, &cfg).unwrap();

        let path = root
            .join(".ctx")
            .join(format!("HANDOFF.{p}.{p}.yaml", p = cfg.project_name(root)));
        let raw = std::fs::read_to_string(&path).unwrap();
        let hf: HandoffFile = serde_yaml::from_str(&raw).unwrap();
        assert_eq!(
            hf.log.len(),
            1,
            "duplicate same-day entries should be suppressed"
        );
    }

    #[test]
    fn handoff_file_roundtrips() {
        let hf = HandoffFile {
            project: "test".into(),
            id: "test".into(),
            updated: Utc::now().date_naive(),
            items: vec![],
            log: vec![],
        };
        let yaml = serde_yaml::to_string(&hf).unwrap();
        let back: HandoffFile = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back.project, "test");
    }

    #[test]
    fn handoff_md_is_written_to_the_repository_root() {
        use crate::config::Config;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let _ = std::process::Command::new("git")
            .args(["init", root.to_str().unwrap()])
            .output();

        let cfg = Config::default();
        let summary = "done=0 running=0 pending=0 blocked=0";
        write_handoff(root, &[], summary, &cfg).unwrap();

        // The tracked snapshot lives beside the code, not under .ctx/.
        assert!(
            root.join("HANDOFF.md").exists(),
            "HANDOFF.md must be written to the repository root so it can be tracked"
        );
        assert!(
            !root.join(".ctx").join("HANDOFF.md").exists(),
            "HANDOFF.md must not also be written under .ctx/"
        );

        let md = std::fs::read_to_string(root.join("HANDOFF.md")).unwrap();
        assert!(md.starts_with("# Handoff — "), "unexpected header: {md}");
    }

    #[test]
    fn pending_tasks_are_recorded_but_completed_ones_are_not() {
        let mut pending_task = Task::new("t1", "Queued work");
        pending_task.status = Status::Pending;
        let mut done_task = Task::new("t2", "Finished work");
        done_task.status = Status::Done;
        let mut running_task = Task::new("t0", "In flight");
        running_task.status = Status::Running;

        let items = items_from_tasks(&[done_task, pending_task, running_task]);

        let ids = items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec!["t0", "t1"],
            "blocked/running order ahead of pending"
        );
        assert!(
            !ids.contains(&"t2"),
            "completed work must not be an outstanding item"
        );
    }

    #[test]
    fn item_cap_discards_queued_work_before_in_flight_work() {
        let mut tasks = Vec::new();
        for index in 0..MAX_HANDOFF_ITEMS + 10 {
            let mut task = Task::new(format!("pending-{index}"), "Queued");
            task.status = Status::Pending;
            tasks.push(task);
        }
        let mut running = Task::new("running-1", "In flight");
        running.status = Status::Running;
        tasks.push(running);

        let items = items_from_tasks(&tasks);

        assert_eq!(items.len(), MAX_HANDOFF_ITEMS);
        assert_eq!(
            items[0].id, "running-1",
            "the in-flight task must survive the cap"
        );
    }

    #[test]
    fn working_tree_state_is_not_recorded_as_a_handoff_item() {
        use crate::config::Config;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let _ = std::process::Command::new("git")
            .args(["init", root.to_str().unwrap()])
            .output();

        let cfg = Config::default();
        let summary = "done=0 running=0 pending=1 blocked=0";
        let mut task = Task::new("t1", "Pending work");
        task.status = Status::Pending;
        write_handoff(root, &[task], summary, &cfg).unwrap();

        let path = root
            .join(".ctx")
            .join(format!("HANDOFF.{p}.{p}.yaml", p = cfg.project_name(root)));
        let raw = std::fs::read_to_string(&path).unwrap();
        let hf: HandoffFile = serde_yaml::from_str(&raw).unwrap();

        assert!(
            !hf.items.iter().any(|i| i.id == "uncommitted-work"),
            "working-tree dirt is a live query and must not be persisted as a durable handoff item"
        );
        assert!(
            !std::fs::read_to_string(root.join("HANDOFF.md"))
                .unwrap()
                .contains("Uncommitted changes"),
            "HANDOFF.md must not embed a live uncommitted-file count"
        );
    }
}
