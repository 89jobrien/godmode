#![allow(clippy::items_after_test_module)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use godmode_core::{
    agent, agent_index, builder, context, detect, dispatch, graph, insights, integrations,
    memory_banking, model, pipeline, plan, policy, registry, release, review, session::Session,
    skill, templates, workflow,
};

mod commands;

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

    /// Emit SARIF v2.1.0 output (verify and review commands).
    #[arg(long, global = true)]
    sarif: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print triage summary at session start.
    Handon {
        /// Emit a single-line summary instead of the full triage.
        #[arg(long)]
        compact: bool,
    },

    /// Validate session state at session end.
    Handoff,

    /// Session file management (pruning, etc.).
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

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

    /// Emit full session context for hooks and subagents.
    Context,

    /// Show graph counts and next runnable task(s) — fast mid-session state check.
    Status {
        /// Emit the old single-line summary instead of the sectioned view.
        #[arg(long)]
        compact: bool,
    },

    /// Agent operations: list installed agents, generate index, or dispatch a plan.
    Agent {
        #[command(subcommand)]
        action: AgentAction,
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

    /// Interactive or file-driven task graph construction.
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },

    /// Hook observability: list, log, and test hooks.
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },

    /// Skill registry management.
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },

    /// Plugin conformance and consistency auditing.
    Review {
        #[command(subcommand)]
        action: ReviewAction,
    },

    /// Plugin release: bump version, tag, push.
    Release {
        #[command(subcommand)]
        action: ReleaseAction,
    },

    /// Workflow DAG execution per agent.
    Workflow {
        #[command(subcommand)]
        action: WorkflowAction,
    },

    /// Render the task graph as DOT or SVG.
    VisualizeGraph {
        /// Output format: dot or svg.
        #[arg(long, default_value = "dot")]
        format: String,
        /// Write output to this file instead of stdout.
        #[arg(long)]
        out: Option<String>,
    },

    /// Memory banking: persistent source-backed project context.
    MemoryBanking {
        #[command(subcommand)]
        action: MemoryBankingAction,
    },

    /// Insight capture and retrieval (append-only JSONL).
    Insight {
        #[command(subcommand)]
        action: InsightAction,
    },

    /// Pipeline execution: list, start, advance, and status multi-step pipelines.
    Pipeline {
        #[command(subcommand)]
        action: PipelineAction,
    },

    /// Governance policy management: resolve, check, list, audit.
    Policy {
        #[command(subcommand)]
        action: PolicyCmdAction,
    },

    /// Pin the session to a specific repo root path.
    Pin {
        /// Path to pin (defaults to current directory).
        path: Option<String>,
    },

    /// Remove the pinned root from the session.
    Unpin,

    /// First-time setup: create global config and project state dirs.
    Init,

    /// Validate environment: required tools, 1Password auth, worktrees.
    Doctor,

    /// Generate a test module stub for a crate and testing dimension.
    Scaffold {
        /// Crate name (e.g. godmode-core).
        crate_name: String,
        /// Testing dimension: unit, property, fuzz, conformance, integration, regression.
        dimension: String,
    },

    /// Check whether a Rust source file has associated tests.
    TestCheck {
        /// Path to the .rs file to check.
        path: String,
    },
}

