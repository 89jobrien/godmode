//! writing-plans — PostToolUse/Write hook.
//! Appends a trace event when a plan doc is created.

use std::path::Path;

use chrono::Utc;

/// Run the writing-plans hook. Returns a message for stderr (always empty — silent hook).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.contains("docs/plans/") || !file_path.ends_with(".md") {
        return String::new();
    }

    let trace_dir = root.join(".ctx/godmode/traces");
    let _ = std::fs::create_dir_all(&trace_dir);
    let trace_file = trace_dir.join("trace.jsonl");

    let event = format!(
        r#"{{"event":"plan_created","file":"{}","ts":"{}"}}"#,
        file_path,
        Utc::now().to_rfc3339()
    );

    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_file)
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{event}")
        });

    String::new()
}
