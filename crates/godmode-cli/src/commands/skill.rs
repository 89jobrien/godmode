//! Skill registry management.

use anyhow::Result;
use godmode_core::{registry, skill};
use std::path::Path;

use crate::SkillAction;

pub fn run_skill_action(root: &Path, json: bool, action: SkillAction) -> Result<()> {
    match action {
        SkillAction::List => {
            let skills_dir = root.join("skills");
            let skills = skill::list_local(&skills_dir)?;
            if skills.is_empty() {
                if json {
                    println!("[]");
                } else {
                    println!("No skills found.");
                }
                return Ok(());
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&skills)?);
            } else {
                println!("{:<30} PATH", "NAME");
                for s in &skills {
                    println!("{:<30} {}", s.name, s.path.display());
                }
            }
            Ok(())
        }
        SkillAction::Install { path } => {
            let p = std::path::PathBuf::from(&path);
            if !p.join("SKILL.md").exists() {
                anyhow::bail!("no SKILL.md found in {}", p.display());
            }
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid path: {}", p.display()))?
                .to_string();
            let mut reg = registry::Registry::load_global()?;
            let entry = registry::RegistryEntry {
                name: name.clone(),
                kind: registry::EntryKind::Skill,
                path: p.canonicalize().unwrap_or(p),
                version: "1.0.0".to_string(),
            };
            let is_new = reg.install(entry);
            reg.save_global()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "name": name, "new": is_new})
                );
            } else if is_new {
                println!("Installed skill '{}'.", name);
            } else {
                println!("Skill '{}' already registered.", name);
            }
            Ok(())
        }
        SkillAction::Uninstall { name } => {
            let mut reg = registry::Registry::load_global()?;
            let removed = reg.uninstall(&name);
            reg.save_global()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "name": name, "removed": removed})
                );
            } else if removed {
                println!("Uninstalled skill '{}'.", name);
            } else {
                println!("Skill '{}' was not in the registry.", name);
            }
            Ok(())
        }
    }
}
