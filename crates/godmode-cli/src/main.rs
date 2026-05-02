use anyhow::Result;
use clap::{Parser, Subcommand};
use godmode_core::{detect, dispatch, graph, integrations, model, plan, templates};

#[derive(Parser)]
#[command(
    name = "godmode",
    version,
    about = "Rust-native development task graph and session manager"
)]
struct Cli {
    /// Emit machine-readable JSON instead of human text.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print triage summary at session start.
    Handon,

    /// Validate session state at session end.
    Handoff,

    /// Task graph management.
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    /// Plan operations.
    Plan {
        #[command(subcommand)]
        action: PlanAction,
    },

    /// Show independent chains ready for parallel agent dispatch (JSON).
    Dispatch {
        /// Maximum concurrent agents.
        #[arg(long, default_value = "5")]
        max: usize,
        /// Show the critical path instead of independent chains.
        #[arg(long)]
        critical_path: bool,
    },

    /// Show graph counts and next runnable task(s) — fast mid-session state check.
    Status,

    /// Ingest a plan and emit an orca-strait dispatch payload.
    Agent {
        /// Path to the plan markdown file.
        path: String,
        /// Maximum concurrent agent chains.
        #[arg(long, default_value = "5")]
        max: usize,
    },

    /// Run verification gate: nextest + clippy + fmt + non-empty git log.
    Verify {
        /// Scope to a single crate instead of --workspace.
        #[arg(long)]
        crate_name: Option<String>,
    },

    /// Wave state management for parallel agent sessions.
    Wave {
        #[command(subcommand)]
        action: WaveAction,
    },

    /// Git worktree lifecycle management.
    Worktree {
        #[command(subcommand)]
        action: WorktreeAction,
    },

    /// CI failure triage.
    Ci {
        #[command(subcommand)]
        action: CiAction,
    },

    /// GitHub issue operations.
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
}

#[derive(Subcommand)]
enum WaveAction {
    /// Initialise a new wave state file.
    Init {
        #[arg(long, default_value = "1")]
        wave: u32,
        /// Comma-separated agent/crate names.
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
    },
    /// Show current wave status.
    Status,
    /// Mark an agent slot as done.
    Done {
        agent: String,
        #[arg(long, value_delimiter = ',')]
        commits: Vec<String>,
    },
    /// Mark an agent slot as blocked.
    Block { agent: String },
    /// Exit 1 if any slot is still pending.
    Check,
}

#[derive(Subcommand)]
enum WorktreeAction {
    /// Create a worktree for a branch (optionally linked to a GH issue).
    Add {
        branch: String,
        #[arg(long)]
        issue: Option<u64>,
    },
    /// Remove a worktree after verifying its branch is merged into main.
    Remove { branch: String },
}

#[derive(Subcommand)]
enum CiAction {
    /// Fetch latest failed CI run and classify root cause.
    Triage {
        #[arg(long)]
        run_id: Option<String>,
    },
}

#[derive(Subcommand)]
enum IssueAction {
    /// List open GitHub issues.
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        label: Option<String>,
    },
    /// Close a GitHub issue with a commit reference.
    Close {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        commit: String,
    },
}

#[derive(Subcommand)]
enum TaskAction {
    /// List all tasks with status.
    List,

    /// Add a new task.
    Add {
        id: String,
        title: String,
        #[arg(long, value_delimiter = ',')]
        depends_on: Vec<String>,
        #[arg(long)]
        crate_name: Option<String>,
    },

    /// Mark a task as running.
    Start { id: String },

    /// Mark a running task as done.
    Done {
        id: String,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },

    /// Mark a task as blocked.
    Block { id: String, reason: String },

    /// Unblock a blocked task (resets to pending).
    Unblock { id: String },

    /// Remove a task.
    Remove { id: String },

    /// Clear tasks from the graph.
    Clear {
        /// Remove only completed (done) tasks.
        #[arg(long, conflicts_with = "all")]
        done: bool,
        /// Remove all tasks.
        #[arg(long, conflicts_with = "done")]
        all: bool,
    },

    /// Show the next runnable task(s).
    Next,

    /// Run the shell command attached to a task's `run:` field.
    Run {
        id: String,
        /// Automatically mark the task done if the command exits 0.
        #[arg(long)]
        auto_done: bool,
    },

