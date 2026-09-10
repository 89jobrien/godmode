//! Persistent insight capture and date-based Markdown report generation.
//!
//! Insights are appended as JSON Lines under `.ctx/godmode/traces` and can be
//! filtered by UTC date before being rendered into the report directory.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
/// Calendar date type used to select daily insight reports.
pub use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::report_index::{JsonFileIndex, ReportIndexPort};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A timestamped observation captured during a godmode session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Insight {
    /// Short heading used to identify the observation.
    pub title: String,
    /// Full explanatory content of the observation.
    pub body: String,
    /// Optional labels used to classify the observation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// UTC time at which the observation was captured.
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

/// Append a single insight to `.ctx/godmode/traces/insights.jsonl`.
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

/// Read all insights from `.ctx/godmode/traces/insights.jsonl`.
/// Returns an empty vec if the file does not exist.
pub fn list(root: &Path) -> Result<Vec<Insight>> {
    let canonical = insights_path(root);
    let legacy = root.join(".ctx/insights.jsonl");
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for path in [&canonical, &legacy] {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        for line in contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            match serde_json::from_str::<Insight>(line) {
                Ok(insight) => {
                    let identity = serde_json::to_string(&insight)?;
                    if seen.insert(identity) {
                        out.push(insight);
                    }
                }
                Err(error) => eprintln!("godmode: skipping malformed insight line: {error}"),
            }
        }
    }
    if legacy.exists() {
        if let Some(parent) = canonical.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let rows = out
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join(
                "
",
            );
        std::fs::write(
            &canonical,
            if rows.is_empty() {
                rows
            } else {
                format!(
                    "{rows}
"
                )
            },
        )?;
        std::fs::remove_file(legacy)?;
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

/// Render insights to `.ctx/godmode/reports/insights/insights-YYYY-MM-DD.md`.
///
/// Overwrites the report for the given date and updates the report index on a
/// best-effort basis.
pub fn render_markdown(root: &Path, date: NaiveDate) -> Result<PathBuf> {
    render_markdown_with_index(root, date, &JsonFileIndex::new(root))
}

/// Render a daily report using an injected report-index adapter.
///
/// # Errors
///
/// Returns an error when insight migration, report writing, or directory creation fails.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// use godmode_core::{insights, report_index::JsonFileIndex};
/// let root = std::path::Path::new(".");
/// let _ = insights::render_markdown_with_index(root, insights::today(), &JsonFileIndex::new(root))?;
/// # Ok(()) }
/// ```
pub fn render_markdown_with_index(
    root: &Path,
    date: NaiveDate,
    index: &dyn ReportIndexPort,
) -> Result<PathBuf> {
    let insights = list_for_date(root, date)?;
    let path = insights_md_path(root, &date);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let legacy = root.join(".ctx").join(format!("insights-{date}.md"));
    if insights.is_empty() && legacy.exists() {
        std::fs::rename(&legacy, &path).or_else(|_| {
            std::fs::copy(&legacy, &path)?;
            std::fs::remove_file(&legacy)
        })?;
    } else {
        let mut buf = format!(
            "# Insights — {}
",
            date.format("%Y-%m-%d")
        );
        for insight in &insights {
            buf.push_str(&format!(
                "
## {}

{}
",
                insight.title, insight.body
            ));
        }
        std::fs::write(&path, &buf)?;
        if legacy.exists() {
            std::fs::remove_file(legacy)?;
        }
    }
    let filename = format!("insights-{}.md", date.format("%Y-%m-%d"));
    let _ = index.add_entry("insights", &filename);
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

    #[test]
    fn list_migrates_legacy_store_without_duplicate_rows() {
        let dir = TempDir::new().unwrap();
        let legacy = dir.path().join(".ctx/insights.jsonl");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        let insight = sample_insight("Legacy");
        let row = serde_json::to_string(&insight).unwrap();
        std::fs::write(
            &legacy,
            format!(
                "{row}
{row}
"
            ),
        )
        .unwrap();

        let all = list(dir.path()).unwrap();

        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Legacy");
        assert!(insights_path(dir.path()).exists());
    }

    #[derive(Default)]
    struct RecordingIndex(std::sync::Mutex<Vec<(String, String)>>);

    impl ReportIndexPort for RecordingIndex {
        fn add_entry(&self, category: &str, filename: &str) -> Result<()> {
            self.0
                .lock()
                .unwrap()
                .push((category.to_owned(), filename.to_owned()));
            Ok(())
        }
        fn add_item(&self, _: &str, _: &str) -> Result<()> {
            Ok(())
        }
        fn rebuild(&self) -> Result<crate::report_index::ReportIndex> {
            unreachable!()
        }
        fn load(&self) -> Result<crate::report_index::ReportIndex> {
            unreachable!()
        }
    }

    #[test]
    fn render_markdown_with_index_uses_injected_port_and_migrates_legacy_report() {
        let dir = TempDir::new().unwrap();
        let date = today();
        let legacy = dir.path().join(".ctx").join(format!("insights-{date}.md"));
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(
            &legacy,
            "# legacy
",
        )
        .unwrap();
        let index = RecordingIndex::default();

        let path = render_markdown_with_index(dir.path(), date, &index).unwrap();

        assert!(path.exists());
        assert!(!legacy.exists());
        assert_eq!(index.0.lock().unwrap().len(), 1);
    }
}
