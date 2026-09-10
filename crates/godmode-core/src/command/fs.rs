//! Filesystem adapters for command sources and rendered projections.

use super::{CommandDefinition, RenderedCommand};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Load command definitions from disk.
pub fn load(source_dir: &Path) -> Result<Vec<CommandDefinition>> {
    super::load_command_definitions(source_dir)
}

/// Write rendered commands using an explicit mutation mode.
pub fn write(
    commands: &[RenderedCommand],
    output_dir: &Path,
    mode: crate::write_mode::WriteMode,
) -> Result<Vec<PathBuf>> {
    super::write_rendered_commands_with_mode(commands, output_dir, mode)
}

/// Check generated command files against expected content.
pub fn check(commands: &[RenderedCommand], output_dir: &Path) -> Result<()> {
    super::check_rendered_commands(commands, output_dir)
}
