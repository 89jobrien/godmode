//! Session root pinning.

use anyhow::Result;
use godmode_core::detect;
use std::path::Path;

pub fn run_pin(root: &Path, json: bool, path: Option<String>) -> Result<()> {
    let target = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir()?,
    };
    detect::pin_root(root, &target)?;
    let canonical = target.canonicalize()?;
    if json {
        println!(
            "{}",
            serde_json::json!({"pinned_root": canonical.to_string_lossy()})
        );
    } else {
        println!("Pinned to {}", canonical.display());
    }
    Ok(())
}

pub fn run_unpin(root: &Path, json: bool) -> Result<()> {
    let removed = detect::unpin_root(root)?;
    if json {
        println!("{}", serde_json::json!({"unpinned": removed}));
    } else if removed {
        println!("Unpinned.");
    } else {
        println!("No pin was set.");
    }
    Ok(())
}