#[derive(Subcommand)]
enum AgentAction {
    /// List available agents (table or JSON).
    List {
        /// Filter by name or description keyword (case-insensitive).
        #[arg(long)]
        filter: Option<String>,
    },
    /// Regenerate agents/INDEX.md.
    Index,
    /// Ingest a plan file and emit an orca-strait dispatch payload.
    Dispatch {
        /// Path to the plan markdown file.
        path: String,
        /// Maximum concurrent agent chains.
        #[arg(long, default_value = "5")]
        max: usize,
    },
    /// Generate .md from agent YAML definitions.
    Generate {
        /// Name of a single agent YAML to generate (stem, no extension). Omit for --all.
        name: Option<String>,
        /// Generate .md for all agents/*.yaml files.
        #[arg(long)]
        all: bool,
    },
    /// Migrate agents/*.md frontmatter to agents/*.yaml stubs.
    Migrate {
        /// Name of a single agent .md to migrate (stem, no extension). Omit for --all.
        name: Option<String>,
        /// Migrate all agents/*.md files.
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// List all registered skills.
    List,
    /// Install a skill from a local directory path.
    Install {
        /// Absolute path to the skill directory.
        path: String,
    },
    /// Remove a skill from the registry by name.
    Uninstall {
        /// Skill name to remove.
        name: String,
    },
}

#[derive(Subcommand)]
enum ReviewAction {
    /// Run all conformance checks (skills + agents + plugin.json).
    #[command(name = "self")]
    Self_,
    /// Check skill dirs for SKILL.md, frontmatter, and link integrity.
    Skills,
    /// Check agent frontmatter completeness.
    Agents,
}

#[derive(Subcommand)]
enum ReleaseAction {
    /// Show current plugin version.
    Current,
    /// Increment patch version in all files listed in .version-bump.json.
    Bump {
        /// Set an explicit version instead of auto-incrementing.
        #[arg(long)]
        version: Option<String>,
    },
    /// Create annotated git tag for the current version.
    Tag,
    /// Push current branch and version tag to origin.
    Push,
    /// Generate and prepend a changelog entry from commits since last tag.
    Changelog,
    /// Cross-check plugin.json, Cargo.toml, and git tag versions.
    Validate,
}

#[derive(Subcommand)]
enum HookAction {
    /// List all hooks registered in hooks/hooks.json.
    List,
    /// Print the last N lines from .ctx/godmode/traces/hooks.log.
    Log {
        /// Number of lines to show (default 20).
        #[arg(long, default_value = "20")]
        tail: usize,
    },
    /// Run a hook script with synthetic stdin JSON and show exit code + stderr.
    Test {
        /// Path to the hook script to test.
        script: String,
    },
    /// Run all numbered migration scripts in hooks/migrations/.
    Migrate,
    /// Run a built-in hook by name (Rust implementation).
    Run {
        /// Hook name: stop-guard, auto-block, pre-commit, quality-gate.
        name: String,
    },
}

#[derive(Subcommand)]
enum GraphAction {
    /// Build a task graph interactively or from a template file.
    Build {
        /// Path to a template YAML file (non-interactive mode).
        #[arg(long)]
        input: Option<String>,
        /// Variable substitutions in key=value format (used with --input).
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
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
    List {
        /// Filter to tasks with a specific priority (high, normal, low).
        #[arg(long, value_name = "LEVEL")]
        priority: Option<model::Priority>,
        /// Case-insensitive keyword filter on title, crate_name, and notes.
        #[arg(long)]
        filter: Option<String>,
    },

    /// Add a new task. Omit ID to auto-assign the next available "tN" slot.
    Add {
        /// Task title (required).
        title: String,
        /// Task ID (e.g. t5). Auto-assigned if omitted.
        #[arg(long)]
        id: Option<String>,
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
    Next {
        /// Filter to runnable tasks with a specific priority (high, normal, low).
        #[arg(long, value_name = "LEVEL")]
        priority: Option<model::Priority>,
    },

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
enum SessionAction {
    /// Delete session JSONL files older than N days.
    Prune {
        /// Delete files older than this many days.
        #[arg(long, value_name = "DAYS")]
        older_than: u64,
        /// Print what would be deleted without removing anything.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum PlanAction {
    /// Parse a plan markdown file and populate the task graph.
    Ingest {
        /// Path to the plan markdown file.
        path: String,
    },
}

#[derive(Subcommand)]
enum WorkflowAction {
    /// Execute a workflow DAG for an agent.
    Run {
        /// Agent name (used to locate the workflow in agents/<agent>.yaml).
        agent: String,
        /// Workflow name (must match a workflows[] entry in the agent YAML).
        workflow: String,
    },
    /// List workflows for an agent (or all agents).
    List {
        /// Filter to a specific agent name.
        #[arg(long)]
        agent: Option<String>,
    },
    /// Show current state of a named workflow from .ctx/workflow-<name>.json.
    Status {
        /// Workflow name.
        name: String,
    },
}

#[derive(Subcommand)]
enum MemoryBankingAction {
    /// Print memory-bank contents for context injection (SessionStart hook).
    Inject,
    /// Print update reminder if session had commits (Stop hook).
    Remind,
    /// Create .ctx/memory-banking/ with empty template files.
    Init,
    /// Show memory-banking status and staleness.
    Status,
}

#[derive(Subcommand)]
enum InsightAction {
    /// Record a new insight.
    Add {
        /// Insight title (short heading).
        title: String,
        /// Insight body text.
        #[arg(long)]
        body: String,
        /// Optional tags (comma-separated).
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// List recorded insights.
    List {
        /// Filter to a specific date (YYYY-MM-DD). Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Render insights to `.ctx/insights-YYYY-MM-DD.md`.
    Render {
        /// Date to render (YYYY-MM-DD). Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
}

#[derive(Subcommand)]
enum PipelineAction {
    /// List all available pipelines with name and description.
    List,
    /// Show steps for a pipeline, marking current position if active.
    Show {
        /// Pipeline name (stem of the YAML file in pipelines/).
        name: String,
    },
    /// Start a pipeline, optionally from a named entry-point skill.
    Start {
        /// Pipeline name to activate.
        name: String,
        /// Entry-point skill to start from (must be a valid entry_point).
        #[arg(long)]
        from: Option<String>,
    },
    /// Mark current step done and advance to the next step.
    Next,
    /// Skip current step and advance without marking it done.
    Skip,
    /// Deactivate the current pipeline (clear saved state).
    Stop,
    /// Show the active pipeline name, current step, and progress.
    Status,
    /// Run a pipeline headlessly — walk task graph, execute run: fields.
    Run {
        /// Pipeline name to run.
        name: String,
        /// Skill to start from (must be a valid entry_point).
        #[arg(long)]
        from: Option<String>,
        /// Stop on first task failure.
        #[arg(long)]
        fail_fast: bool,
    },
}

#[derive(Subcommand)]
enum PolicyCmdAction {
    /// Resolve the effective policy for an agent.
    Resolve {
        /// Agent name (matches agents/cfg/<name>.cfg.yaml).
        agent: String,
        /// Governance level override (open/standard/strict/locked).
        #[arg(long)]
        level: Option<String>,
    },
    /// Check if a tool call is allowed by an agent's policy.
    Check {
        /// Agent name.
        agent: String,
        /// Tool name to check (Read, Write, Edit, Bash, Glob, Grep, Agent).
        tool: String,
        /// Content to check against blocked patterns.
        #[arg(long)]
        input: Option<String>,
        /// Governance level override.
        #[arg(long)]
        level: Option<String>,
    },
    /// List all available policies (default, categories, levels).
    List,
    /// Show governance audit trail.
    Audit {
        /// Filter to a specific date (YYYY-MM-DD). Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
}

/// Filter a task slice by optional priority and optional keyword.
///
/// Priority filtering is applied first, then keyword filtering (case-insensitive
/// substring match on title, crate_name, and notes).
fn filter_tasks<'a>(
    tasks: &'a [model::Task],
    priority: Option<&model::Priority>,
    keyword: Option<&str>,
) -> Vec<&'a model::Task> {
    let mut result: Vec<&model::Task> = match priority {
        None => tasks.iter().collect(),
        Some(p) => tasks.iter().filter(|t| &t.priority == p).collect(),
    };
    if let Some(kw) = keyword {
        let kw_lower = kw.to_lowercase();
        result.retain(|t| {
            t.title.to_lowercase().contains(&kw_lower)
                || t.crate_name
                    .as_deref()
                    .is_some_and(|c| c.to_lowercase().contains(&kw_lower))
                || t.notes.to_lowercase().contains(&kw_lower)
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, title: &str, crate_name: Option<&str>, notes: &str) -> model::Task {
        model::Task {
            id: id.to_string(),
            title: title.to_string(),
            status: model::Status::Pending,
            depends_on: vec![],
            notes: notes.to_string(),
            crate_name: crate_name.map(|s| s.to_string()),
            commit: None,
            completed: None,
            priority: model::Priority::Normal,
            run: None,
            started_at: None,
            completed_at: None,
            tags: vec![],
        }
    }

    #[test]
    fn filter_tasks_no_filters_returns_all() {
        let tasks = vec![make_task("t1", "Alpha", None, "")];
        let result = filter_tasks(&tasks, None, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_tasks_keyword_matches_title() {
        let tasks = vec![
            make_task("t1", "Add logging", None, ""),
            make_task("t2", "Fix bug", None, ""),
        ];
        let result = filter_tasks(&tasks, None, Some("log"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "t1");
    }

    #[test]
    fn filter_tasks_keyword_case_insensitive() {
        let tasks = vec![make_task("t1", "Add LOGGING", None, "")];
        let result = filter_tasks(&tasks, None, Some("logging"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_tasks_keyword_matches_crate_name() {
        let tasks = vec![make_task("t1", "Something", Some("godmode-core"), "")];
        let result = filter_tasks(&tasks, None, Some("core"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_tasks_keyword_matches_notes() {
        let tasks = vec![make_task("t1", "Task", None, "needs review before merge")];
        let result = filter_tasks(&tasks, None, Some("review"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_tasks_keyword_no_match() {
        let tasks = vec![make_task("t1", "Alpha", None, "beta")];
        let result = filter_tasks(&tasks, None, Some("gamma"));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn filter_tasks_priority_and_keyword_combined() {
        let tasks = vec![
            make_task("t1", "Add logging", None, ""),
            make_task("t2", "Add metrics", None, ""),
        ];
        // t1 is Normal priority (default), filter by High should exclude both
        let result = filter_tasks(&tasks, Some(&model::Priority::High), Some("add"));
        assert_eq!(result.len(), 0);
    }
}

/// Exit with code 2 for empty result sets. Code 1 is reserved for actual errors.
/// Convention: 0 = success with results, 1 = error, 2 = success but empty.
fn exit_empty(json: bool) -> ! {
    if json {
        println!("[]");
    } else {
        println!("No results.");
    }
    std::process::exit(2);
}

fn main() -> miette::Result<()> {
    install_miette_theme();
    match run() {
        Ok(()) => Ok(()),
        Err(err) => match err.downcast::<templates::TemplateError>() {
            Ok(err) => Err(miette::Report::new(err)),
            Err(err) => Err(miette::miette!("{err:#}")),
        },
    }
}

fn install_miette_theme() {
    let handler = miette::MietteHandlerOpts::new()
        .graphical_theme(miette::GraphicalTheme::unicode())
        .color(true);
    let _ = miette::set_hook(Box::new(move |_| Box::new(handler.clone().build())));
}

// TODO(rustqual,#86): split run() into per-subcommand handlers in a commands/ module.
// cognitive_complexity=697, cyclomatic_complexity=258, long_function≈1500 lines, nesting_depth=6.
// Each subcommand (task, plan, dispatch, wave, worktree, …) should become its own
// `commands/<name>.rs` file with a single `pub fn handle(args, root) -> Result<()>` entry point.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let json = cli.json;
    let sarif = cli.sarif;
    let root = detect::root_or_cwd()?;

    match cli.cmd {
        Cmd::Handon { compact } => {
            let out = integrations::handon(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else if compact {
                let g = &out.graph;
                println!(
                    "godmode: {}D {}R {}P {}B",
                    g.done, g.running, g.pending, g.blocked
                );
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

        Cmd::Session { action } => match action {
            SessionAction::Prune {
                older_than,
                dry_run,
            } => {
                use godmode_core::integrations::crux;
                use godmode_core::session::prune_sessions_older_than;
                let sessions_dir = crux::sessions_dir(&root);
                let pruned = prune_sessions_older_than(&sessions_dir, older_than, dry_run)?;
                if json {
                    let paths: Vec<String> =
                        pruned.iter().map(|p| p.display().to_string()).collect();
                    println!("{}", serde_json::to_string_pretty(&paths)?);
                } else if pruned.is_empty() {
                    println!("No session files to prune.");
                } else if dry_run {
                    println!("{} file(s) would be deleted.", pruned.len());
                } else {
                    println!("Pruned {} session file(s).", pruned.len());
                }
                Ok(())
            }
        },

        Cmd::Task { action } => {
            commands::run_task_action(&root, json, action)?;
            Ok(())
        }

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

        Cmd::Context => {
            let ctx = context::build(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&ctx)?);
            } else {
                println!("project: {}", ctx.project);
                if ctx.running.is_empty() {
                    println!("running: (none)");
                } else {
                    for t in &ctx.running {
                        let crate_info = t
                            .crate_name
                            .as_deref()
                            .map(|c| format!(" [{}]", c))
                            .unwrap_or_default();
                        println!("running: {} — {}{}", t.id, t.title, crate_info);
                    }
                }
                println!("pending: {}", ctx.pending_count);
                if !ctx.blocked.is_empty() {
                    for b in &ctx.blocked {
                        println!("blocked: {} — {}", b.id, b.reason);
                    }
                }
                println!("critical path: {} tasks deep", ctx.critical_path_depth);
                if !ctx.recent_commits.is_empty() {
                    println!("recent:");
                    for c in &ctx.recent_commits {
                        println!("  {c}");
                    }
                }
            }
            Ok(())
        }

        Cmd::Status { compact } => {
            let g = graph::load(&root)?;
            let summary = g.summary();
            let next = graph::runnable(&g);
            let critical = dispatch::critical_path(&g);
            let blocked_tasks: Vec<&model::Task> = g
                .tasks
                .iter()
                .filter(|t| t.status == model::Status::Blocked)
                .collect();
            if json {
                let blocked_detail: Vec<serde_json::Value> = blocked_tasks
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "title": t.title,
                            "reason": t.notes,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "done": summary.done,
                        "running": summary.running,
                        "pending": summary.pending,
                        "blocked": summary.blocked,
                        "blocked_detail": blocked_detail,
                        "next": next.iter().map(|t| &t.id).collect::<Vec<_>>(),
                        "critical_depth": critical.len(),
                    }))?
                );
            } else if compact {
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
            } else {
                println!("=== godmode status ===");
                println!("  done     {}", summary.done);
                println!("  running  {}", summary.running);
                println!("  pending  {}", summary.pending);
                if blocked_tasks.is_empty() {
                    println!("  blocked  {}", summary.blocked);
                } else {
                    let blocked_inline: Vec<String> = blocked_tasks
                        .iter()
                        .map(|t| {
                            if t.notes.is_empty() {
                                format!("{}: (no reason)", t.id)
                            } else {
                                format!("{}: {}", t.id, t.notes)
                            }
                        })
                        .collect();
                    println!(
                        "  blocked  {}  [{}]",
                        summary.blocked,
                        blocked_inline.join(", ")
                    );
                }
                println!();
                if !critical.is_empty() {
                    let path_str: Vec<&str> = critical.iter().map(|t| t.id.as_str()).collect();
                    println!(
                        "  critical path ({} tasks): {}",
                        critical.len(),
                        path_str.join(" -> ")
                    );
                }
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

        Cmd::Hook { action } => {
            commands::run_hook_action(&root, json, action)?;
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

        Cmd::Agent { action } => match action {
            AgentAction::List { filter } => {
                let mut agents = agent_index::list_agents(&root)?;
                if let Some(kw) = &filter {
                    agents = agent_index::filter_agents(agents, kw);
                }
                // Always regenerate INDEX.md
                agent_index::generate_agent_index(&root, &agents)?;
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
                let agents = agent_index::list_agents(&root)?;
                agent_index::generate_agent_index(&root, &agents)?;
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
            let g = graph::load(&root)?;
            let dot = graph::to_dot(&g);
            let content = match format.as_str() {
                "dot" => dot,
                "svg" => {
                    // Try piping DOT through `dot -Tsvg`; degrade gracefully if missing.
                    match std::process::Command::new("dot")
                        .args(["-Tsvg"])
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::inherit())
                        .spawn()
                    {
                        Ok(mut child) => {
                            use std::io::Write;
                            if let Some(stdin) = child.stdin.take() {
                                let mut stdin = stdin;
                                let _ = stdin.write_all(dot.as_bytes());
                            }
                            let output = child.wait_with_output()?;
                            if !output.status.success() {
                                anyhow::bail!("graphviz dot exited with {}", output.status);
                            }
                            String::from_utf8_lossy(&output.stdout).into_owned()
                        }
                        Err(_) => {
                            eprintln!(
                                "warning: graphviz `dot` not found — falling back to DOT format"
                            );
                            dot
                        }
                    }
                }
                other => anyhow::bail!("unsupported format '{other}'; expected dot or svg"),
            };
            if let Some(path) = out {
                std::fs::write(&path, &content)?;
                if !json {
                    println!("wrote {path}");
                } else {
                    println!("{}", serde_json::json!({"path": path}));
                }
            } else {
                print!("{content}");
            }
            Ok(())
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
