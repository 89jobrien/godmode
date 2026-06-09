//! design — PreToolUse/Write hook.
//! Warns when writing to src/ without a design doc dated today in docs/designs/.

use std::path::Path;

/// Run the design hook. Returns a warning message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.contains("/src/") {
        return String::new();
    }

    let designs_dir = root.join("docs").join("designs");
    if !designs_dir.exists() {
        return "[godmode:design] Writing src/ without a design doc — run /godmode:design first"
            .to_string();
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let has_today_doc = std::fs::read_dir(&designs_dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with(&today) && n.ends_with("-design.md"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if has_today_doc {
        String::new()
    } else {
        "[godmode:design] Writing src/ without a design doc — run /godmode:design first".to_string()
    }
}
