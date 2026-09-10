//! Command generation and installation adapter.

use std::path::Path;

use anyhow::Result;
use godmode_core::command as core_command;

use crate::{CommandAction, CommandTargetArg};

pub(super) fn run(root: &Path, json: bool, action: CommandAction) -> Result<()> {
    let (source_dir, target, output_dir, check, dry_run) = match action {
        CommandAction::Generate {
            target,
            source_dir,
            output_dir,
            check,
        } => {
            let output = match (target, output_dir) {
                (CommandTargetArg::Claude, Some(path)) => path,
                (CommandTargetArg::Claude, None) => root.join("commands"),
                (CommandTargetArg::OpenCode, Some(path)) => path,
                (CommandTargetArg::OpenCode, None) => {
                    anyhow::bail!(
                        "--output-dir is required for OpenCode generation; use command install-opencode for the default global destination"
                    );
                }
            };
            (
                source_dir.unwrap_or_else(|| root.join("command-support/gm")),
                match target {
                    CommandTargetArg::Claude => core_command::CommandTarget::Claude,
                    CommandTargetArg::OpenCode => core_command::CommandTarget::OpenCode,
                },
                output,
                check,
                false,
            )
        }
        CommandAction::InstallOpenCode {
            source_dir,
            output_dir,
            dry_run,
        } => {
            let output = output_dir.map_or_else(default_opencode_command_dir, Ok)?;
            (
                source_dir.unwrap_or_else(|| root.join("command-support/gm")),
                core_command::CommandTarget::OpenCode,
                output,
                false,
                dry_run,
            )
        }
    };

    let definitions = core_command::load_command_definitions(&source_dir)?;
    let rendered =
        core_command::render_commands(&definitions, target, &source_dir.join("templates"))?;
    let paths = if check {
        core_command::check_rendered_commands(&rendered, &output_dir)?;
        rendered
            .iter()
            .map(|item| output_dir.join(&item.file_name))
            .collect::<Vec<_>>()
    } else {
        core_command::write_rendered_commands(&rendered, &output_dir, dry_run)?
    };
    render_result(json, target, &output_dir, check, dry_run, paths)
}

fn default_opencode_command_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
    Ok(std::path::PathBuf::from(home).join(".config/opencode/commands"))
}

fn render_result(
    json: bool,
    target: core_command::CommandTarget,
    output_dir: &Path,
    check: bool,
    dry_run: bool,
    paths: Vec<std::path::PathBuf>,
) -> Result<()> {
    if json {
        let target_name = match target {
            core_command::CommandTarget::Claude => "claude",
            core_command::CommandTarget::OpenCode => "opencode",
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "target": target_name,
                "output_dir": output_dir,
                "check": check,
                "dry_run": dry_run,
                "commands": paths,
            }))?
        );
    } else {
        let verb = if check {
            "Checked"
        } else if dry_run {
            "Would write"
        } else {
            "Wrote"
        };
        println!("{verb} {} commands:", paths.len());
        for path in paths {
            println!("  {}", path.display());
        }
    }
    Ok(())
}