    /// Pull pending todos/issues into the task graph.
    Pull {
        /// Doob project name (defaults to Cargo.toml package name).
        #[arg(long)]
        project: Option<String>,
        /// Pull from GitHub Issues instead of doob.
        #[arg(long)]
        github: bool,
        /// GitHub repo (owner/repo) — defaults to current repo.
        #[arg(long, requires = "github")]
        repo: Option<String>,
        /// Filter by label (GitHub only).
        #[arg(long, requires = "github")]
        label: Option<String>,
    },

    /// Mark completed tasks as done in doob (uses `doob:` UUID in notes field).
    PushDone,

    /// Reset all blocked tasks to pending in one operation.
    UnblockAll,

    /// Apply a template to the task graph.
    Apply {
        /// Template name (looks in templates/ then ~/.config/godmode/templates/).
        name: String,
        /// Variable substitutions in key=value format.
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
    },

    /// List available templates.
    ListTemplates,
}

#[derive(Subcommand)]
enum PlanAction {
    /// Parse a plan markdown file and populate the task graph.
    Ingest {
        /// Path to the plan markdown file.
        path: String,
    },
}

fn exit_empty(json: bool) -> ! {
    if json {
        println!("[]");
    } else {
        println!("No results.");
    }
    std::process::exit(1);
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let json = cli.json;
    let root = detect::root_or_cwd()?;

    match cli.cmd {
        Cmd::Handon => {
            let out = integrations::handon(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{}", out.human);
            }
            Ok(())
        }

        Cmd::Handoff => {
            let out = integrations::handoff(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{}", out.human);
            }
            Ok(())
        }

        Cmd::Task { action } => {
            let mut g = graph::load(&root)?;
            match action {
                TaskAction::List => {
                    if g.tasks.is_empty() {
                        if json {
                            println!("[]");
                        } else {
                            println!("No tasks.");
                        }
                        return Ok(());
                    }
                    if json {
                        println!("{}", serde_json::to_string_pretty(&g.tasks)?);
                    } else {
                        for t in &g.tasks {
                            let crate_tag = t
                                .crate_name
                                .as_deref()
                                .map(|c| format!(" ({})", c))
                                .unwrap_or_default();
                            println!(
                                "[{}] {:8} {}{}",
                                t.id,
                                t.status.to_string(),
                                t.title,
                                crate_tag
                            );
                        }
                    }
                }

                TaskAction::Add {
                    id,
                    title,
                    depends_on,
                    crate_name,
                } => {
                    let mut task = model::Task::new(id.clone(), title);
                    task.depends_on = depends_on;
                    task.crate_name = crate_name;
                    graph::add(&mut g, task)?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}"}}"#, id);
                    } else {
                        println!("Task '{}' added.", id);
                    }
                }

                TaskAction::Start { id } => {
                    graph::start_traced(&mut g, &id, Some(&root))?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}","status":"running"}}"#, id);
                    } else {
                        println!("Task '{}' is now running.", id);
                    }
                }

                TaskAction::Done { id, commit, notes } => {
                    graph::complete_traced(
                        &mut g,
                        &id,
                        commit.as_deref(),
                        notes.as_deref(),
                        Some(&root),
                    )?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}","status":"done"}}"#, id);
                    } else {
                        println!("Task '{}' marked done.", id);
                    }
                }

                TaskAction::Block { id, reason } => {
                    graph::block(&mut g, &id, &reason)?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}","status":"blocked"}}"#, id);
                    } else {
                        println!("Task '{}' blocked: {}", id, reason);
                    }
                }

                TaskAction::Unblock { id } => {
                    graph::unblock(&mut g, &id)?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}","status":"pending"}}"#, id);
                    } else {
                        println!("Task '{}' unblocked.", id);
                    }
                }

                TaskAction::Remove { id } => {
                    graph::remove(&mut g, &id)?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}","removed":true}}"#, id);
                    } else {
                        println!("Task '{}' removed.", id);
                    }
                }

                TaskAction::Clear { done, all } => {
                    if !done && !all {
                        anyhow::bail!(
                            "specify --done to clear completed tasks or --all to clear everything"
                        );
                    }
                    let count = graph::clear(&mut g, done);
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"removed":{}}}"#, count);
                    } else {
                        println!("Removed {} task(s).", count);
                    }
                }

                TaskAction::Next => {
                    let next = graph::runnable(&g);
                    if next.is_empty() {
                        exit_empty(json);
                    }
                    if json {
                        println!("{}", serde_json::to_string_pretty(&next)?);
                    } else {
                        for t in next {
                            let crate_tag = t
                                .crate_name
                                .as_deref()
                                .map(|c| format!(" ({})", c))
                                .unwrap_or_default();
                            println!("[{}] {}{}", t.id, t.title, crate_tag);
                        }
                    }
                }

                TaskAction::Run { id, auto_done } => {
                    let run_cmd = g
                        .tasks
                        .iter()
                        .find(|t| t.id == id)
                        .ok_or_else(|| anyhow::anyhow!("task '{}' not found", id))?
                        .run
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("task '{}' has no `run:` field", id))?;
                    let exit = integrations::rx::run_cmd(&run_cmd)?;
                    if exit.success() && auto_done {
                        graph::complete_traced(&mut g, &id, None, None, Some(&root))?;
                        graph::save(&root, &g)?;
                        if !json {
                            println!("Task '{}' marked done.", id);
                        }
                    } else if !exit.success() {
                        std::process::exit(exit.code().unwrap_or(2));
                    }
                }

                TaskAction::Pull {
                    project,
                    github,
                    repo,
                    label,
                } => {
                    let tasks = if github {
                        integrations::gh::pull_issues(repo.as_deref(), label.as_deref())?
                    } else {
                        let project = match project {
                            Some(p) => p,
                            None => detect::package_name(&root)?,
                        };
                        let todos = integrations::doob::todo_list(&project)?;
                        integrations::doob::todos_to_tasks(&todos)
                    };
                    let mut imported = 0usize;
                    for task in tasks {
                        match graph::add(&mut g, task) {
                            Ok(()) => imported += 1,
                            Err(e) if e.to_string().contains("already exists") => {}
                            Err(e) => return Err(e),
                        }
                    }
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"imported":{}}}"#, imported);
                    } else if github {
                        println!("Imported {} issue(s) from GitHub.", imported);
                    } else {
                        println!("Imported {} pending todos from doob.", imported);
                    }
                }

                TaskAction::PushDone => {
                    let mut pushed = 0usize;
                    for task in g.tasks.iter().filter(|t| t.status == model::Status::Done) {
                        if let Some(uuid) = task.notes.strip_prefix("doob:") {
                            integrations::doob::todo_done(uuid.trim())?;
                            pushed += 1;
                        }
                    }
                    if json {
                        println!(r#"{{"ok":true,"pushed":{}}}"#, pushed);
                    } else {
                        println!("Pushed {} completed tasks to doob.", pushed);
                    }
                }

                TaskAction::UnblockAll => {
                    let count = graph::unblock_all(&mut g);
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"unblocked":{}}}"#, count);
                    } else if count == 0 {
                        println!("No blocked tasks.");
                    } else {
                        println!("Unblocked {} task(s).", count);
                    }
                }

                TaskAction::Apply { name, vars } => {
                    let path = templates::find(&root, &name)?;
                    let tmpl = templates::load(&path, &vars)?;
                    let tmpl_name = tmpl.meta.name.clone();
                    let (applied, skipped) = templates::apply(&mut g, tmpl)?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(
                            r#"{{"ok":true,"applied":{},"skipped":{}}}"#,
                            applied, skipped
                        );
                    } else {
                        println!(
                            "Applied {} task(s) from template '{}'. ({} skipped)",
                            applied, tmpl_name, skipped
                        );
                    }
                }

                TaskAction::ListTemplates => {
                    let entries = templates::list(&root)?;
                    if entries.is_empty() {
                        if json {
                            println!("[]");
                        } else {
                            println!("No templates found.");
                        }
                        return Ok(());
                    }
                    if json {
                        let arr: Vec<serde_json::Value> = entries
                            .iter()
                            .map(|e| {
                                serde_json::json!({
                                    "name": e.meta.name,
                                    "description": e.meta.description,
                                    "source": e.source.to_string(),
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&arr)?);
                    } else {
                        for e in &entries {
                            println!("[{}] {} — {}", e.source, e.meta.name, e.meta.description);
                        }
                    }
                }
            }
            Ok(())
        }

        Cmd::Plan { action } => match action {
            PlanAction::Ingest { path } => {
                let markdown = std::fs::read_to_string(&path)?;
                let tasks = plan::parse(&markdown)?;
                let count = tasks.len();
                let mut g = graph::load(&root)?;
                for task in tasks {
                    if let Err(e) = graph::add(&mut g, task)
                        && !e.to_string().contains("already exists")
                    {
                        return Err(e);
                    }
                }
                graph::save(&root, &g)?;
                if json {
                    println!(r#"{{"ok":true,"ingested":{}}}"#, count);
                } else {
                    println!("Ingested {} tasks from {}.", count, path);
                }
                Ok(())
            }
        },

        Cmd::Status => {
            let g = graph::load(&root)?;
            let summary = g.summary();
            let next = graph::runnable(&g);
            let critical = dispatch::critical_path(&g);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "done": summary.done,
                        "running": summary.running,
                        "pending": summary.pending,
                        "blocked": summary.blocked,
                        "next": next.iter().map(|t| &t.id).collect::<Vec<_>>(),
                        "critical_depth": critical.len(),
                    }))?
                );
            } else {
                println!(
                    "{} done  {} running  {} pending  {} blocked",
                    summary.done, summary.running, summary.pending, summary.blocked
                );
                println!("  critical: {} tasks deep", critical.len());
                for t in &next {
                    let crate_tag = t
                        .crate_name
                        .as_deref()
                        .map(|c| format!(" ({})", c))
                        .unwrap_or_default();
                    println!("  next: [{}] {}{}", t.id, t.title, crate_tag);
                }
            }
            Ok(())
        }

        Cmd::Dispatch {
            max,
            critical_path: cp,
        } => {
            let g = graph::load(&root)?;
            if cp {
                let path = dispatch::critical_path(&g);
                if path.is_empty() {
                    exit_empty(json);
                }
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "critical_path": path,
                            "depth": path.len(),
                        }))?
                    );
                } else {
                    println!("=== critical path ({} tasks) ===", path.len());
                    for t in &path {
                        println!("[{}] {}", t.id, t.title);
                    }
                }
            } else {
                let chains = dispatch::independent_chains(&g, max);
                if chains.is_empty() {
                    exit_empty(json);
                }
                println!("{}", serde_json::to_string_pretty(&chains)?);
            }
            Ok(())
        }

        Cmd::Agent { path, max } => {
            let markdown = std::fs::read_to_string(&path)?;
            let tasks = plan::parse(&markdown)?;
            if tasks.is_empty() {
                anyhow::bail!("no tasks found in {}", path);
            }
            let mut g = graph::load(&root)?;
            let mut ingested = 0usize;
            for task in tasks {
                match graph::add(&mut g, task) {
                    Ok(()) => ingested += 1,
                    Err(e) if e.to_string().contains("already exists") => {} // idempotent
                    Err(e) => return Err(e),
                }
            }
            graph::save(&root, &g)?;
            let chains = dispatch::independent_chains(&g, max);
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

        Cmd::Verify { crate_name } => {
            let report = godmode_core::verify::run(&root, crate_name.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                let s = |r: &godmode_core::verify::StepResult| if r.ok { "✓" } else { "✗" };
                println!("nextest  {}", s(&report.nextest));
                println!("clippy   {}", s(&report.clippy));
                println!("fmt      {}", s(&report.fmt));
                println!("commits  {}", s(&report.commits));
                if !report.passed {
                    for step in [
                        &report.nextest,
                        &report.clippy,
                        &report.fmt,
                        &report.commits,
                    ] {
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
                    println!(r#"{{"ok":true,"agent":"{}","status":"done"}}"#, agent);
                } else {
                    println!("Agent '{}' marked done.", agent);
                }
                Ok(())
            }
            WaveAction::Block { agent } => {
                godmode_core::wave::mark_blocked(&root, &agent)?;
                if json {
                    println!(r#"{{"ok":true,"agent":"{}","status":"blocked"}}"#, agent);
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
                        r#"{{"settled":{},"all_done":{}}}"#,
                        settled,
                        godmode_core::wave::all_done(&state)
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
                        r#"{{"ok":true,"branch":"{}","path":"{}"}}"#,
                        info.branch,
                        info.path.display()
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
                    println!(r#"{{"ok":true,"branch":"{}","removed":true}}"#, branch);
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
                    println!(r#"{{"ok":true,"number":{}}}"#, number);
                } else {
                    println!("Issue #{} closed (commit {}).", number, commit);
                }
                Ok(())
            }
        },
    }
}
