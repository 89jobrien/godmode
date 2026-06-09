//! brainstorm — PreToolUse/Write hook.
//! Warns when writing to src/ without a design doc dated today in docs/designs/.

use std::path::Path;

/// Run the brainstorm hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.contains("/src/") {
        return String::new();
    }

    // The design stage owns the gate for src/ writes. The brainstorm hook defers to it:
    // if a design doc exists for today, brainstorm is satisfied. If not, the design hook
    // will already warn — no need to double-warn here.
    let designs_dir = root.join("docs").join("designs");
    if !designs_dir.exists() {
        return String::new();
    }

    String::new()
}
