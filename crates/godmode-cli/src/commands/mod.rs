//! Command adapters extracted from `main.rs`.

mod command;
pub(crate) mod hook;
mod session;
pub(crate) mod task;
mod workspace;

use anyhow::Result;
use godmode_core::{
    agent, agent_index, builder, detect, dispatch, insights, memory_banking, pipeline, plan,
    policy, registry, release, review, session::Session, skill, workflow,
};

use crate::*;

pub(crate) use hook::run_hook_action;
pub(crate) use task::run_task_action;

fn exit_empty(json: bool) -> ! {
    if json {
        println!("[]");
    } else {
        println!("No results.");
    }
    std::process::exit(2);
}

// qual:allow reason: "exhaustive CLI integration root covered by command tests"
pub(crate) fn dispatch(root: std::path::PathBuf, json: bool, sarif: bool, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Handon { compact } => session::run_handon(&root, json, compact),
        Cmd::Handoff => session::run_handoff(&root, json),
        Cmd::Session { action } => session::run_session(&root, json, action),
        Cmd::Trace { action } => session::run_trace(&root, json, action),

        Cmd::Task { action } => run_task_action(&root, json, action),

        Cmd::Plan { action } => match action {
            PlanAction::Ingest { path } => {
                let markdown = std::fs::read_to_string(&path)?;
                let tasks = plan::parse(&markdown)?;
                let count = tasks.len();
                let mut session = Session::open(&root)?;
                for task in tasks {
                    if let Err(e) = session.add_task(task)
                        && !e.to_string().contains("already exists")
                    {
                        return Err(e);
                    }
                }
                session.save()?;
                if json {
                    println!("{}", serde_json::json!({"ok": true, "ingested": count}));
                } else {
                    println!("Ingested {} tasks from {}.", count, path);
                }
                Ok(())
            }
        },

        Cmd::Command { action } => command::run(&root, json, action),

        Cmd::Context => workspace::run_context(&root, json),

        Cmd::Status { compact } => workspace::run_status(&root, json, compact),

        Cmd::Hook { action } => run_hook_action(&root, json, action),

        Cmd::Dispatch { max, critical_path } => {
            workspace::run_dispatch(&root, json, max, critical_path)
        }

        Cmd::Agent { action } => match action {
            AgentAction::List { filter } => {
                let all_agents = agent_index::list_agents(&root)?;
                agent_index::generate_agent_index(&root, &all_agents)?;
                let mut agents = all_agents;
                if let Some(kw) = &filter {
                    agents = agent_index::filter_agents(agents, kw);
                }
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

            AgentAction::Index { check } => {
                let agents = agent_index::list_agents(&root)?;
                if check && !agent_index::agent_index_is_current(&root, &agents)? {
                    anyhow::bail!("agents/INDEX.md is stale");
                }
                if !check {
                    agent_index::generate_agent_index(&root, &agents)?;
                }
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "entries": agents.len(), "check": check})
                    );
                } else if check {
                    println!("agents/INDEX.md is current.");
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

            AgentAction::InstallOpenCode {
                catalog,
                output_dir,
                dry_run,
            } => {
                let catalog = agent::load_opencode_catalog(catalog.as_deref())?;
                let output_dir = if let Some(path) = output_dir {
                    path
                } else {
                    let home = std::env::var_os("HOME")
                        .ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
                    std::path::PathBuf::from(home)
                        .join(".config")
                        .join("opencode")
                        .join("agents")
                };
                let paths = agent::install_opencode_agents(&catalog, &output_dir, dry_run)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "dry_run": dry_run,
                            "agents": paths,
                        }))?
                    );
                } else {
                    let verb = if dry_run {
                        "Would install"
                    } else {
                        "Installed"
                    };
                    println!("{verb} {} OpenCode agents:", paths.len());
                    for path in paths {
                        println!("  {}", path.display());
                    }
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
                let mut session = Session::open(&root)?;
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
                    println!(
                        "Paste the chains array into orca-strait or feed to godmode-crate-agent."
                    );
                }
                Ok(())
            }
        },

        Cmd::Verify { crate_name } => {
            let config = godmode_core::config::Config::load(&root);
            let report =
                godmode_core::verify::run_with_config(&root, crate_name.as_deref(), &config)?;
            if sarif {
                let mut log = godmode_core::sarif::from_verify(&report);
                // Merge rich clippy SARIF (with file locations) as a second run
                let clippy_log = godmode_core::sarif::clippy_sarif(&root, crate_name.as_deref())?;
                log.runs.extend(clippy_log.runs);
                // Merge globstar SARIF if available
                if let Some(gs_log) = godmode_core::sarif::globstar_sarif(&root) {
                    log.runs.extend(gs_log.runs);
                }
                println!("{}", serde_json::to_string_pretty(&log)?);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                let icon = |ok: bool| if ok { "✓" } else { "✗" };
                for step in &report.steps {
                    println!("{:<9}{}", step.name, icon(step.ok));
                }
                if !report.passed {
                    for step in &report.steps {
                        if !step.ok && !step.output.is_empty() {
                            eprintln!("{}", step.output);
                        }
                    }
                }
            }
            if !report.passed {
                std::process::exit(1);
            }
            Ok(())
        }

        Cmd::Wave { action } => match action {
            WaveAction::Init { wave, agents } => {
                let agent_refs: Vec<&str> = agents.iter().map(|s| s.as_str()).collect();
                let state = godmode_core::wave::init(&root, wave, &agent_refs)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&state)?);
                } else {
                    println!(
                        "Wave {} initialised: {} agent(s).",
                        wave,
                        state.agents.len()
                    );
                    for (name, slot) in &state.agents {
                        println!("  {} — {:?}", name, slot.status);
                    }
                }
                Ok(())
            }
            WaveAction::Status => {
                let state = godmode_core::wave::load(&root)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&state)?);
                } else {
                    println!("Wave {}:", state.wave);
                    for (name, slot) in &state.agents {
                        println!(
                            "  {:20} {:?}  commits: {}",
                            name,
                            slot.status,
                            slot.commits.join(", ")
                        );
                    }
                }
                Ok(())
            }
            WaveAction::Done { agent, commits } => {
                godmode_core::wave::mark_done(&root, &agent, commits)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "agent": agent, "status": "done"})
                    );
                } else {
                    println!("Agent '{}' marked done.", agent);
                }
                Ok(())
            }
            WaveAction::Block { agent } => {
                godmode_core::wave::mark_blocked(&root, &agent)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "agent": agent, "status": "blocked"})
                    );
                } else {
                    println!("Agent '{}' marked blocked.", agent);
                }
                Ok(())
            }
            WaveAction::Check => {
                let state = godmode_core::wave::load(&root)?;
                let settled = godmode_core::wave::check(&state);
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"settled": settled, "all_done": godmode_core::wave::all_done(&state)})
                    );
                } else if settled {
                    println!(
                        "Wave settled. all_done={}",
                        godmode_core::wave::all_done(&state)
                    );
                } else {
                    let pending: Vec<_> = state
                        .agents
                        .iter()
                        .filter(|(_, s)| s.status == godmode_core::wave::SlotStatus::Pending)
                        .map(|(n, _)| n.as_str())
                        .collect();
                    println!("Wave not settled. Pending: {}", pending.join(", "));
                }
                if !settled {
                    std::process::exit(1);
                }
                Ok(())
            }
        },

        Cmd::Worktree { action } => match action {
            WorktreeAction::Add { branch, issue } => {
                let info = godmode_core::worktree::add(&root, &branch, issue)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "branch": info.branch, "path": info.path.display().to_string()})
                    );
                } else {
                    println!(
                        "Worktree created: {} → {}",
                        info.branch,
                        info.path.display()
                    );
                }
                Ok(())
            }
            WorktreeAction::Remove { branch } => {
                godmode_core::worktree::remove(&root, &branch)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "branch": branch, "removed": true})
                    );
                } else {
                    println!("Worktree removed: {}", branch);
                }
                Ok(())
            }
        },

        Cmd::Ci { action } => match action {
            CiAction::Triage { run_id } => {
                let result = godmode_core::integrations::gh::ci_triage(run_id.as_deref())?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("Run:   {}", result.run_id);
                    println!("Class: {:?}", result.class);
                    println!("Fix:   {}", result.fix_hint);
                    if !result.raw_snippet.is_empty() {
                        println!("\n--- log snippet ---\n{}", result.raw_snippet);
                    }
                }
                Ok(())
            }
        },

        Cmd::Issue { action } => match action {
            IssueAction::List { repo, label } => {
                let tasks =
                    godmode_core::integrations::gh::pull_issues(repo.as_deref(), label.as_deref())?;
                if tasks.is_empty() {
                    if json {
                        println!("[]");
                    } else {
                        println!("No open issues.");
                    }
                    return Ok(());
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&tasks)?);
                } else {
                    for t in &tasks {
                        println!("[{}] {}", t.id, t.title);
                    }
                }
                Ok(())
            }
            IssueAction::Close {
                number,
                repo,
                commit,
            } => {
                godmode_core::integrations::gh::issue_close(number, repo.as_deref(), &commit)?;
                if json {
                    println!("{}", serde_json::json!({"ok": true, "number": number}));
                } else {
                    println!("Issue #{} closed (commit {}).", number, commit);
                }
                Ok(())
            }
        },

        Cmd::Graph { action } => match action {
            GraphAction::Build { input, vars } => {
                let summary = match input {
                    Some(path) => {
                        let p = std::path::PathBuf::from(&path);
                        builder::build_from_file(&root, &p, &vars)?
                    }
                    None => builder::build_interactive(&root)?,
                };
                if json {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                } else {
                    println!(
                        "Added {} task(s), {} dep(s) wired.",
                        summary.added, summary.wired
                    );
                    if !summary.findings.is_empty() {
                        for f in &summary.findings {
                            eprintln!("! {}", f);
                        }
                    }
                    if summary.next.is_empty() {
                        std::process::exit(1);
                    }
                }
                Ok(())
            }
        },

        Cmd::Skill { action } => match action {
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
            SkillAction::Index { check } => {
                let skills = skill::list_local(&root.join("skills"))?;
                if check && !skill::skill_index_is_current(&root, &skills)? {
                    anyhow::bail!("skills/INDEX.md is stale");
                }
                if !check {
                    skill::generate_skill_index(&root, &skills)?;
                }
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "entries": skills.len(), "check": check})
                    );
                } else if check {
                    println!("skills/INDEX.md is current.");
                } else {
                    println!("Generated skills/INDEX.md with {} entries.", skills.len());
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
        },

        Cmd::Review { action } => {
            let report = match action {
                ReviewAction::Self_ => review::run_all(&root)?,
                ReviewAction::Skills => review::check_skills(&root)?,
                ReviewAction::Agents => review::check_agents(&root)?,
            };
            if sarif {
                let log = godmode_core::sarif::from_review(&report);
                println!("{}", serde_json::to_string_pretty(&log)?);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if report.passed {
                println!("{} checks passed.", report.checks);
            } else {
                for f in &report.findings {
                    println!("{}", f.message);
                }
                println!(
                    "\n{} checks failed out of {} total.",
                    report.findings.len(),
                    report.checks
                );
            }
            if !report.passed {
                std::process::exit(1);
            }
            Ok(())
        }

        Cmd::Release { action } => match action {
            ReleaseAction::Current => {
                let v = release::current_version(&root)?;
                if json {
                    println!("{}", serde_json::json!({"version": v}));
                } else {
                    println!("{}", v);
                }
                Ok(())
            }
            ReleaseAction::Bump { version } => {
                let info = release::bump(&root, version.as_deref())?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&info)?);
                } else {
                    println!("{} → {}", info.old_version, info.new_version);
                }
                Ok(())
            }
            ReleaseAction::Tag => {
                let tag = release::tag(&root)?;
                if json {
                    println!("{}", serde_json::json!({"tag": tag}));
                } else {
                    println!("Tagged {}", tag);
                }
                Ok(())
            }
            ReleaseAction::Push => {
                release::push(&root)?;
                if json {
                    println!("{}", serde_json::json!({"ok": true}));
                } else {
                    println!("Pushed branch and tag.");
                }
                Ok(())
            }
            ReleaseAction::Changelog => {
                let entry = release::generate_changelog(&root)?;
                release::write_changelog(&root, &entry)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"ok": true, "version": entry.version, "date": entry.date})
                    );
                } else {
                    println!("Updated CHANGELOG.md for version {}.", entry.version);
                }
                Ok(())
            }

            ReleaseAction::Validate => {
                let warnings = release::validate_versions(&root)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&warnings)?);
                } else if warnings.is_empty() {
                    println!("All versions consistent.");
                } else {
                    println!("Version drift detected:");
                    for w in &warnings {
                        println!("  - {w}");
                    }
                    std::process::exit(1);
                }
                Ok(())
            }
        },

        Cmd::Workflow { action } => match action {
            WorkflowAction::Run {
                agent: agent_name,
                workflow: wf_name,
            } => {
                let agents_dir = root.join("agents");
                let agent_path = agents_dir.join(format!("{}.yaml", agent_name));
                let agent_def = agent::load(&agent_path)?;
                let wf_ref = agent_def
                    .workflows
                    .iter()
                    .find(|w| w.name == wf_name)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "workflow '{}' not found in agent '{}'",
                            wf_name,
                            agent_name
                        )
                    })?;
                let wf_path = root.join(&wf_ref.path);
                let wf_def = workflow::load(&wf_path)?;
                let final_state = workflow::run(&wf_def, &root)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&final_state)?);
                } else {
                    for s in &final_state.steps {
                        let state_str = format!("{:?}", s.state);
                        let code = s
                            .exit_code
                            .map(|c| format!(" (exit {})", c))
                            .unwrap_or_default();
                        println!("[{:8}] {}{}", state_str, s.id, code);
                    }
                }
                Ok(())
            }

            WorkflowAction::List {
                agent: agent_filter,
            } => {
                let agents_dir = root.join("agents");
                let mut entries: Vec<serde_json::Value> = vec![];
                if agents_dir.exists() {
                    let yaml_files: Vec<std::path::PathBuf> = std::fs::read_dir(&agents_dir)?
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("yaml"))
                        .collect();
                    for yf in yaml_files {
                        let Ok(a) = agent::load(&yf) else { continue };
                        if agent_filter.as_deref().is_some_and(|f| a.name != f) {
                            continue;
                        }
                        for wf in &a.workflows {
                            entries.push(serde_json::json!({
                                "agent": a.name,
                                "workflow": wf.name,
                                "path": wf.path,
                                "slash_command": wf.slash_command,
                            }));
                        }
                    }
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&entries)?);
                } else if entries.is_empty() {
                    println!("No workflows found.");
                } else {
                    println!("{:<30} {:<30} PATH", "AGENT", "WORKFLOW");
                    for e in &entries {
                        println!(
                            "{:<30} {:<30} {}",
                            e["agent"].as_str().unwrap_or(""),
                            e["workflow"].as_str().unwrap_or(""),
                            e["path"].as_str().unwrap_or(""),
                        );
                    }
                }
                Ok(())
            }

            WorkflowAction::Status { name } => {
                let state_path = root
                    .join(".ctx")
                    .join("godmode")
                    .join(format!("workflow-{}.json", name));
                if !state_path.exists() {
                    if json {
                        println!("null");
                    } else {
                        println!("No state file found for workflow '{}'.", name);
                    }
                    return Ok(());
                }
                let raw = std::fs::read_to_string(&state_path)?;
                let state: workflow::WorkflowState = serde_json::from_str(&raw)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&state)?);
                } else {
                    println!("Workflow: {}", state.workflow);
                    for s in &state.steps {
                        let state_str = format!("{:?}", s.state);
                        let code = s
                            .exit_code
                            .map(|c| format!(" (exit {})", c))
                            .unwrap_or_default();
                        println!("[{:8}] {}{}", state_str, s.id, code);
                    }
                }
                Ok(())
            }
        },

        Cmd::VisualizeGraph { format, out } => {
            workspace::run_visualize_graph(&root, json, &format, out)
        }
        Cmd::MemoryBanking { action } => {
            match action {
                MemoryBankingAction::Inject => memory_banking::inject(&root, json)?,
                MemoryBankingAction::Remind => memory_banking::remind(&root, json)?,
                MemoryBankingAction::Init => memory_banking::init(&root)?,
                MemoryBankingAction::Status => memory_banking::status(&root, json)?,
            }
            Ok(())
        }

        Cmd::Insight { action } => {
            fn parse_date_or_today(d: &Option<String>) -> Result<insights::NaiveDate> {
                match d {
                    Some(s) => Ok(insights::NaiveDate::parse_from_str(s, "%Y-%m-%d")?),
                    None => Ok(insights::today()),
                }
            }

            match action {
                InsightAction::Add { title, body, tags } => {
                    let insight = insights::new_insight(title, body, tags);
                    insights::append(&root, &insight)?;
                    if json {
                        println!("{}", serde_json::to_string(&insight)?);
                    } else {
                        println!("Recorded: {}", insight.title);
                    }
                }
                InsightAction::List { date } => {
                    let d = parse_date_or_today(&date)?;
                    let items = insights::list_for_date(&root, d)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&items)?);
                    } else if items.is_empty() {
                        println!("No insights for {d}.");
                        std::process::exit(2);
                    } else {
                        for i in &items {
                            let tags = if i.tags.is_empty() {
                                String::new()
                            } else {
                                format!(" [{}]", i.tags.join(", "))
                            };
                            println!("- {}{}", i.title, tags);
                        }
                    }
                }
                InsightAction::Render { date } => {
                    let d = parse_date_or_today(&date)?;
                    let path = insights::render_markdown(&root, d)?;
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({ "path": path.display().to_string() })
                        );
                    } else {
                        println!("Wrote {}", path.display());
                    }
                }
            }
            Ok(())
        }

        Cmd::Pipeline { action } => match action {
            PipelineAction::List => {
                let pipelines = pipeline::load_pipelines(&root)?;
                if pipelines.is_empty() {
                    if json {
                        println!("[]");
                    } else {
                        println!("No pipelines found.");
                    }
                    return Ok(());
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&pipelines)?);
                } else {
                    for p in &pipelines {
                        println!("{} — {}", p.name, p.description);
                    }
                }
                Ok(())
            }

            PipelineAction::Show { name } => {
                let p = pipeline::load_pipeline(&root, &name)?;
                let state = pipeline::load_state(&root)?;
                let active_idx = state
                    .as_ref()
                    .filter(|s| s.active == name)
                    .map(|s| s.current_step);
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "pipeline": p,
                            "current_step": active_idx,
                        }))?
                    );
                } else {
                    println!("Pipeline: {} — {}", p.name, p.description);
                    for (i, step) in p.steps.iter().enumerate() {
                        let marker = if active_idx == Some(i) { ">>" } else { "  " };
                        println!("{} [{}] {}", marker, i + 1, step.skill);
                    }
                }
                Ok(())
            }

            PipelineAction::Start { name, from } => {
                let p = pipeline::load_pipeline(&root, &name)?;
                let state = pipeline::start(&p, from.as_deref())?;
                let first = pipeline::current_step(&state, &p)
                    .map(|s| s.skill.as_str())
                    .unwrap_or("(none)");
                pipeline::save_state(&root, &state)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&state)?);
                } else {
                    println!("Pipeline '{}' started at step: {}", name, first);
                }
                Ok(())
            }

            PipelineAction::Next => advance_pipeline(&root, json, pipeline::advance),

            PipelineAction::Skip => advance_pipeline(&root, json, pipeline::skip),

            PipelineAction::Stop => {
                pipeline::clear_state(&root)?;
                if json {
                    println!("{}", serde_json::json!({"ok": true}));
                } else {
                    println!("Pipeline stopped.");
                }
                Ok(())
            }

            PipelineAction::Run {
                name,
                from: _from,
                fail_fast,
            } => {
                let result = pipeline::run_tasks(&root, &name, fail_fast)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    for sr in &result.steps {
                        if sr.skipped {
                            println!("  [skip] {}", sr.skill);
                        } else {
                            println!(
                                "  [{}] {} — {} task(s), {} failed",
                                if sr.tasks_failed > 0 { "FAIL" } else { "ok" },
                                sr.skill,
                                sr.tasks_run,
                                sr.tasks_failed,
                            );
                        }
                    }
                    if result.completed {
                        println!("Pipeline complete.");
                    } else if let Some(ref skill) = result.stopped_at {
                        println!("Stopped at: {skill}");
                        std::process::exit(1);
                    }
                }
                Ok(())
            }

            PipelineAction::Status => {
                let state = pipeline::load_state(&root)?;
                match state {
                    None => {
                        if json {
                            println!("null");
                        } else {
                            println!("No active pipeline.");
                        }
                    }
                    Some(s) => {
                        let p = pipeline::load_pipeline(&root, &s.active)?;
                        let (done, total) = pipeline::progress(&s, &p);
                        let current = pipeline::current_step(&s, &p)
                            .map(|step| step.skill.as_str())
                            .unwrap_or("(complete)");
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "active": s.active,
                                    "current_step": current,
                                    "progress": { "done": done, "total": total },
                                    "complete": pipeline::is_complete(&s, &p),
                                }))?
                            );
                        } else {
                            println!("Pipeline: {}", s.active);
                            println!("Step:     {}", current);
                            println!("Progress: {}/{}", done, total);
                        }
                    }
                }
                Ok(())
            }
        },

        Cmd::Policy { action } => {
            match action {
                PolicyCmdAction::Resolve { agent, level } => {
                    let level_parsed = level
                        .as_deref()
                        .map(|l| l.parse::<policy::GovernanceLevel>())
                        .transpose()?;
                    let resolved = policy::resolve(&root, &agent, level_parsed.as_ref())?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&resolved)?);
                    } else {
                        println!("Agent:    {}", resolved.agent);
                        println!("Category: {}", resolved.category);
                        println!("Level:    {}", resolved.level);
                        println!("Sources:  {}", resolved.sources.join(" + "));
                        println!();
                        let p = &resolved.policy;
                        if p.allowed_tools.is_empty() {
                            println!("Allowed tools: (all)");
                        } else {
                            println!("Allowed tools: {}", p.allowed_tools.join(", "));
                        }
                        if !p.blocked_tools.is_empty() {
                            println!("Blocked tools: {}", p.blocked_tools.join(", "));
                        }
                        println!("Max calls/dispatch: {}", p.max_calls_per_dispatch);
                        if !p.require_human_approval.is_empty() {
                            println!("Require approval: {}", p.require_human_approval.join(", "));
                        }
                        println!();
                        println!("Subagent constraints:");
                        println!("  max_concurrent: {}", p.subagent.max_concurrent);
                        println!("  verify_branch:  {}", p.subagent.must_verify_branch);
                        println!("  no_main:        {}", p.subagent.no_commit_to_main);
                        println!("  max_retries:    {}", p.subagent.max_retries_on_failure);
                        println!(
                            "  require_commit: {}",
                            p.subagent.require_commit_before_done
                        );
                        if !p.subagent.blocked_flags.is_empty() {
                            println!("  blocked_flags:  {}", p.subagent.blocked_flags.join(", "));
                        }
                    }
                }
                PolicyCmdAction::Check {
                    agent,
                    tool,
                    input,
                    level,
                } => {
                    let level_parsed = level
                        .as_deref()
                        .map(|l| l.parse::<policy::GovernanceLevel>())
                        .transpose()?;
                    let resolved = policy::resolve(&root, &agent, level_parsed.as_ref())?;
                    let result = policy::check_tool(&resolved.policy, &tool, input.as_deref());
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        let symbol = match result.action {
                            policy::PolicyAction::Allow => "ALLOW",
                            policy::PolicyAction::Deny => "DENY",
                            policy::PolicyAction::Review => "REVIEW",
                        };
                        println!("{symbol}: {}", result.reason);
                    }
                    // Exit 1 on deny for scripting
                    if result.action == policy::PolicyAction::Deny {
                        std::process::exit(1);
                    }
                }
                PolicyCmdAction::List => {
                    let index = policy::list_policies(&root)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&index)?);
                    } else {
                        if let Some(ref d) = index.default {
                            println!("Default: {} (level: {})", d.name, d.level);
                        }
                        if !index.categories.is_empty() {
                            println!();
                            println!("Categories:");
                            let mut cats: Vec<_> = index.categories.keys().collect();
                            cats.sort();
                            for cat in cats {
                                let p = &index.categories[cat];
                                println!(
                                    "  {cat:<8}  tools: {}  max: {}",
                                    if p.allowed_tools.is_empty() {
                                        "(all)".to_string()
                                    } else {
                                        p.allowed_tools.join(",")
                                    },
                                    p.max_calls_per_dispatch,
                                );
                            }
                        }
                        if !index.levels.is_empty() {
                            println!();
                            println!("Levels:");
                            for level_name in &["open", "standard", "strict", "locked"] {
                                if let Some(p) = index.levels.get(*level_name) {
                                    println!(
                                        "  {:<10}  max: {}",
                                        level_name, p.max_calls_per_dispatch,
                                    );
                                }
                            }
                        }
                    }
                }
                PolicyCmdAction::Audit { date } => {
                    let date_str = date.unwrap_or_else(|| insights::today().to_string());
                    let events = policy::read_audit_events(&root, Some(&date_str))?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&events)?);
                    } else if events.is_empty() {
                        println!("No governance events for {date_str}.");
                    } else {
                        let denied = events.iter().filter(|e| e.action == "denied").count();
                        let reviews = events
                            .iter()
                            .filter(|e| e.action == "review" || e.action == "warn")
                            .count();
                        let allowed = events.iter().filter(|e| e.action == "allowed").count();
                        println!("Governance audit for {date_str}:");
                        println!(
                            "  {} events: {} denied, {} review, {} allowed",
                            events.len(),
                            denied,
                            reviews,
                            allowed,
                        );
                        println!();
                        for ev in &events {
                            if ev.action == "denied" || ev.action == "review" || ev.action == "warn"
                            {
                                println!(
                                    "  [{action}] {agent} -> {tool}: {reason}",
                                    action = ev.action.to_uppercase(),
                                    agent = ev.agent_id,
                                    tool = ev.tool_name,
                                    reason = ev.reason,
                                );
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        Cmd::Pin { path } => {
            let target = match path {
                Some(p) => std::path::PathBuf::from(p),
                None => std::env::current_dir()?,
            };
            detect::pin_root(&root, &target)?;
            let canonical = target.canonicalize()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"pinned_root": canonical.to_string_lossy()})
                );
            } else {
                println!("Pinned to {}", canonical.display());
            }
            Ok(())
        }

        Cmd::Unpin => {
            let removed = detect::unpin_root(&root)?;
            if json {
                println!("{}", serde_json::json!({"unpinned": removed}));
            } else if removed {
                println!("Unpinned.");
            } else {
                println!("No pin was set.");
            }
            Ok(())
        }

        Cmd::Init => {
            use godmode_core::doctor::RealProbe;
            use godmode_core::init::{RealFs, run_init};

            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            let global_dir = std::path::PathBuf::from(&home)
                .join(".config")
                .join("godmode");
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let report = run_init(&RealFs, &RealProbe, &cwd, &global_dir)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                if report.global_created {
                    println!("Created global config: {}", report.global_path.display());
                } else {
                    println!(
                        "Global config already exists: {}",
                        report.global_path.display()
                    );
                }
                if report.project_created {
                    if let Some(ref p) = report.project_path {
                        println!("Created project state: {}", p.display());
                    }
                } else if report.project_path.is_some() {
                    println!("Project state already exists.");
                } else {
                    println!("No Rust project detected (no Cargo.toml found).");
                }
                if report.gitignore_updated {
                    println!("Added .ctx/ to .gitignore");
                }
                println!();
                println!("Doctor:");
                for c in &report.doctor.checks {
                    let icon = if c.passed { "ok" } else { "FAIL" };
                    println!("  [{icon}] {}: {}", c.name, c.detail);
                }
            }
            Ok(())
        }

        Cmd::Doctor => {
            use godmode_core::doctor::{RealProbe, run_doctor};

            let report = run_doctor(&RealProbe);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                for c in &report.checks {
                    let icon = if c.passed { "ok" } else { "FAIL" };
                    println!("[{icon}] {}: {}", c.name, c.detail);
                }
                if report.all_passed {
                    println!("\nAll checks passed.");
                } else {
                    println!("\nSome checks failed.");
                }
            }
            Ok(())
        }

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

        Cmd::TestCheck { path } => {
            use godmode_core::test_check;

            let git_root = detect::root_or_cwd()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            match test_check::check_test_coverage(&path, &git_root) {
                Some(msg) => {
                    if json {
                        println!("{}", serde_json::json!({"covered": false, "message": msg}));
                    } else {
                        eprintln!("{msg}");
                    }
                    std::process::exit(2);
                }
                None => {
                    if json {
                        println!("{}", serde_json::json!({"covered": true}));
                    }
                    Ok(())
                }
            }
        }
    }
}

/// Shared logic for `pipeline next` and `pipeline skip`.
fn advance_pipeline(
    root: &std::path::Path,
    json: bool,
    op: for<'a> fn(
        &mut pipeline::PipelineState,
        &'a pipeline::Pipeline,
    ) -> Option<&'a pipeline::PipelineStep>,
) -> Result<()> {
    let mut state =
        pipeline::load_state(root)?.ok_or_else(|| anyhow::anyhow!("No active pipeline."))?;
    let p = pipeline::load_pipeline(root, &state.active.clone())?;
    let next = op(&mut state, &p);
    pipeline::save_state(root, &state)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&state)?);
    } else if let Some(step) = next {
        println!("Advanced to: {}", step.skill);
    } else {
        println!("Pipeline complete.");
    }
    Ok(())
}
