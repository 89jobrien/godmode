//! Test module stub generation.

use anyhow::Result;
use godmode_core::scaffold::{self, Dimension};

pub fn run_scaffold(crate_name: String, dimension: String) -> Result<()> {
    let dim: Dimension = dimension.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    let stub = scaffold::generate(&crate_name, dim);
    println!("{stub}");
    Ok(())
}
