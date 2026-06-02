# Plan: godmode verify, wave, worktree, ci, issue subcommands

**Status**: done

## Goal

Extract repeated skill-side shell logic into godmode CLI subcommands so skills reference
`godmode <cmd>` instead of raw git/cargo/gh invocations. Four new modules plus gh extension.

## Architecture

- Crates affected: `godmode-core`, `godmode-cli`
- New modules: `verify`, `wave`, `worktree`; extend `integrations::gh`
- State files: `.ctx/wave-status.json` (wave), `.worktrees/<branch>/` (worktree)
- No new Cargo dependencies — all subprocess via `std::process::Command`

## Tech Stack

- Rust edition 2024, existing deps: `anyhow`, `serde`, `serde_json`, `clap`

## Tasks

### Task 1: godmode-core::verify module

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/verify.rs`
**Run**: `cargo nextest run -p godmode-core -E 'test(verify)'`

Implement `verify::run(root: &Path, crate_name: Option<&str>) -> Result<VerifyReport>`.

`VerifyReport`:

```rust
#[derive(Debug, serde::Serialize)]
pub struct VerifyReport {
    pub nextest: StepResult,
    pub clippy: StepResult,
    pub fmt: StepResult,
    pub commits: StepResult,
    pub passed: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct StepResult {
    pub ok: bool,
    pub output: String,
}
```

Each step runs a subprocess, captures stdout+stderr, sets `ok` from exit code.

Steps:

1. nextest: `cargo nextest run [-p <crate>] [--workspace if no crate]`
2. clippy: `cargo clippy [-p <crate>] [--workspace] -- -D warnings`
3. fmt: `cargo fmt --all --check`
4. commits: `git -C <root> log --oneline -3` — ok if output non-empty

`passed = nextest.ok && clippy.ok && fmt.ok && commits.ok`

Tests:

```rust
#[test]
fn verify_report_serialises() {
    let r = VerifyReport {
        nextest: StepResult { ok: true, output: "ok".into() },
        clippy: StepResult { ok: true, output: "".into() },
        fmt: StepResult { ok: true, output: "".into() },
        commits: StepResult { ok: true, output: "abc1234 feat: x".into() },
        passed: true,
    };
    let j = serde_json::to_string(&r).unwrap();
    assert!(j.contains("\"passed\":true"));
}

#[test]
fn verify_report_failed_when_any_step_fails() {
    let r = VerifyReport {
        nextest: StepResult { ok: false, output: "FAILED".into() },
        clippy: StepResult { ok: true, output: "".into() },
        fmt: StepResult { ok: true, output: "".into() },
        commits: StepResult { ok: true, output: "abc".into() },
        passed: false,
    };
    assert!(!r.passed);
}
```

### Task 2: godmode-core::wave module

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/wave.rs`
**Run**: `cargo nextest run -p godmode-core -E 'test(wave)'`

State file path: `<root>/.ctx/wave-status.json`

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SlotStatus { Pending, Done, Blocked }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSlot {
    pub status: SlotStatus,
    pub branch: String,
    pub commits: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WaveState {
    pub wave: u32,
    pub agents: std::collections::BTreeMap<String, AgentSlot>,
}
```

Functions:

- `init(root, wave_n, agents: &[&str]) -> Result<WaveState>` — writes fresh state file
- `load(root) -> Result<WaveState>` — reads `.ctx/wave-status.json`
- `save(root, &WaveState) -> Result<()>`
- `mark_done(root, agent: &str, commits: Vec<String>) -> Result<()>`
- `mark_blocked(root, agent: &str) -> Result<()>`
- `check(state: &WaveState) -> bool` — true if all slots done or blocked, none pending
- `all_done(state: &WaveState) -> bool` — true if all slots done (none blocked)

Tests:

```rust
#[test]
fn wave_init_creates_pending_slots() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".ctx")).unwrap();
    let state = init(dir.path(), 1, &["crate-a", "crate-b"]).unwrap();
    assert_eq!(state.agents.len(), 2);
    assert_eq!(state.agents["crate-a"].status, SlotStatus::Pending);
}

#[test]
fn wave_mark_done_updates_slot() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".ctx")).unwrap();
    init(dir.path(), 1, &["crate-a"]).unwrap();
    mark_done(dir.path(), "crate-a", vec!["abc1234".into()]).unwrap();
    let state = load(dir.path()).unwrap();
    assert_eq!(state.agents["crate-a"].status, SlotStatus::Done);
    assert_eq!(state.agents["crate-a"].commits, vec!["abc1234"]);
}

#[test]
fn wave_check_false_while_pending() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".ctx")).unwrap();
    let state = init(dir.path(), 1, &["crate-a", "crate-b"]).unwrap();
    assert!(!check(&state));
}

#[test]
fn wave_check_true_when_all_settled() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".ctx")).unwrap();
    init(dir.path(), 1, &["crate-a"]).unwrap();
    mark_done(dir.path(), "crate-a", vec!["abc".into()]).unwrap();
    let state = load(dir.path()).unwrap();
    assert!(check(&state));
}
```

### Task 3: godmode-core::worktree module

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/worktree.rs`
**Run**: `cargo nextest run -p godmode-core -E 'test(worktree)'`

```rust
pub struct WorktreeInfo {
    pub branch: String,
    pub path: std::path::PathBuf,
    pub issue_number: Option<u64>,
}
```

Functions:

- `add(root, branch: &str, issue_number: Option<u64>) -> Result<WorktreeInfo>`
  1. Ensures `.worktrees/` dir exists under root
  2. Ensures `.worktrees/` is in `.gitignore` (appends if missing)
  3. Runs `git -C <root> fetch origin main`
  4. Runs `git -C <root> worktree add .worktrees/<branch> -b <branch>`
  5. Returns `WorktreeInfo`

