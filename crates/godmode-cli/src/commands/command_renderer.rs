use crate::{CommandAction, CommandTargetArg};
use anyhow::Result;
use godmode_core::command;
use std::path::Path;
pub fn run_command_action(root: &Path, json: bool, action: CommandAction) -> Result<()> {
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
                (CommandTargetArg::OpenCode, None) => anyhow::bail!(
                    "--output-dir is required for OpenCode generation; use command install-opencode for the default global destination"
                ),
            };
            (
                source_dir.unwrap_or_else(|| root.join("command-support/gm")),
                command::CommandTarget::from(target),
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
            let output = if let Some(path) = output_dir {
                path
            } else {
                let home =
                    std::env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
                std::path::PathBuf::from(home).join(".config/opencode/commands")
            };
            (
                source_dir.unwrap_or_else(|| root.join("command-support/gm")),
                command::CommandTarget::OpenCode,
                output,
                false,
                dry_run,
            )
        }
    };
    let definitions = command::load_command_definitions(&source_dir)?;
    let rendered = command::render_commands(&definitions, target, &source_dir.join("templates"))?;
    let paths = if check {
        command::check_rendered_commands(&rendered, &output_dir)?;
        rendered
            .iter()
            .map(|item| output_dir.join(&item.file_name))
            .collect::<Vec<_>>()
    } else {
        command::write_rendered_commands(&rendered, &output_dir, dry_run)?
    };
    if json {
        let target_name = match target {
            command::CommandTarget::Claude => "claude",
            command::CommandTarget::OpenCode => "opencode",
        };
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"ok":true,"target":target_name,"output_dir":output_dir,"check":check,"dry_run":dry_run,"commands":paths})
            )?
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
