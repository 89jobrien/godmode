//! Filesystem adapter for the trace log.
//!
//! This is the only module in the trace stack that touches the disk. It knows
//! *where* the log lives and how to append and read lines, and nothing about
//! the event vocabulary -- that belongs to [`crate::hooks::trace_log`], which
//! calls into here.
//!
//! [`trace_query`](crate::trace_query) is deliberately kept pure so the query
//! layer can be exercised with literal input and no temporary directories. The
//! intended flow is: an adapter reads, the domain queries, the caller wires the
//! two together.
//!
//! There is no `TraceStore` trait, because there is exactly one store. If a
//! second backend appears -- reading a log shipped from another machine, or
//! streaming from stdin -- that is when this module should be split behind a
//! port rather than the call sites growing conditionals.

use std::io::Write;
use std::path::{Path, PathBuf};

/// Log location relative to a repository root.
pub const TRACE_RELATIVE: &str = ".ctx/godmode/traces/trace.jsonl";

/// Absolute path of the log for a repository root.
pub fn trace_path(root: &Path) -> PathBuf {
    root.join(TRACE_RELATIVE)
}

/// Repository root of the current working directory, if there is one.
///
/// Readers and writers must agree on which log they mean, so both go through
/// here rather than each shelling out to git.
pub fn discover_root() -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!root.is_empty()).then(|| PathBuf::from(root))
}

/// Append one pre-serialised JSON line. Best-effort: never returns an error,
/// because tracing must not be able to fail its caller.
pub fn append_line(root: &Path, line: &str) {
    let path = trace_path(root);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(format!("{line}\n").as_bytes()));
}

/// Read the raw log body, or `None` when there is no log yet.
pub fn read_body(root: &Path) -> Option<String> {
    std::fs::read_to_string(trace_path(root)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::TempDir;

    #[test]
    fn trace_path_is_rooted_at_the_godmode_context_dir() {
        let p = trace_path(Path::new("/repo"));
        assert!(p.ends_with(".ctx/godmode/traces/trace.jsonl"));
    }

    #[test]
    fn append_line_creates_missing_directories() {
        let dir = TempDir::new().unwrap();
        assert!(!trace_path(dir.path()).exists());

        append_line(dir.path(), r#"{"a":1}"#);
        assert!(trace_path(dir.path()).exists());
    }

    #[test]
    fn append_line_appends_rather_than_truncates() {
        let dir = TempDir::new().unwrap();
        append_line(dir.path(), r#"{"n":1}"#);
        append_line(dir.path(), r#"{"n":2}"#);

        let body = read_body(dir.path()).unwrap();
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 2);
        let last: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(last.get("n").and_then(Value::as_i64), Some(2));
    }

    #[test]
    fn every_appended_line_is_valid_json() {
        // A torn line would poison every downstream reader, so serialisation is
        // done by the caller and only the newline handling lives here.
        let dir = TempDir::new().unwrap();
        append_line(dir.path(), r#"{"x":"has \"quotes\" and, commas"}"#);
        let body = read_body(dir.path()).unwrap();
        serde_json::from_str::<Value>(body.lines().next().unwrap()).unwrap();
    }

    #[test]
    fn read_body_is_none_when_no_log_exists() {
        let dir = TempDir::new().unwrap();
        assert!(read_body(dir.path()).is_none());
    }

    #[test]
    fn append_line_against_an_unusable_path_does_not_panic() {
        // Tracing is best-effort: a bad root must not take down the caller.
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("not-a-dir");
        std::fs::write(&file, b"x").unwrap();
        append_line(&file, r#"{"a":1}"#);
    }

    #[test]
    fn discover_root_finds_the_enclosing_repo() {
        // The crate lives inside the godmode workspace, so this must resolve.
        assert!(discover_root().is_some());
    }
}
