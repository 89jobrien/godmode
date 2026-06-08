//! memory-banking — SessionStart hook.
//! Prints memory-bank contents if .ctx/godmode/memory-bank/ exists.

use std::path::Path;

/// Run the memory-banking hook. Returns content for stdout (may be empty).
pub fn run(root: &Path) -> String {
    let mb_dir = root.join(".ctx/godmode/memory-bank");
    if !mb_dir.exists() {
        return String::new();
    }

    // Read and concatenate all markdown files in memory-bank/
    let mut output = String::new();
    let entries = match std::fs::read_dir(&mb_dir) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .collect();

    files.sort_by_key(|e| e.file_name());

    for entry in files {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&content);
        }
    }

    output
}
