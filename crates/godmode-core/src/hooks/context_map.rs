//! context-map — PreToolUse/Edit hook.
//! Warns if editing src/ without a recent context map.

use std::path::Path;
use std::time::SystemTime;

/// Run the context-map hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.contains("/src/") {
        return String::new();
    }

    let working_dir = root.join(".ctx/_WORKING_DIR");
    if !working_dir.exists() {
        return "[godmode:context-map] Editing src/ without a context map — run /godmode:context-map first".to_string();
    }

    // Check for any context-map file modified in the last 4 hours
    let four_hours_ago = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(4 * 3600))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let has_recent_map = std::fs::read_dir(&working_dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name();
                let name_str = name.to_str().unwrap_or("");
                if !name_str.contains("context-map") {
                    return false;
                }
                e.metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| t > four_hours_ago)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if has_recent_map {
        String::new()
    } else {
        "[godmode:context-map] Editing src/ without a recent context map — run /godmode:context-map first".to_string()
    }
}
