#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! clap = { version = "4", features = ["derive"] }
//! serde = { version = "1", features = ["derive"] }
//! serde_yaml = "0.9"
//! chrono = { version = "0.4", features = ["serde"] }
//! ```
//!
//! task-runner.rs — advance TDD task phase state.
//!
//! Usage:
//!   task-runner.rs init <title> --crate <name>
//!   task-runner.rs red <id>
//!   task-runner.rs green <id>
//!   task-runner.rs refactor <id>
//!   task-runner.rs next
//!   task-runner.rs status
//!   task-runner.rs fail <id> --reason <text>
//!   task-runner.rs close-issues

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

const TASK_FILE: &str = "tdd-tasks.yaml";
const MAX_ATTEMPTS: u32 = 3;

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Pending,
    Red,
    Green,
    Refactor,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pending,
    Active,
    Done,
    Failed,
}

// `crate` is a reserved word — use rename to keep the YAML key as "crate".
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskRaw {
    id: String,
    title: String,
    #[serde(rename = "crate")]
    crate_name: String,
    test: String,
    phase: Phase,
    status: Status,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    issue: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    started_at: String,
    #[serde(default)]
    completed_at: String,
    #[serde(default)]
    attempts: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskFile {
    tasks: Vec<TaskRaw>,
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "task-runner", about = "TDD task phase runner")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bootstrap a tdd-tasks.yaml for the current work unit
    Init {
        title: String,
        #[arg(long = "crate", default_value = "")]
        krate: String,
    },
    /// pending -> red (active): write the failing test
    Red { id: String },
    /// red -> green: runs cargo nextest, advances on success
    Green { id: String },
    /// green -> refactor -> done: runs clippy + fmt + nextest
    Refactor { id: String },
    /// Print the next eligible task
    Next,
    /// Print task table
    Status,
    /// Mark a task as failed
    Fail {
        id: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    /// Close gh: issues for all done tasks
    CloseIssues,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load() -> Result<TaskFile> {
    if !Path::new(TASK_FILE).exists() {
        bail!(
            "{TASK_FILE} not found — run: task-runner.rs init \"<title>\" --crate <name>"
        );
    }
    let content = std::fs::read_to_string(TASK_FILE)
        .with_context(|| format!("reading {TASK_FILE}"))?;
    serde_yaml::from_str(&content).with_context(|| format!("parsing {TASK_FILE}"))
}

fn save(data: &TaskFile) -> Result<()> {
    let yaml = serde_yaml::to_string(data)?;
    std::fs::write(TASK_FILE, yaml).with_context(|| format!("writing {TASK_FILE}"))
}

fn now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn eligible(tasks: &[TaskRaw], task: &TaskRaw) -> bool {
    task.depends_on.iter().all(|dep| {
        tasks
            .iter()
            .find(|t| &t.id == dep)
            .map(|t| t.status == Status::Done)
            .unwrap_or(false)
    })
}

fn get_task_mut<'a>(tasks: &'a mut Vec<TaskRaw>, id: &str) -> Result<&'a mut TaskRaw> {
    tasks
        .iter_mut()
        .find(|t| t.id == id)
        .with_context(|| format!("task '{id}' not found"))
}

