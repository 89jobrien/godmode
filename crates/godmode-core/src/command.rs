//! Multi-target slash-command loading, rendering, and projection.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::io::Write;
use std::path::{Path, PathBuf};

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

/// Fully rendered command ready to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedCommand {
    /// Destination file name.
    pub file_name: String,
    /// Complete Markdown content.
    pub content: String,
}

/// Load sorted top-level YAML command definitions.
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
pub fn render_commands(
    definitions: &[CommandDefinition],
    target: CommandTarget,
    template_dir: &Path,
) -> Result<Vec<RenderedCommand>> {
    validate_definitions(definitions)?;
    let mut definitions = definitions.iter().collect::<Vec<_>>();
    definitions.sort_by(|left, right| left.name.cmp(&right.name));
    definitions
        .into_iter()
        .map(|definition| render_command(definition, target, template_dir))
        .collect()
}

/// Write rendered commands, or return their paths without mutation in dry-run mode.
pub fn write_rendered_commands(
    commands: &[RenderedCommand],
    output_dir: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>> {
    if !dry_run {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("creating command output dir {}", output_dir.display()))?;
    }
    commands
        .iter()
        .map(|command| {
            let path = output_dir.join(&command.file_name);
            if !dry_run {
                write_atomic(&path, command.content.as_bytes())?;
            }
            Ok(path)
        })
        .collect()
}

/// Fail when managed command output differs from the rendered set.
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

fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("command path has no parent: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid command output filename {}", path.display()))?;

    for attempt in 0..100_u32 {
        let temporary = parent.join(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            attempt
        ));
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("creating temporary command {}", temporary.display())
                });
            }
        };

        let result = (|| -> Result<()> {
            file.write_all(content)
                .with_context(|| format!("writing temporary command {}", temporary.display()))?;
            file.sync_all()
                .with_context(|| format!("syncing temporary command {}", temporary.display()))?;
            drop(file);
            std::fs::rename(&temporary, path).with_context(|| {
                format!(
                    "atomically replacing command {} with {}",
                    path.display(),
                    temporary.display()
                )
            })?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        return result;
    }

    bail!(
        "unable to allocate temporary command file for {}",
        path.display()
    )
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
    template_dir: &Path,
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
    let template = if let Some(name) = &definition.template {
        let path = template_dir.join(format!("{name}.md"));
        std::fs::read_to_string(&path)
            .with_context(|| format!("reading command template {}", path.display()))?
    } else {
        String::new()
    };
    let mut body = format!("{frontmatter}\n{template}\n{}", definition.prompt);
    if target == CommandTarget::OpenCode {
        let references = regex::Regex::new(r"/gm:([a-z0-9-]+)")?;
        body = references.replace_all(&body, "/gm-$1").into_owned();
    }
    Ok(RenderedCommand {
        file_name: format!("gm-{}.md", definition.name),
        content: body,
    })
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
}
