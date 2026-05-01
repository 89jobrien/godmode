use anyhow::Result;
use clap::{Parser, Subcommand};
use godmode_core::{detect, dispatch, graph, model, plan, session};

#[derive(Parser)]
#[command(
    name = "godmode",
    version,
    about = "Rust-native development task graph and session manager"
)]
struct Cli {
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

    /// Remove a task.
    Remove { id: String },

    /// Show the next runnable task(s).
    Next,
}

#[derive(Subcommand)]
enum PlanAction {
    /// Parse a plan markdown file and populate the task graph.
    Ingest {
        /// Path to the plan markdown file.
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = detect::root_or_cwd()?;

    match cli.cmd {
        Cmd::Handon => session::handon(&root),

        Cmd::Handoff => {
            let summary = session::handoff(&root)?;
            println!(
                "Session closed. done={} running={} pending={} blocked={}",
                summary.done, summary.running, summary.pending, summary.blocked
            );
            Ok(())
        }

        Cmd::Task { action } => {
            let mut g = graph::load(&root)?;
            match action {
                TaskAction::List => {
                    if g.tasks.is_empty() {
                        println!("No tasks.");
                        return Ok(());
                    }
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

                TaskAction::Add {
                    id,
                    title,
                    depends_on,
                    crate_name,
                } => {
                    let mut task = model::Task::new(id, title);
                    task.depends_on = depends_on;
                    task.crate_name = crate_name;
                    graph::add(&mut g, task)?;
                    graph::save(&root, &g)?;
                    println!("Task added.");
                }

                TaskAction::Start { id } => {
                    graph::start(&mut g, &id)?;
                    graph::save(&root, &g)?;
                    println!("Task '{}' is now running.", id);
                }

                TaskAction::Done { id, commit, notes } => {
                    graph::complete(&mut g, &id, commit.as_deref(), notes.as_deref())?;
                    graph::save(&root, &g)?;
                    println!("Task '{}' marked done.", id);
                }

                TaskAction::Block { id, reason } => {
                    graph::block(&mut g, &id, &reason)?;
                    graph::save(&root, &g)?;
                    println!("Task '{}' blocked: {}", id, reason);
                }

                TaskAction::Remove { id } => {
                    graph::remove(&mut g, &id)?;
                    graph::save(&root, &g)?;
                    println!("Task '{}' removed.", id);
                }

                TaskAction::Next => {
                    let next = graph::runnable(&g);
                    if next.is_empty() {
                        println!("No runnable tasks.");
                    } else {
                        for t in next {
                            println!("[{}] {}", t.id, t.title);
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
                    graph::add(&mut g, task)?;
                }
                graph::save(&root, &g)?;
                println!("Ingested {} tasks from {}.", count, path);
                Ok(())
            }
        },

        Cmd::Dispatch { max } => {
            let g = graph::load(&root)?;
            let chains = dispatch::independent_chains(&g, max);
            println!("{}", serde_json::to_string_pretty(&chains)?);
            Ok(())
        }
    }
}
