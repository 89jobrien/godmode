//! Pure command rendering with caller-supplied templates.

use super::{CommandDefinition, CommandTarget, RenderedCommand};
use anyhow::Result;
use std::collections::BTreeMap;

/// Render commands without reading or writing the filesystem.
pub fn render(
    definitions: &[CommandDefinition],
    target: CommandTarget,
    templates: &BTreeMap<String, String>,
) -> Result<Vec<RenderedCommand>> {
    super::render_commands_with_templates(definitions, target, templates)
}
