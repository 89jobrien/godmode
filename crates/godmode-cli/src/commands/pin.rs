use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Pin { path } => {
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
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
