//! Generates test scaffolds for a crate and testing dimension.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Parses the requested testing dimension and prints its generated scaffold.
pub fn handle(command: Cmd, _root: &Path, _json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Scaffold {
            crate_name,
            dimension,
        } => {
            use godmode_core::scaffold::{self, Dimension};

            let dim: Dimension = dimension.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let stub = scaffold::generate(&crate_name, dim);
            println!("{stub}");
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
