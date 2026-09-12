//! Multi-target slash-command loading, rendering, and projection.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::projection::{ProjectionFile, write_projection};

pub mod fs;
pub mod render;

/// Supported command output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandTarget {
    /// Claude Code command Markdown.
    Claude,
    /// OpenCode command Markdown.
    OpenCode,
}

/// Canonical command definition loaded from YAML.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct CommandDefinition {
    /// Stable command name.
    pub name: String,
    /// Command-palette description.
    pub description: Option<String>,
    /// Optional shared prompt template name.
    pub template: Option<String>,
    /// Command prompt body.
    pub prompt: String,
    /// Claude Code tool allowlist.
    #[serde(default, rename = "allowedTools")]
    pub allowed_tools: Vec<String>,
    /// Optional agent turn limit retained for source compatibility.
    #[serde(default, rename = "maxTurns")]
    pub max_turns: Option<u32>,
}

impl CommandDefinition {
    /// Create a command definition with required fields and empty optional settings.
    pub fn new(name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            template: None,
            prompt: prompt.into(),
            allowed_tools: Vec::new(),
            max_turns: None,
        }
    }

    /// Set the command-palette description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the shared prompt template name.
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Set the Claude Code tool allowlist.
    pub fn with_allowed_tools(mut self, allowed_tools: Vec<String>) -> Self {
        self.allowed_tools = allowed_tools;
        self
    }

    /// Set the optional agent turn limit.
    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = Some(max_turns);
        self
    }
}

/// Fully rendered command ready to write.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RenderedCommand {
    /// Destination file name.
    pub file_name: String,
    /// Complete Markdown content.
    pub content: String,
}

impl RenderedCommand {
    /// Create a rendered command from its destination name and Markdown content.
    pub fn new(file_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            content: content.into(),
        }
    }
}

