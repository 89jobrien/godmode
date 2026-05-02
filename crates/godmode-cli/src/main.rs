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

    /// Show the next runnable task(s).
    Next,

    /// Run the shell command attached to a task's `run:` field.
    Run { id: String },
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
                    graph::start(&mut g, &id)?;
                    graph::save(&root, &g)?;
                    if json {
                        println!(r#"{{"ok":true,"id":"{}","status":"running"}}"#, id);
                    } else {
                        println!("Task '{}' is now running.", id);
                    }
                }

                TaskAction::Done { id, commit, notes } => {
                    graph::complete(&mut g, &id, commit.as_deref(), notes.as_deref())?;
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

                TaskAction::Run { id } => {
                    let task = g
                        .tasks
                        .iter()
                        .find(|t| t.id == id)
                        .ok_or_else(|| anyhow::anyhow!("task '{}' not found", id))?;
                    let run_cmd = task
                        .run
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("task '{}' has no `run:` field", id))?;
                    let status = integrations::rx::run_cmd(&run_cmd)?;
                    if !status.success() {
                        std::process::exit(status.code().unwrap_or(2));
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
                    graph::add(&mut g, task)?;
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

        Cmd::Dispatch { max } => {
            let g = graph::load(&root)?;
            let chains = dispatch::independent_chains(&g, max);
            if chains.is_empty() {
                exit_empty(json);
            }
            println!("{}", serde_json::to_string_pretty(&chains)?);
            Ok(())
        }
    }
}
