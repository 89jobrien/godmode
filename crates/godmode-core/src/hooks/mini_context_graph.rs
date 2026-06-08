//! mini-context-graph — PostToolUse/Write hook.
//! After writing markdown docs, reminds to ingest into kgx if active.

use std::path::Path;

/// Run the mini-context-graph hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    if !file_path.ends_with(".md") {
        return String::new();
    }
    if file_path.contains("/.ctx/")
        || file_path.contains("/skills/")
        || file_path.contains("/agents/")
    {
        return String::new();
    }

    let kgx_dir = root.join(".kgx");
    if !kgx_dir.exists() {
        return String::new();
    }

    let basename = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    format!(
        "[godmode:mini-context-graph] Wrote {basename} — consider ingesting into kgx: `kgx ingest`"
    )
}
