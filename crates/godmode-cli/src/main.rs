use anyhow::Result;
use clap::{Parser, Subcommand};
use godmode_core::{detect, dispatch, graph, integrations, model, plan};

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

    /// Pull pending doob todos into the task graph.
    Pull {
        /// Doob project name (defaults to Cargo.toml package name).
        #[arg(long)]
        project: Option<String>,
    },

    /// Mark completed tasks as done in doob (uses `doob:` UUID in notes field).
    PushDone,

    /// Reset all blocked tasks to pending in one operation.
    UnblockAll,
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
                        exit_empty(json);
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

                TaskAction::Pull { project } => {
                    let project = match project {
                        Some(p) => p,
                        None => detect::package_name(&root)?,
                    };
                    let todos = integrations::doob::todo_list(&project)?;
                    let tasks = integrations::doob::todos_to_tasks(&todos);
                    let count = tasks.len();
                    for task in tasks {
                        if let Err(e) = graph::add(&mut g, task)
                            && !e.to_string().contains("already exists")
                        {
                            return Err(e);
                        }
                    }
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"imported":{}}}"#, count);
                    } else {
                        println!(
                            "Imported {} pending todos from doob project '{}'.",
                            count, project
                        );
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
    }
}
