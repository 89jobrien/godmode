//! context-map — PreToolUse/Edit hook.
//! Warns if editing src/ without a recent context map.

use std::path::Path;
use std::time::SystemTime;

/// Run the context-map hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.contains("/src/") {
        return String::new();
    }

    let working_dir = [
        root.join(".ctx/godmode/_WORKING_DIR"),
        root.join(".ctx/_WORKING_DIR"),
    ]
    .into_iter()
    .find(|path| path.exists());
    let Some(working_dir) = working_dir else {
        return "[godmode:context-map] Editing src/ without a context map — run /godmode:context-map first".to_string();
    };

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
                e.file_type()
                    .ok()
                    .filter(|kind| kind.is_file())
                    .and_then(|_| e.metadata().ok())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_non_source_edits_even_without_a_map() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(run(root.path(), "/repo/tests/case.rs"), "");
    }

    #[test]
    fn canonical_map_matrix_covers_missing_empty_current_and_stale() {
        use std::time::{Duration, SystemTime};
        let root = tempfile::tempdir().unwrap();
        let source = "/repo/src/lib.rs";
        assert!(run(root.path(), source).contains("without a context map"));
        let scratch = root.path().join(".ctx/godmode/_WORKING_DIR");
        std::fs::create_dir_all(&scratch).unwrap();
        assert!(run(root.path(), source).contains("without a recent context map"));
        let map = scratch.join("context-map-current.md");
        std::fs::write(&map, "map").unwrap();
        assert_eq!(run(root.path(), source), "");
        filetime::set_file_mtime(
            &map,
            filetime::FileTime::from_system_time(SystemTime::now() - Duration::from_secs(5 * 3600)),
        )
        .unwrap();
        assert!(run(root.path(), source).contains("without a recent context map"));
    }

    #[cfg(unix)]
    #[test]
    fn context_map_entry_metadata_failure_is_ignored() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let scratch = root.path().join(".ctx/godmode/_WORKING_DIR");
        std::fs::create_dir_all(&scratch).unwrap();
        symlink(
            scratch.join("missing"),
            scratch.join("context-map-broken.md"),
        )
        .unwrap();
        assert!(run(root.path(), "/repo/src/lib.rs").contains("without a recent context map"));
    }

    #[test]
    fn accepts_recent_legacy_context_map_scratch_file() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join(".ctx/_WORKING_DIR");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("context-map-test.md"), "map").unwrap();
        assert_eq!(run(root.path(), "/repo/src/lib.rs"), "");
    }
    #[test]
    fn scratch_read_error_degrades_to_warning() {
        let root = tempfile::tempdir().unwrap();
        let scratch = root.path().join(".ctx/godmode/_WORKING_DIR");
        std::fs::create_dir_all(scratch.parent().unwrap()).unwrap();
        std::fs::write(scratch, "not a directory").unwrap();
        assert!(run(root.path(), "/repo/src/lib.rs").contains("without a recent context map"));
    }
}
