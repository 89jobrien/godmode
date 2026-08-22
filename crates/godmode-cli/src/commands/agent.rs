//! Agent listing, index generation, definition generation, and dispatch.

use anyhow::Result;
use godmode_core::session::Session;
use godmode_core::{agent, agent_index, dispatch, plan};
use std::path::Path;

use crate::AgentAction;

pub fn run_agent_action(root: &Path, json: bool, action: AgentAction) -> Result<()> {
    match action {
        AgentAction::List { filter } => {
            let mut agents = agent_index::list_agents(root)?;
            if let Some(kw) = &filter {
                agents = agent_index::filter_agents(agents, kw);
            }
            // Always regenerate INDEX.md
            agent_index::generate_agent_index(root, &agents)?;
            if agents.is_empty() {
                if json {
                    println!("[]");
                } else {
                    println!("No agents found.");
                }
                return Ok(());
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&agents)?);
            } else {
                println!("{:<36} {:<10} SKILLS", "NAME", "COLOR");
                for a in &agents {
                    println!("{:<36} {:<10} {}", a.name, a.color, a.skills.join(", "));
                }
            }
            Ok(())
        }

        AgentAction::Index => {
            let agents = agent_index::list_agents(root)?;
            agent_index::generate_agent_index(root, &agents)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "entries": agents.len()})
                );
            } else {
                println!("Generated agents/INDEX.md with {} entries.", agents.len());
            }
            Ok(())
        }

        AgentAction::Generate { name, all } => {
            let agents_dir = root.join("agents");
            if !agents_dir.exists() {
                anyhow::bail!("agents/ directory not found at {}", agents_dir.display());
            }

            let cfg_dir = agents_dir.join("cfg");
            let names: Vec<String> = if all {
                // Collect from cfg/ first, then fall back to flat YAML
                let mut from_cfg = agent::list_cfg_agents(&agents_dir).unwrap_or_default();
                // Also pick up flat agents/*.yaml that don't have a cfg/ counterpart
                for entry in std::fs::read_dir(&agents_dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("yaml"))
                {
                    if let Some(stem) = entry
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .filter(|s| !from_cfg.contains(&s.to_string()))
                    {
                        from_cfg.push(stem.to_string());
                    }
                }
                from_cfg
            } else {
                let n = name
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("provide a name or --all"))?;
                vec![n.to_string()]
            };

            let mut generated = 0usize;
            for n in &names {
                let cfg_path = cfg_dir.join(format!("{n}.cfg.yaml"));
                if cfg_path.exists() {
                    // New path: cfg + prompt -> .md
                    let (md, out) = agent::generate_from_cfg(&agents_dir, n)?;
                    std::fs::write(&out, &md)?;
                    generated += 1;
                    if !json {
                        println!("Generated {} (from cfg)", out.display());
                    }
                } else {
                    // Legacy path: flat .yaml -> .md
                    let yp = agents_dir.join(format!("{n}.yaml"));
                    let def = agent::load(&yp)?;
                    let md = agent::generate_md(&def);
                    let out = yp.with_extension("md");
                    std::fs::write(&out, &md)?;
                    generated += 1;
                    if !json {
                        println!("Generated {}", out.display());
                    }
                }
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "generated": generated})
                );
            }
            Ok(())
        }

        AgentAction::Migrate { name, all } => {
            let agents_dir = root.join("agents");
            if !agents_dir.exists() {
                anyhow::bail!("agents/ directory not found at {}", agents_dir.display());
            }
            let md_files: Vec<std::path::PathBuf> = if all {
                std::fs::read_dir(&agents_dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension().and_then(|x| x.to_str()) == Some("md")
                            && p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|n| n != "INDEX.md")
                                .unwrap_or(false)
                    })
                    .collect()
            } else {
                let n = name
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("provide a name or --all"))?;
                vec![agents_dir.join(format!("{}.md", n))]
            };
            let mut migrated = 0usize;
            let mut errors = 0usize;
            for mp in &md_files {
                match agent::migrate_md_to_yaml(mp, &agents_dir) {
                    Ok(out) => {
                        migrated += 1;
                        if !json {
                            println!("Migrated {} -> {}", mp.display(), out.display());
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        if !json {
                            eprintln!("SKIP {}: {}", mp.display(), e);
                        }
                    }
                }
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "migrated": migrated, "errors": errors})
                );
            }
            Ok(())
        }

        AgentAction::Dispatch { path, max } => {
            let markdown = std::fs::read_to_string(&path)?;
            let tasks = plan::parse(&markdown)?;
            if tasks.is_empty() {
                anyhow::bail!("no tasks found in {}", path);
            }
            let mut session = Session::open(root)?;
            let mut ingested = 0usize;
            for task in tasks {
                match session.add_task(task) {
                    Ok(()) => ingested += 1,
                    Err(e) if e.to_string().contains("already exists") => {}
                    Err(e) => return Err(e),
                }
            }
            session.save()?;
            let chains = dispatch::independent_chains(session.graph(), max);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "plan": path,
                        "ingested": ingested,
                        "chains": chains,
                    }))?
                );
            } else {
                println!("=== godmode agent dispatch ===");
                println!("Plan:    {}", path);
                println!("Chains:  {}", chains.len());
                println!();
                println!("{}", serde_json::to_string_pretty(&chains)?);
                println!();
                println!("Paste the chains array into orca-strait or feed to godmode-crate-agent.");
            }
            Ok(())
        }
    }
}