fn run_cargo(args: &[&str]) -> bool {
    Command::new("cargo")
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_init(title: &str, crate_name: &str) -> Result<()> {
    if Path::new(TASK_FILE).exists() {
        println!("{TASK_FILE} already exists — edit it directly to add tasks");
        return Ok(());
    }
    let scaffold = TaskFile {
        tasks: vec![TaskRaw {
            id: "t1".into(),
            title: title.into(),
            crate_name: crate_name.into(),
            test: String::new(),
            phase: Phase::Pending,
            status: Status::Pending,
            depends_on: vec![],
            issue: String::new(),
            notes: String::new(),
            started_at: String::new(),
            completed_at: String::new(),
            attempts: 0,
        }],
    };
    save(&scaffold)?;
    println!("Created {TASK_FILE} — fill in 'test:' and add more tasks, then run: task-runner.rs red t1");
    Ok(())
}

fn cmd_red(id: &str) -> Result<()> {
    let mut data = load()?;
    {
        let tasks_snapshot = data.tasks.clone();
        let task = get_task_mut(&mut data.tasks, id)?;

        if !eligible(&tasks_snapshot, task) {
            println!("Task {id} is blocked — depends_on tasks not yet done:");
            for dep in &task.depends_on {
                println!("  - {dep}");
            }
            bail!("blocked");
        }
        if task.status == Status::Done {
            println!("Task {id} is already done");
            return Ok(());
        }
        task.attempts += 1;
        if task.attempts > MAX_ATTEMPTS {
            bail!(
                "Task {id} has exceeded {MAX_ATTEMPTS} attempts — run: task-runner.rs fail {id} --reason <text>"
            );
        }
        task.phase = Phase::Red;
        task.status = Status::Active;
        task.started_at = now();

        println!("[red] {id}: {}", task.title);
        println!("  crate: {}  test: {}", task.crate_name, task.test);
        println!(
            "  Run: cargo nextest run -p {} -E 'test({})'",
            task.crate_name, task.test
        );
        println!("  Confirm it FAILS before implementing. Then run: task-runner.rs green {id}");
    }
    save(&data)
}

fn cmd_green(id: &str) -> Result<()> {
    let mut data = load()?;
    {
        let task = get_task_mut(&mut data.tasks, id)?;
        if task.phase != Phase::Red {
            bail!("Task {id} phase is {:?} — must be red before green", task.phase);
        }
        let crate_name = task.crate_name.clone();
        println!("[green] running: cargo nextest run -p {crate_name}");
        if !run_cargo(&["nextest", "run", "-p", &crate_name]) {
            bail!("Tests failed — fix implementation and retry green");
        }
        task.phase = Phase::Green;
        println!("[green] {id} confirmed. Run: task-runner.rs refactor {id}");
    }
    save(&data)
}

fn cmd_refactor(id: &str) -> Result<()> {
    let mut data = load()?;
    let crate_name;
    let title;
    {
        let task = get_task_mut(&mut data.tasks, id)?;
        if task.phase != Phase::Green {
            bail!("Task {id} phase is {:?} — must be green before refactor", task.phase);
        }
        crate_name = task.crate_name.clone();
        title = task.title.clone();
    }

    println!("[refactor] clippy...");
    if !run_cargo(&["clippy", "-p", &crate_name, "--", "-D", "warnings"]) {
        bail!("clippy failed — fix warnings");
    }

    println!("[refactor] fmt check...");
    if !run_cargo(&["fmt", "-p", &crate_name, "--", "--check"]) {
        bail!("fmt check failed — run cargo fmt");
    }

    println!("[refactor] nextest...");
    if !run_cargo(&["nextest", "run", "-p", &crate_name]) {
        bail!("tests failed after refactor");
    }

    {
        let task = get_task_mut(&mut data.tasks, id)?;
        task.phase = Phase::Done;
        task.status = Status::Done;
        task.completed_at = now();
    }

    let newly_eligible: Vec<String> = {
        let tasks = &data.tasks;
        tasks
            .iter()
            .filter(|t| t.status == Status::Pending && eligible(tasks, t))
            .map(|t| format!("    - {}: {}", t.id, t.title))
            .collect()
    };

    save(&data)?;

    println!("[done] {id}: {title}");
    println!("  Commit: git commit -m \"feat({crate_name}): {title}\"");
    if !newly_eligible.is_empty() {
        println!("  Next eligible tasks:");
        for line in &newly_eligible {
            println!("{line}");
        }
    }
    Ok(())
}

fn cmd_next() -> Result<()> {
    let data = load()?;
    let eligible_tasks: Vec<&TaskRaw> = data
        .tasks
        .iter()
        .filter(|t| t.status == Status::Pending && eligible(&data.tasks, t))
        .collect();
    match eligible_tasks.first() {
        None => println!("No eligible tasks. Either all done or all blocked."),
        Some(t) => println!("{}: {}  [crate: {}]", t.id, t.title, t.crate_name),
    }
    Ok(())
}

fn cmd_status() -> Result<()> {
    let data = load()?;
    println!("{:<6} {:<35} {:<10} {:<10} {}", "ID", "TITLE", "PHASE", "STATUS", "CRATE");
    println!("{}", "-".repeat(75));
    for t in &data.tasks {
        println!(
            "{:<6} {:<35} {:<10} {:<10} {}",
            t.id,
            if t.title.len() > 34 { &t.title[..34] } else { &t.title },
            format!("{:?}", t.phase).to_lowercase(),
            format!("{:?}", t.status).to_lowercase(),
            t.crate_name
        );
    }
    Ok(())
}

fn cmd_fail(id: &str, reason: &str) -> Result<()> {
    let mut data = load()?;
    {
        let task = get_task_mut(&mut data.tasks, id)?;
        task.status = Status::Failed;
        task.notes = reason.to_string();
        println!("[failed] {id} — {reason}");
        println!("Redesign or ask the user before retrying.");
    }
    save(&data)
}

fn cmd_close_issues() -> Result<()> {
    let data = load()?;
    let done_with_issues: Vec<&TaskRaw> = data
        .tasks
        .iter()
        .filter(|t| t.status == Status::Done && t.issue.starts_with("gh:"))
        .collect();
    if done_with_issues.is_empty() {
        println!("No done tasks with gh: issues.");
        return Ok(());
    }
    for t in done_with_issues {
        let num = t.issue.trim_start_matches("gh:");
        println!("Closing gh#{num}: {}", t.title);
        let ok = Command::new("gh")
            .args(["issue", "close", num, "--comment", &format!("Completed in task {}", t.id)])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            eprintln!("  Warning: could not close #{num}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { title, krate } => cmd_init(&title, &krate),
        Cmd::Red { id } => cmd_red(&id),
        Cmd::Green { id } => cmd_green(&id),
        Cmd::Refactor { id } => cmd_refactor(&id),
        Cmd::Next => cmd_next(),
        Cmd::Status => cmd_status(),
        Cmd::Fail { id, reason } => cmd_fail(&id, &reason),
        Cmd::CloseIssues => cmd_close_issues(),
    }
}
