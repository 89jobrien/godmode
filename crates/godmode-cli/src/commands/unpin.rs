use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Unpin => {
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
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
