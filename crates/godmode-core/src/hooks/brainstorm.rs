//! brainstorm — PreToolUse/Write hook.
//! Warns when writing to src/ without a design doc dated today.

use std::path::Path;

/// Run the brainstorm hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.contains("/src/") {
        return String::new();
    }

    let plans_dir = root.join("docs").join("plans");
    if !plans_dir.exists() {
        return "[godmode:brainstorm] Writing src/ without a design doc — run /godmode:brainstorm first".to_string();
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let has_today_doc = std::fs::read_dir(&plans_dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.contains(&today))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if has_today_doc {
        String::new()
    } else {
        "[godmode:brainstorm] Writing src/ without a design doc — run /godmode:brainstorm first"
            .to_string()
    }
}
