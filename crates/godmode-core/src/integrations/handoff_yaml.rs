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

/// Build handoff items from task graph state.
pub fn items_from_tasks(tasks: &[Task]) -> Vec<HandoffItem> {
    let mut items = Vec::new();

    for t in tasks {
        match t.status {
            Status::Running => items.push(HandoffItem::running(t)),
            Status::Blocked => items.push(HandoffItem::blocked(t)),
            Status::Pending | Status::Done => {}
        }
    }
    items
}

/// Build a dirty-files handoff item if there are uncommitted changes.
pub fn dirty_files_item(dirty: &[String]) -> Option<HandoffItem> {
    if dirty.is_empty() {
        return None;
    }
    let file_list: Vec<String> = dirty
        .iter()
        .map(|l| {
            // git status --porcelain lines start with 2-char status prefix
            l.get(3..).unwrap_or(l).to_string()
        })
        .collect();
    Some(HandoffItem {
        id: "uncommitted-work".into(),
        doob_uuid: None,
        name: "uncommitted-work".into(),
        priority: "P1".into(),
        status: "open".into(),
        title: format!(
            "Uncommitted changes ({} file{})",
            dirty.len(),
            if dirty.len() == 1 { "" } else { "s" }
        ),
        description: format!(
            "Working tree has uncommitted changes:\n{}",
            file_list.join("\n")
        ),
        files: file_list,
        completed: None,
        extra: vec![],
    })
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
pub fn write_handoff(
    root: &Path,
    tasks: &[Task],
    dirty_files: &[String],
    summary_text: &str,
    cfg: &Config,
) -> Result<(PathBuf, Vec<String>)> {
    let project = cfg.project_name(root);
    let ctx_dir = root.join(".ctx");
    let _ = std::fs::create_dir_all(&ctx_dir);
    let path = ctx_dir.join(format!("HANDOFF.{project}.{project}.yaml"));

    let today = Utc::now().date_naive();

    // Build items from current state
    let mut items = items_from_tasks(tasks);
    if let Some(dirty_item) = dirty_files_item(dirty_files) {
        items.push(dirty_item);
    }

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
pub fn write_handoff_md(root: &Path, handoff: &HandoffFile) -> Result<()> {
    let path = root.join(".ctx").join("HANDOFF.md");
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
    fn skips_done_and_pending() {
        let t1 = Task::new("t1", "Done");
        let mut t2 = Task::new("t2", "Also done");
        t2.status = Status::Done;
        let items = items_from_tasks(&[t1, t2]);
        assert!(items.is_empty());
    }

    #[test]
    fn dirty_files_item_none_when_empty() {
        assert!(dirty_files_item(&[]).is_none());
    }

    #[test]
    fn dirty_files_item_builds() {
        let dirty = vec!["M  src/lib.rs".into(), "?? new.txt".into()];
        let item = dirty_files_item(&dirty).unwrap();
        assert_eq!(item.id, "uncommitted-work");
        assert_eq!(item.files.len(), 2);
        assert_eq!(item.files[0], "src/lib.rs");
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

        write_handoff(root, &[], &[], summary, &cfg).unwrap();
        write_handoff(root, &[], &[], summary, &cfg).unwrap();

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
}
