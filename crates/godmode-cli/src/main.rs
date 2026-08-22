#![allow(clippy::items_after_test_module)]

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use godmode_core::{detect, model, templates};

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
    if std::env::args().nth(1).as_deref() == Some("completions") {
        clap_complete::generate(
            clap_complete_nushell::Nushell,
            &mut Cli::command(),
            "godmode",
            &mut std::io::stdout(),
        );
        return Ok(());
    }
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

/// Thin dispatcher: parse the CLI, resolve the repo root, and hand off to the
/// matching handler in `commands::`.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let json = cli.json;
    let sarif = cli.sarif;

    let root = detect::root_or_cwd()?;

    match cli.cmd {
        Cmd::Handon { compact } => commands::run_handon(&root, json, compact),
        Cmd::Handoff => commands::run_handoff(&root, json),
        Cmd::Session { action } => commands::run_session_action(&root, json, action),
        Cmd::Task { action } => commands::run_task_action(&root, json, action),
        Cmd::Plan { action } => commands::run_plan_action(&root, json, action),
        Cmd::Context => commands::run_context(&root, json),
        Cmd::Status { compact } => commands::run_status(&root, json, compact),
        Cmd::Hook { action } => commands::run_hook_action(&root, json, action),
        Cmd::Dispatch { max, critical_path } => {
            commands::run_dispatch(&root, json, max, critical_path)
        }
        Cmd::Agent { action } => commands::run_agent_action(&root, json, action),
        Cmd::Verify { crate_name } => commands::run_verify(&root, json, sarif, crate_name),
        Cmd::Wave { action } => commands::run_wave_action(&root, json, action),
        Cmd::Worktree { action } => commands::run_worktree_action(&root, json, action),
        Cmd::Ci { action } => commands::run_ci_action(json, action),
        Cmd::Issue { action } => commands::run_issue_action(json, action),
        Cmd::Graph { action } => commands::run_graph_action(&root, json, action),
        Cmd::Skill { action } => commands::run_skill_action(&root, json, action),
        Cmd::Review { action } => commands::run_review_action(&root, json, sarif, action),
        Cmd::Release { action } => commands::run_release_action(&root, json, action),
        Cmd::Workflow { action } => commands::run_workflow_action(&root, json, action),
        Cmd::VisualizeGraph { format, out } => {
            commands::run_visualize_graph(&root, json, format, out)
        }
        Cmd::MemoryBanking { action } => commands::run_memory_banking_action(&root, json, action),
        Cmd::Insight { action } => commands::run_insight_action(&root, json, action),
        Cmd::Pipeline { action } => commands::run_pipeline_action(&root, json, action),
        Cmd::Policy { action } => commands::run_policy_action(&root, json, action),
        Cmd::Pin { path } => commands::run_pin(&root, json, path),
        Cmd::Unpin => commands::run_unpin(&root, json),
        Cmd::Init => commands::run_init_cmd(json),
        Cmd::Doctor => commands::run_doctor_cmd(json),
        Cmd::Scaffold {
            crate_name,
            dimension,
        } => commands::run_scaffold(crate_name, dimension),
        Cmd::TestCheck { path } => commands::run_test_check(json, path),
    }
}