- `remove(root, branch: &str) -> Result<()>`
  1. Verifies branch is merged: `git -C <root> log --oneline main..<branch>` must be empty
  2. Runs `git -C <root> worktree remove .worktrees/<branch>`
  3. Runs `git -C <root> branch -d <branch>`

- `gitignore_contains(root, entry: &str) -> bool` — pure helper, reads `.gitignore`
- `ensure_gitignore(root, entry: &str) -> Result<()>` — appends if not present

Tests:

```rust
#[test]
fn gitignore_contains_detects_entry() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), ".worktrees/\n.ctx/\n").unwrap();
    assert!(gitignore_contains(dir.path(), ".worktrees/"));
    assert!(!gitignore_contains(dir.path(), ".env"));
}

#[test]
fn ensure_gitignore_appends_missing_entry() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), ".ctx/\n").unwrap();
    ensure_gitignore(dir.path(), ".worktrees/").unwrap();
    let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(content.contains(".worktrees/"));
}

#[test]
fn ensure_gitignore_no_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), ".worktrees/\n").unwrap();
    ensure_gitignore(dir.path(), ".worktrees/").unwrap();
    let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert_eq!(content.matches(".worktrees/").count(), 1);
}
```

### Task 4: extend integrations::gh — ci_triage and issue_close

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/integrations/gh.rs`
**Run**: `cargo nextest run -p godmode-core -E 'test(gh)'`

Add to `gh.rs`:

```rust
#[derive(Debug, serde::Serialize, PartialEq)]
pub enum CiFailureClass {
    CompileError,
    TestFailure,
    ClippyWarning,
    FmtCheck,
    PreCommitHook,
    RunnerEnvironment,
    FalsePositiveDetection,
    DependencyIssue,
    Unknown,
}

#[derive(Debug, serde::Serialize)]
pub struct CiTriageResult {
    pub run_id: String,
    pub class: CiFailureClass,
    pub fix_hint: String,
    pub raw_snippet: String, // first 20 lines of failure log
}
```

- `classify_log(log: &str) -> CiFailureClass` — pure pattern match:
  - contains `"error[E"` or `"missing match arm"` → `CompileError`
  - contains `"FAILED"` and `"assertion"` or `"panicked"` → `TestFailure`
  - contains `"error: "` and `"-D warnings"` → `ClippyWarning`
  - contains `"Diff in "` → `FmtCheck`
  - contains `"gitleaks"` or `"obfsck"` or `"coursers"` → `PreCommitHook`
  - contains `"No such file or directory"` or `"xcode"` or `"wrong target"` → `RunnerEnvironment`
  - contains `"secret"` and `"false positive"` → `FalsePositiveDetection`
  - contains `"lockfile"` or `"yanked"` or `"version conflict"` → `DependencyIssue`
  - else → `Unknown`

- `fix_hint(class: &CiFailureClass) -> &'static str` — returns one-line fix string per class

- `ci_triage(run_id: Option<&str>) -> Result<CiTriageResult>` — shells out:
  1. If `run_id` is None: `gh run list --limit 1 --status failure --json databaseId`
     to find most recent failure ID
  2. `gh run view <id> --log-failed` → captures stdout
  3. Calls `classify_log` on output
  4. Returns `CiTriageResult`

- `issue_close(number: u64, repo: Option<&str>, commit_sha: &str) -> Result<()>`
  - Runs `gh issue close <N> [--repo <r>] --comment "Implemented in <sha>."`

Tests:

```rust
#[test]
fn classify_compile_error() {
    assert_eq!(classify_log("error[E0308]: mismatched types"), CiFailureClass::CompileError);
}

#[test]
fn classify_test_failure() {
    assert_eq!(classify_log("FAILED\nassertion failed: x == y"), CiFailureClass::TestFailure);
}

#[test]
fn classify_clippy() {
    assert_eq!(classify_log("error: unused variable\n-D warnings"), CiFailureClass::ClippyWarning);
}

#[test]
fn classify_fmt() {
    assert_eq!(classify_log("Diff in src/lib.rs"), CiFailureClass::FmtCheck);
}

#[test]
fn classify_unknown() {
    assert_eq!(classify_log("something completely unrecognised"), CiFailureClass::Unknown);
}
```

### Task 5: expose new modules in godmode-core lib.rs

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/lib.rs`
**Run**: `cargo check -p godmode-core`

Add:

```rust
pub mod verify;
pub mod wave;
pub mod worktree;
```

Existing modules stay unchanged.

### Task 6: CLI subcommands — verify, wave, worktree, ci, issue

**Crate**: `godmode-cli`
**File(s)**: `crates/godmode-cli/src/main.rs`
**Run**: `cargo nextest run -p godmode-core && cargo check -p godmode-cli`

Add to `Cmd` enum:

```rust
/// Run verification gate: nextest + clippy + fmt + commits.
Verify {
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
```

```rust
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
    /// Remove a worktree after verifying its branch is merged.
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
    /// List open GitHub issues (mirrors task pull --github but standalone).
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
```

Match arms: delegate to `verify::run`, `wave::*`, `worktree::*`, `gh::ci_triage`,
`gh::pull_issues`, `gh::issue_close`. Human output + `--json` on all.

`Verify` exits 1 if `report.passed == false`.
`Wave Check` exits 1 if `!wave::check(&state)`.
`Worktree Remove` exits 1 if branch not fully merged.