/// Load sorted top-level YAML command definitions.
///
/// # Errors
///
/// Returns an error when the source directory or a definition cannot be read, parsed, or validated.
///
/// # Examples
///
/// ```
/// use godmode_core::command::load_command_definitions;
///
/// # fn main() -> anyhow::Result<()> {
/// let source = tempfile::tempdir()?;
/// std::fs::write(source.path().join("plan.yaml"), "name: plan\nprompt: Plan work\n")?;
/// let definitions = load_command_definitions(source.path())?;
/// assert_eq!(definitions[0].name, "plan");
/// # Ok(())
/// # }
/// ```
pub fn load_command_definitions(source_dir: &Path) -> Result<Vec<CommandDefinition>> {
    let mut paths = std::fs::read_dir(source_dir)
        .with_context(|| format!("reading command source dir {}", source_dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.extension().is_some_and(|ext| ext == "yaml"));
    paths.sort();

    let mut definitions = Vec::with_capacity(paths.len());
    for path in paths {
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading command definition {}", path.display()))?;
        let definition: CommandDefinition = serde_yaml::from_str(&raw)
            .with_context(|| format!("parsing command definition {}", path.display()))?;
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .with_context(|| format!("invalid command definition filename {}", path.display()))?;
        if stem != definition.name {
            bail!(
                "command definition name {:?} does not match filename {}",
                definition.name,
                path.display()
            );
        }
        definitions.push(definition);
    }
    validate_definitions(&definitions)?;
    Ok(definitions)
}

/// Render definitions deterministically for one target.
///
/// # Errors
///
/// Returns an error when a referenced template cannot be read or a definition cannot be validated or rendered.
///
/// # Examples
///
/// ```
/// use godmode_core::command::{CommandDefinition, CommandTarget, render_commands};
///
/// # fn main() -> anyhow::Result<()> {
/// let templates = tempfile::tempdir()?;
/// let definitions = [CommandDefinition::new("plan", "Plan $ARGUMENTS")];
/// let rendered = render_commands(&definitions, CommandTarget::OpenCode, templates.path())?;
/// assert_eq!(rendered[0].file_name, "gm-plan.md");
/// # Ok(())
/// # }
/// ```
pub fn render_commands(
    definitions: &[CommandDefinition],
    target: CommandTarget,
    template_dir: &Path,
) -> Result<Vec<RenderedCommand>> {
    let mut templates = std::collections::BTreeMap::new();
    for name in definitions
        .iter()
        .filter_map(|definition| definition.template.as_ref())
    {
        if templates.contains_key(name) {
            continue;
        }
        let path = template_dir.join(format!("{name}.md"));
        let template = std::fs::read_to_string(&path)
            .with_context(|| format!("reading command template {}", path.display()))?;
        templates.insert(name.clone(), template);
    }
    render_commands_with_templates(definitions, target, &templates)
}

/// Render commands using already-loaded templates and no filesystem access.
///
/// # Errors
///
/// Returns an error when definitions are invalid, a referenced template is absent, or rendering fails.
pub fn render_commands_with_templates(
    definitions: &[CommandDefinition],
    target: CommandTarget,
    templates: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<RenderedCommand>> {
    validate_definitions(definitions)?;
    let mut definitions = definitions.iter().collect::<Vec<_>>();
    definitions.sort_by(|left, right| left.name.cmp(&right.name));
    definitions
        .into_iter()
        .map(|definition| {
            let template = definition
                .template
                .as_ref()
                .map(|name| {
                    templates
                        .get(name)
                        .map(String::as_str)
                        .with_context(|| format!("missing command template {name}"))
                })
                .transpose()?;
            render_command(definition, target, template)
        })
        .collect()
}

/// Write rendered commands, or return their paths without mutation in dry-run mode.
///
/// # Errors
///
/// Returns an error when a destination name is invalid or the atomic projection transaction fails.
///
/// # Examples
///
/// ```
/// use godmode_core::command::{RenderedCommand, write_rendered_commands};
///
/// # fn main() -> anyhow::Result<()> {
/// let output = tempfile::tempdir()?;
/// let commands = [RenderedCommand::new("gm-plan.md", "# Plan\n")];
/// let paths = write_rendered_commands(&commands, output.path(), false)?;
/// assert_eq!(std::fs::read_to_string(&paths[0])?, "# Plan\n");
/// # Ok(())
/// # }
/// ```
pub fn write_rendered_commands(
    commands: &[RenderedCommand],
    output_dir: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>> {
    write_rendered_commands_with_mode(commands, output_dir, dry_run.into())
}

/// Write rendered commands according to an explicit mutation mode.
///
/// # Errors
///
/// Returns an error when a destination name is invalid or the atomic projection transaction fails.
pub fn write_rendered_commands_with_mode(
    commands: &[RenderedCommand],
    output_dir: &Path,
    mode: crate::write_mode::WriteMode,
) -> Result<Vec<PathBuf>> {
    let files = commands
        .iter()
        .map(|command| ProjectionFile::new(&command.file_name, &command.content))
        .collect::<Vec<_>>();
    write_projection(&files, output_dir, !mode.writes())
}

/// Fail when managed command output differs from the rendered set.
///
/// # Errors
///
/// Returns an error when generated output is missing, stale, unexpected, or unreadable.
///
/// # Examples
///
/// ```
/// use godmode_core::command::{
///     RenderedCommand, check_rendered_commands, write_rendered_commands,
/// };
///
/// # fn main() -> anyhow::Result<()> {
/// let output = tempfile::tempdir()?;
/// let commands = [RenderedCommand::new("gm-plan.md", "# Plan\n")];
/// write_rendered_commands(&commands, output.path(), false)?;
/// check_rendered_commands(&commands, output.path())?;
/// # Ok(())
/// # }
/// ```
pub fn check_rendered_commands(commands: &[RenderedCommand], output_dir: &Path) -> Result<()> {
    let expected = commands
        .iter()
        .map(|command| command.file_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for command in commands {
        let path = output_dir.join(&command.file_name);
        let actual = std::fs::read_to_string(&path)
            .with_context(|| format!("missing generated command {}", path.display()))?;
        if actual != command.content {
            bail!("stale generated command {}", path.display());
        }
    }
    if output_dir.exists() {
        for entry in std::fs::read_dir(output_dir)? {
            let name = entry?.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("gm-") && name.ends_with(".md") && !expected.contains(name.as_ref())
            {
                bail!(
                    "unexpected managed command {}",
                    output_dir.join(name.as_ref()).display()
                );
            }
        }
    }
    Ok(())
}

fn validate_definitions(definitions: &[CommandDefinition]) -> Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for definition in definitions {
        validate_definition(definition)?;
        if !names.insert(&definition.name) {
            bail!("duplicate command name {:?}", definition.name);
        }
    }
    Ok(())
}

fn validate_definition(definition: &CommandDefinition) -> Result<()> {
    if definition.name.is_empty()
        || definition.name.starts_with("-")
        || definition.name.ends_with("-")
        || definition.name.contains("--")
        || !definition
            .name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("unsafe command name {:?}", definition.name);
    }
    if definition.prompt.trim().is_empty() {
        bail!("command {} has an empty prompt", definition.name);
    }
    Ok(())
}

fn render_command(
    definition: &CommandDefinition,
    target: CommandTarget,
    template: Option<&str>,
) -> Result<RenderedCommand> {
    let description = definition.description.clone().unwrap_or_else(|| {
        definition
            .prompt
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or(&definition.name)
            .replace("$ARGUMENTS", "")
            .trim()
            .trim_end_matches(':')
            .trim()
            .to_string()
    });
    let quoted = serde_json::to_string(&description)?;
    let frontmatter = match target {
        CommandTarget::Claude => {
            if definition.allowed_tools.is_empty() {
                format!("---\ndescription: {quoted}\nallowed-tools: []\n---\n")
            } else {
                let tools = definition
                    .allowed_tools
                    .iter()
                    .map(|tool| format!("  - {tool}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("---\ndescription: {quoted}\nallowed-tools:\n{tools}\n---\n")
            }
        }
        CommandTarget::OpenCode => {
            format!("---\ndescription: {quoted}\nsubtask: false\n---\n")
        }
    };
    let mut body = format!(
        "{frontmatter}\n{}\n{}",
        template.unwrap_or_default(),
        definition.prompt
    );
    if target == CommandTarget::OpenCode {
        let references = regex::Regex::new(r"/gm:([a-z0-9-]+)")?;
        body = references.replace_all(&body, "/gm-$1").into_owned();
    }
    Ok(RenderedCommand::new(
        format!("gm-{}.md", definition.name),
        body,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn definition() -> CommandDefinition {
        CommandDefinition {
            name: "plan".into(),
            description: Some("Plan work".into()),
            template: None,
            prompt: "Next: /gm:ingest $ARGUMENTS".into(),
            allowed_tools: vec!["Read".into()],
            max_turns: Some(10),
        }
    }

    #[test]
    fn renders_target_specific_frontmatter_and_references() {
        let temp = TempDir::new().unwrap();
        let claude = render_commands(&[definition()], CommandTarget::Claude, temp.path()).unwrap();
        let opencode =
            render_commands(&[definition()], CommandTarget::OpenCode, temp.path()).unwrap();
        assert!(claude[0].content.contains("allowed-tools:\n  - Read"));
        assert!(claude[0].content.contains("/gm:ingest"));
        assert!(opencode[0].content.contains("subtask: false"));
        assert!(opencode[0].content.contains("/gm-ingest"));
        assert!(!opencode[0].content.contains("allowed-tools"));
    }

    #[test]
    fn command_defaults_templates_and_arguments_roundtrip() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("ops.md"), "Template $ARGUMENTS").unwrap();
        let definition = CommandDefinition {
            name: "audit".into(),
            description: None,
            template: Some("ops".into()),
            prompt: "\nAudit $ARGUMENTS:\nNext: /gm:review".into(),
            allowed_tools: vec![],
            max_turns: None,
        };

        let claude = render_commands(
            std::slice::from_ref(&definition),
            CommandTarget::Claude,
            temp.path(),
        )
        .unwrap();
        let opencode =
            render_commands(&[definition], CommandTarget::OpenCode, temp.path()).unwrap();

        assert!(claude[0].content.contains("description: \"Audit\""));
        assert!(claude[0].content.contains("allowed-tools: []"));
        assert!(claude[0].content.contains("Template $ARGUMENTS"));
        assert!(claude[0].content.contains("Audit $ARGUMENTS:"));
        assert!(opencode[0].content.contains("Template $ARGUMENTS"));
        assert!(opencode[0].content.contains("Audit $ARGUMENTS:"));
        assert!(opencode[0].content.contains("/gm-review"));
    }

    #[test]
    fn render_rejects_non_kebab_command_names() {
        let temp = TempDir::new().unwrap();
        for name in ["-plan", "plan-", "plan--now"] {
            let mut invalid = definition();
            invalid.name = name.to_string();
            assert!(render_commands(&[invalid], CommandTarget::Claude, temp.path()).is_err());
        }
    }

    #[test]
    fn load_rejects_filename_name_mismatch() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("wrong.yaml"),
            "name: plan\nprompt: Plan work\n",
        )
        .unwrap();
        assert!(load_command_definitions(temp.path()).is_err());
    }

    #[test]
    fn load_reports_missing_source_directory() {
        let temp = TempDir::new().unwrap();
        let error = load_command_definitions(&temp.path().join("missing")).unwrap_err();
        assert!(error.to_string().contains("reading command source dir"));
    }

    #[test]
    fn load_reports_malformed_yaml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("broken.yaml"), "name: [broken").unwrap();
        let error = load_command_definitions(temp.path()).unwrap_err();
        assert!(error.to_string().contains("parsing command definition"));
    }

    #[test]
    fn load_rejects_empty_prompt() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("empty.yaml"),
            "name: empty\nprompt: '  '\n",
        )
        .unwrap();
        let error = load_command_definitions(temp.path()).unwrap_err();
        assert!(error.to_string().contains("empty prompt"));
    }

    #[test]
    fn load_reports_definition_read_errors() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join("blocked.yaml")).unwrap();
        let error = load_command_definitions(temp.path()).unwrap_err();
        assert!(error.to_string().contains("reading command definition"));
    }

    #[test]
    fn render_rejects_duplicate_command_names() {
        let temp = TempDir::new().unwrap();
        assert!(
            render_commands(
                &[definition(), definition()],
                CommandTarget::Claude,
                temp.path()
            )
            .is_err()
        );
    }

    #[test]
    fn atomic_write_replaces_existing_command() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("gm-plan.md");
        std::fs::write(&path, "old").unwrap();
        let rendered =
            render_commands(&[definition()], CommandTarget::Claude, temp.path()).unwrap();
        write_rendered_commands(&rendered, temp.path(), false).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), rendered[0].content);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn dry_run_does_not_create_output_directory() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("commands");
        let rendered =
            render_commands(&[definition()], CommandTarget::OpenCode, temp.path()).unwrap();
        let paths = write_rendered_commands(&rendered, &output, true).unwrap();
        assert_eq!(paths, vec![output.join("gm-plan.md")]);
        assert!(!output.exists());
    }

    #[test]
    fn stale_check_detects_modified_output() {
        let temp = TempDir::new().unwrap();
        let rendered =
            render_commands(&[definition()], CommandTarget::Claude, temp.path()).unwrap();
        write_rendered_commands(&rendered, temp.path(), false).unwrap();
        check_rendered_commands(&rendered, temp.path()).unwrap();
        std::fs::write(temp.path().join("gm-plan.md"), "stale").unwrap();
        assert!(check_rendered_commands(&rendered, temp.path()).is_err());
    }

    #[test]
    fn stale_check_rejects_unexpected_managed_command() {
        let temp = TempDir::new().unwrap();
        let rendered =
            render_commands(&[definition()], CommandTarget::Claude, temp.path()).unwrap();
        write_rendered_commands(&rendered, temp.path(), false).unwrap();
        std::fs::write(temp.path().join("gm-unexpected.md"), "unexpected").unwrap();

        let error = check_rendered_commands(&rendered, temp.path()).unwrap_err();

        assert!(error.to_string().contains("unexpected managed command"));
        assert!(error.to_string().contains("gm-unexpected.md"));
    }

    #[test]
    fn pure_renderer_uses_supplied_templates_without_filesystem() {
        let mut templates = std::collections::BTreeMap::new();
        templates.insert("ops".to_string(), "Pure template".to_string());
        let mut command = definition();
        command.template = Some("ops".into());
        let rendered =
            render_commands_with_templates(&[command], CommandTarget::Claude, &templates).unwrap();
        assert!(rendered[0].content.contains("Pure template"));
    }

    #[test]
    fn pure_renderer_reports_missing_template() {
        let mut command = definition();
        command.template = Some("missing".into());
        let error =
            render_commands_with_templates(&[command], CommandTarget::Claude, &Default::default())
                .unwrap_err()
                .to_string();
        assert!(error.contains("missing"), "{error}");
    }

    #[test]
    fn explicit_preview_mode_does_not_write_commands() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("commands");
        let rendered =
            render_commands(&[definition()], CommandTarget::OpenCode, temp.path()).unwrap();
        write_rendered_commands_with_mode(
            &rendered,
            &output,
            crate::write_mode::WriteMode::Preview,
        )
        .unwrap();
        assert!(!output.exists());
    }

    #[test]
    fn command_write_error_names_destination() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("commands");
        std::fs::write(&output, "file").unwrap();
        let rendered =
            render_commands(&[definition()], CommandTarget::OpenCode, temp.path()).unwrap();
        let error = write_rendered_commands_with_mode(
            &rendered,
            &output,
            crate::write_mode::WriteMode::Write,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("commands"), "{error}");
    }
}
