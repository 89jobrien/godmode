use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
pub use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::report_index::{JsonFileIndex, ReportIndexPort};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Insight {
    pub title: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub ts: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// File path helpers
// ---------------------------------------------------------------------------

fn insights_path(root: &Path) -> PathBuf {
    root.join(".ctx")
        .join("godmode")
        .join("traces")
        .join("insights.jsonl")
}

fn insights_md_path(root: &Path, date: &NaiveDate) -> PathBuf {
    root.join(".ctx")
        .join("godmode")
        .join("reports")
        .join("insights")
        .join(format!("insights-{}.md", date.format("%Y-%m-%d")))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return today's date (UTC).
pub fn today() -> NaiveDate {
    Utc::now().date_naive()
}

/// Create a new `Insight` with the current timestamp.
pub fn new_insight(title: String, body: String, tags: Vec<String>) -> Insight {
    Insight {
        title,
        body,
        tags,
        ts: Utc::now(),
    }
}

/// Append a single insight to `.ctx/insights.jsonl`. Non-fatal on I/O errors
/// when called from hooks; callers that need error reporting should use `?`.
pub fn append(root: &Path, insight: &Insight) -> Result<()> {
    let path = insights_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(f, "{}", serde_json::to_string(insight)?)?;
    Ok(())
}

/// Read all insights from `.ctx/insights.jsonl`.
/// Returns an empty vec if the file does not exist.
pub fn list(root: &Path) -> Result<Vec<Insight>> {
    let path = insights_path(root);
    if !path.exists() {
        return Ok(vec![]);
    }
    let contents = std::fs::read_to_string(&path)?;
    let mut out = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<Insight>(line) {
            Ok(i) => out.push(i),
            Err(e) => eprintln!("godmode: skipping malformed insight line: {e}"),
        }
    }
    Ok(out)
}

/// Filter insights to a single date (UTC).
pub fn list_for_date(root: &Path, date: NaiveDate) -> Result<Vec<Insight>> {
    let all = list(root)?;
    Ok(all
        .into_iter()
        .filter(|i| i.ts.date_naive() == date)
        .collect())
}

/// Render insights to the `.ctx/insights-YYYY-MM-DD.md` markdown format.
/// Overwrites the file for the given date.
pub fn render_markdown(root: &Path, date: NaiveDate) -> Result<PathBuf> {
    let insights = list_for_date(root, date)?;
    let path = insights_md_path(root, &date);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut buf = format!("# Insights — {}\n", date.format("%Y-%m-%d"));
    for insight in &insights {
        buf.push_str(&format!("\n## {}\n\n{}\n", insight.title, insight.body));
    }
    std::fs::write(&path, &buf)?;

    // Update the report index (non-fatal — don't block render on index failure)
    let filename = format!("insights-{}.md", date.format("%Y-%m-%d"));
    let _ = JsonFileIndex::new(root).add_entry("insights", &filename);

    Ok(path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_insight(title: &str) -> Insight {
        Insight {
            title: title.to_string(),
            body: "Some educational content.".to_string(),
            tags: vec![],
            ts: Utc::now(),
        }
    }

    #[test]
    fn append_and_list_roundtrips() {
        let dir = TempDir::new().unwrap();
        let a = sample_insight("First");
        let b = sample_insight("Second");
        append(dir.path(), &a).unwrap();
        append(dir.path(), &b).unwrap();

        let all = list(dir.path()).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].title, "First");
        assert_eq!(all[1].title, "Second");
    }

    #[test]
    fn list_returns_empty_when_no_file() {
        let dir = TempDir::new().unwrap();
        let all = list(dir.path()).unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn list_for_date_filters_correctly() {
        let dir = TempDir::new().unwrap();
        let today = Utc::now().date_naive();
        let mut old = sample_insight("Old");
        old.ts = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        append(dir.path(), &old).unwrap();
        append(dir.path(), &sample_insight("Today")).unwrap();

        let filtered = list_for_date(dir.path(), today).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Today");
    }

    #[test]
    fn render_markdown_writes_file() {
        let dir = TempDir::new().unwrap();
        let today = Utc::now().date_naive();
        append(dir.path(), &sample_insight("Alpha")).unwrap();
        append(dir.path(), &sample_insight("Beta")).unwrap();

        let path = render_markdown(dir.path(), today).unwrap();
        assert!(path.exists());
        let md = std::fs::read_to_string(&path).unwrap();
        assert!(md.contains("# Insights"));
        assert!(md.contains("## Alpha"));
        assert!(md.contains("## Beta"));
    }

    #[test]
    fn append_with_tags_roundtrips() {
        let dir = TempDir::new().unwrap();
        let mut i = sample_insight("Tagged");
        i.tags = vec!["rust".to_string(), "testing".to_string()];
        append(dir.path(), &i).unwrap();

        let all = list(dir.path()).unwrap();
        assert_eq!(all[0].tags, vec!["rust", "testing"]);
    }

    #[test]
    fn malformed_lines_are_skipped() {
        let dir = TempDir::new().unwrap();
        let path = insights_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not valid json\n").unwrap();
        append(dir.path(), &sample_insight("Good")).unwrap();

        let all = list(dir.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Good");
    }
}
