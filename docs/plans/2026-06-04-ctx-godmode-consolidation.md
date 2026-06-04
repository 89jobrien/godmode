# Plan: Consolidate all state paths under `.ctx/godmode/`

## Goal

Move all godmode-owned state files from scattered `.ctx/` locations into a
single `.ctx/godmode/` namespace, with subdirectories for traces, sessions,
reports, and memory-bank.

## Architecture

- Crates affected: `godmode-core` (path helpers, tests), `godmode-cli` (doc
  comments only)
- New types: none — only path constants change
- Data flow: every module that builds a `.ctx/X` path switches to
  `.ctx/godmode/X`

### Target directory tree

```
.ctx/godmode/
  tasks.yaml              # was GODMODE.tasks.yaml
  pipeline.yaml           # was GODMODE.pipeline.yaml
  wave-status.json        # was wave-status.json
  session.json            # was GODMODE.session.json
  .initialized            # was .initialized
  workflow-*.json          # was workflow-*.json
  traces/
    trace.jsonl            # was GODMODE.trace.jsonl
    hooks.log              # was GODMODE.hooks.log
    activity.jsonl         # was traces/activity.jsonl
    insights.jsonl         # was insights.jsonl
  sessions/
    YYYY-MM-DD.jsonl       # unchanged structure
    YYYY-MM-DD-summary.jsonl
  reports/
    introspection-*.md     # was .ctx/introspection-*.md
    reflect-*.md           # was .ctx/reflect-*.md
    insights-*.md          # was .ctx/insights-*.md
    governance-audit-*.jsonl
    code-review-*.md       # new: standardized output
    doublecheck-*.md       # new: standardized output
    release-notes-*.md     # new: standardized output
  memory-bank/
    health-history.jsonl   # already here
    patterns.md            # already here
    mistakes.md
    project-brief.md
    product-context.md
    tech-context.md
    system-patterns.md
    active-context.md
    progress.md
  _WORKING_DIR/            # scratch space (unchanged structure)
```

**Untouched**: `.ctx/HANDOFF.*.yaml` — owned by `hj`/atelier, not godmode.

## Tech Stack

- Rust 2024 edition
- No new dependencies

## Risk

- `.ctx/godmode/` already created by `godmode init` — existing installs have
  both `.ctx/GODMODE.tasks.yaml` and `.ctx/godmode/tasks.yaml`. Task 1 adds
  a migration fallback.
- 20+ hook scripts and skills reference old paths via string literals in
  comments and Nushell code — breakage is silent (file-not-found → graceful
  degradation). Must grep-verify after the change.

## Tasks

### Task 1: Add `ctx_dir()` helper and migrate `graph.rs`

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/graph.rs`
**Run**: `cargo nextest run -p godmode-core -- graph`

1. Write failing test:

   ```rust
   #[test]
   fn task_file_uses_godmode_subdir() {
       let root = Path::new("/tmp/test-root");
       let path = task_file(root);
       assert_eq!(path, root.join(".ctx/godmode/tasks.yaml"));
   }
   ```

   Run: `cargo nextest run -p godmode-core -- task_file_uses_godmode_subdir`
   Expected: FAIL (currently returns `.ctx/GODMODE.tasks.yaml`)

2. Implement — update `task_file()`:

   ```rust
   pub fn task_file(root: &Path) -> PathBuf {
       let new = root.join(".ctx").join("godmode").join("tasks.yaml");
       if new.exists() {
           return new;
       }
       let legacy = root.join(".ctx").join("GODMODE.tasks.yaml");
       if legacy.exists() {
           return legacy;
       }
       new
   }
   ```

3. Update `save()` to ensure `.ctx/godmode/` exists:

   ```rust
   pub fn save(root: &Path, graph: &TaskGraph) -> Result<()> {
       let path = task_file(root);
       if let Some(parent) = path.parent() {
           std::fs::create_dir_all(parent)
               .context("create .ctx/godmode directory")?;
       }
       // ... rest unchanged
   ```

4. Verify:

   ```
   cargo nextest run -p godmode-core -- graph  → all green
   cargo clippy -p godmode-core -- -D warnings → zero warnings
   ```

5. Run: `git branch --show-current`
   Verify output matches the expected branch. Stop immediately if not.
   Commit: `feat(graph): migrate task file to .ctx/godmode/tasks.yaml`

### Task 2: Migrate `wave.rs` path

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/wave.rs`
**Run**: `cargo nextest run -p godmode-core -- wave`

1. Write failing test:

   ```rust
   #[test]
   fn state_path_uses_godmode_subdir() {
       let root = Path::new("/tmp/wave-test");
       let path = state_path(root);
       assert_eq!(path, root.join(".ctx/godmode/wave-status.json"));
   }
   ```

   Expected: FAIL

2. Update `state_path()`:

   ```rust
   fn state_path(root: &Path) -> std::path::PathBuf {
       root.join(".ctx").join("godmode").join("wave-status.json")
   }
   ```

3. Update `init()` mkdir:

   ```rust
   std::fs::create_dir_all(root.join(".ctx").join("godmode"))
       .context("failed to create .ctx/godmode directory")?;
   ```

4. Update test `wave_init_creates_state`:

   ```rust
   std::fs::create_dir_all(dir.path().join(".ctx").join("godmode")).unwrap();
   ```

5. Verify: `cargo nextest run -p godmode-core -- wave` → all green
6. Commit: `feat(wave): migrate state to .ctx/godmode/wave-status.json`

### Task 3: Migrate `insights.rs` paths

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/insights.rs`
**Run**: `cargo nextest run -p godmode-core -- insight`

1. Update path helpers:

   ```rust
   fn insights_path(root: &Path) -> PathBuf {
       root.join(".ctx").join("godmode").join("traces").join("insights.jsonl")
   }

   fn insights_md_path(root: &Path, date: &NaiveDate) -> PathBuf {
       root.join(".ctx")
           .join("godmode")
           .join("reports")
           .join(format!("insights-{}.md", date.format("%Y-%m-%d")))
   }
   ```

2. Ensure `append()` creates parent dirs:

   ```rust
   if let Some(parent) = path.parent() {
       std::fs::create_dir_all(parent)?;
   }
   ```

3. Update doc comments: `.ctx/insights.jsonl` → `.ctx/godmode/traces/insights.jsonl`

4. Verify: `cargo nextest run -p godmode-core -- insight` → all green
5. Commit: `feat(insights): migrate to .ctx/godmode/{traces,reports}/`

### Task 4: Migrate `pipeline.rs` state file

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/pipeline.rs`
**Run**: `cargo nextest run -p godmode-core -- pipeline`

1. Update `state_file()`:

   ```rust
   pub fn state_file(root: &Path) -> PathBuf {
       root.join(".ctx").join("godmode").join("pipeline.yaml")
   }
   ```

2. Update tests that create `.ctx/`:

   ```rust
   std::fs::create_dir_all(root.join(".ctx").join("godmode")).unwrap();
   ```

3. Update doc comment on `PipelineState`

4. Verify: `cargo nextest run -p godmode-core -- pipeline` → all green
5. Commit: `feat(pipeline): migrate state to .ctx/godmode/pipeline.yaml`

### Task 5: Migrate `hook_runner.rs` log path

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/integrations/hook_runner.rs`
**Run**: `cargo nextest run -p godmode-core -- hook`

1. Update `append_hook_event()`:

   ```rust
   let log_path = root.join(".ctx").join("godmode").join("traces").join("hooks.log");
   ```

2. Update `read_tail()`:

   ```rust
   let log_path = root.join(".ctx").join("godmode").join("traces").join("hooks.log");
   ```

3. Update module-level doc comments

4. Verify: `cargo nextest run -p godmode-core -- hook` → all green
5. Commit: `feat(hooks): migrate log to .ctx/godmode/traces/hooks.log`

### Task 6: Migrate `workflow.rs` state path

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/workflow.rs`
**Run**: `cargo nextest run -p godmode-core -- workflow`

1. Update `run()` state path:

   ```rust
   let state_path = root
       .join(".ctx")
       .join("godmode")
       .join(format!("workflow-{}.json", def.name));
   ```

2. Update tests that create `.ctx/`:

   ```rust
   std::fs::create_dir_all(dir.join(".ctx").join("godmode")).unwrap();
   ```

3. Verify: `cargo nextest run -p godmode-core -- workflow` → all green
4. Commit: `feat(workflow): migrate state to .ctx/godmode/`

### Task 7: Migrate `session.rs`, `session_trace.rs`, `cruxx.rs`

**Crate**: `godmode-core`
**File(s)**:

- `crates/godmode-core/src/session.rs`
- `crates/godmode-core/src/session_trace.rs`
- `crates/godmode-core/src/integrations/cruxx.rs`
  **Run**: `cargo nextest run -p godmode-core -- session`

1. Update `cruxx.rs` session dir:

   ```rust
   pub fn session_dir(root: &Path) -> PathBuf {
       root.join(".ctx").join("godmode").join("sessions")
   }
   ```

2. Update `session_trace.rs` finalise path:

   ```rust
   let dir = self.root.join(".ctx").join("godmode").join("sessions");
   ```

3. Update `session.rs` summary/trace paths to use `cruxx::session_dir()`

4. Update test helpers

5. Verify: `cargo nextest run -p godmode-core -- session` → all green
6. Commit: `feat(session): migrate traces to .ctx/godmode/sessions/`

### Task 8: Migrate `hooks/*.rs` path references

**Crate**: `godmode-core`
**File(s)**:

- `crates/godmode-core/src/hooks/pre_commit.rs`
- `crates/godmode-core/src/hooks/hook_context.rs`
- `crates/godmode-core/src/hooks/stop_guard.rs`
  **Run**: `cargo nextest run -p godmode-core -- hook`

1. Update `pre_commit.rs`:

   ```rust
   let task_file = root.join(".ctx").join("godmode").join("tasks.yaml");
   ```

   Or better: call `crate::graph::task_file(root)` directly.

2. Update `hook_context.rs` (2 sites):

   ```rust
   let task_file = git_root.join(".ctx").join("godmode").join("tasks.yaml");
   ```

3. Update `stop_guard.rs`:

   ```rust
   let init_file = root.join(".ctx").join("godmode").join(".initialized");
   ```

4. Update test helpers to create `.ctx/godmode/`

5. Verify: `cargo nextest run -p godmode-core -- hook` → all green
6. Commit: `feat(hooks): migrate Rust hooks to .ctx/godmode/ paths`

### Task 9: Migrate `memory_banking.rs`

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/memory_banking.rs`
**Run**: `cargo nextest run -p godmode-core -- memory`

1. Update `mb_dir()`:

   ```rust
   fn mb_dir(git_root: &Path) -> PathBuf {
       git_root.join(".ctx").join("godmode").join("memory-bank")
   }
   ```

   Note: currently uses `memory-banking`, actual dir is `memory-bank`.
   This fixes both the namespace and the stale name.

2. Update all string literals: `.ctx/memory-banking/` → `.ctx/godmode/memory-bank/`

3. Update test paths

4. Verify: `cargo nextest run -p godmode-core -- memory` → all green
5. Commit: `fix(memory-banking): migrate to .ctx/godmode/memory-bank/`

### Task 10: Migrate `init.rs` trace file path

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/init.rs`
**Run**: `cargo nextest run -p godmode-core -- init`

1. Update init to create subdirectories:

   ```rust
   let ctx_dir = root.join(".ctx").join("godmode");
   if !fs.dir_exists(&ctx_dir) {
       fs.create_dir_all(&ctx_dir)?;
       let traces_dir = ctx_dir.join("traces");
       fs.create_dir_all(&traces_dir)?;
       let sessions_dir = ctx_dir.join("sessions");
       fs.create_dir_all(&sessions_dir)?;
       let reports_dir = ctx_dir.join("reports");
       fs.create_dir_all(&reports_dir)?;
       let mb_dir = ctx_dir.join("memory-bank");
       fs.create_dir_all(&mb_dir)?;
       fs.write_file(&ctx_dir.join("tasks.yaml"), "tasks: []\n")?;
       fs.write_file(&traces_dir.join("trace.jsonl"), "")?;
       fs.write_file(&ctx_dir.join("session.toml"), "[session]\n")?;
       project_created = true;
   }
   ```

2. Update test assertions to match new paths

3. Verify: `cargo nextest run -p godmode-core -- init` → all green
4. Commit: `feat(init): scaffold full .ctx/godmode/ directory tree`

### Task 11: Migrate CLI doc comments and `main.rs` string literals

**Crate**: `godmode-cli`
**File(s)**: `crates/godmode-cli/src/main.rs`
**Run**: `cargo nextest run -p godmode-cli`

1. Update doc comments on subcommands:
   - Line 278: `.ctx/GODMODE.hooks.log` → `.ctx/godmode/traces/hooks.log`
   - Line 518: `.ctx/workflow-<name>.json` → `.ctx/godmode/workflow-<name>.json`
   - Line 531: `.ctx/memory-banking/` → `.ctx/godmode/memory-bank/`
   - Line 556: `.ctx/insights-YYYY-MM-DD.md` → `.ctx/godmode/reports/insights-*.md`

2. Update `workflow status` path construction (line 2126):

   ```rust
   let state_path = root.join(".ctx").join("godmode")
       .join(format!("workflow-{}.json", name));
   ```

3. Update init output message (line 2297):

   ```rust
   println!("Added .ctx/ to .gitignore");
   ```

   (This stays as `.ctx/` — the gitignore entry covers the whole tree.)

4. Verify: `cargo check -p godmode-cli` → clean
5. Commit: `docs(cli): update path references to .ctx/godmode/`

### Task 12: Migrate integration test paths

**Crate**: `godmode-core`, `godmode-cli`
**File(s)**:

- `crates/godmode-core/tests/hooks_integration.rs`
- `crates/godmode-core/tests/cruxx_integration.rs`
- `crates/godmode-cli/tests/priority_filter.rs`
- `crates/godmode-cli/tests/visualize_graph.rs`
  **Run**: `cargo nextest run --workspace`

1. Update all test helpers that create `.ctx/GODMODE.tasks.yaml`:

   ```rust
   let ctx = tmp.path().join(".ctx").join("godmode");
   std::fs::create_dir_all(&ctx).unwrap();
   std::fs::write(ctx.join("tasks.yaml"), "tasks: []\n").unwrap();
   ```

2. Update cruxx_integration assertions:

   ```rust
   assert_eq!(path, dir.path().join(".ctx").join("godmode").join("sessions"));
   ```

3. Verify: `cargo nextest run --workspace` → all green
4. Commit: `test: update integration tests for .ctx/godmode/ paths`

### Task 13: Migrate Nushell hook scripts

**File(s)**:

- `hooks/pre-push.nu`
- `hooks/scripts/session-start.nu`
- `hooks/lib/godmode-hook-lib.nu`
- `skills/_lib/trace.nu`
- `skills/_lib/helpers.nu`
  **Run**: `nu -c 'source hooks/lib/godmode-hook-lib.nu'`

1. Update `godmode-hook-lib.nu`:

   ```nu
   let task_file = $"($git_root)/.ctx/godmode/tasks.yaml"
   # ...
   let trace_dir = $"($root)/.ctx/godmode/traces"
   ```

2. Update `pre-push.nu`:

   ```nu
   let task_file = $"($root)/.ctx/godmode/tasks.yaml"
   ```

3. Update `session-start.nu`:

   ```nu
   let task_file = $"($git_root)/.ctx/godmode/tasks.yaml"
   ```

4. Update `skills/_lib/trace.nu`:

   ```nu
   $"(repo-root)/.ctx/godmode/traces/trace.jsonl"
   # ...
   $"(repo-root)/.ctx/godmode/session.json"
   ```

5. Update `skills/_lib/helpers.nu`:

   ```nu
   let trace = $"(repo-root)/.ctx/godmode/traces/trace.jsonl"
   ```

6. Validate each with `nu -c 'source <file>'`
7. Commit: `feat(hooks): migrate nu scripts to .ctx/godmode/ paths`

### Task 14: Migrate Rust-script hooks

**File(s)**:

- `hooks/scripts/godmode-trace.rs`
- `hooks/scripts/session-start.rs`
- `hooks/scripts/post-write-plan-ingest.rs`
  **Run**: `cargo check` (these are standalone scripts, not workspace members)

1. Update `godmode-trace.rs`:

   ```rust
   let trace_file = ctx_dir.join("godmode").join("traces").join("trace.jsonl");
   ```

2. Update `session-start.rs`:

   ```rust
   let task_file = root.join(".ctx").join("godmode").join("tasks.yaml");
   ```

3. Update `post-write-plan-ingest.rs`:

   ```rust
   let task_file = format!("{root}/.ctx/godmode/tasks.yaml");
   ```

4. Update doc comments
5. Commit: `feat(hooks): migrate rust-script hooks to .ctx/godmode/ paths`

### Task 15: Migrate skill hook scripts

**File(s)**:

- `skills/cap/hook.nu`
- `skills/introspection/hook.nu`
- `skills/merge/hook.nu`
- `skills/moa/hook.nu`
- `skills/observability-as-infrastructure/hook.nu`
- `skills/parallel-agents/hook.nu`
- `skills/task-management/hook.nu`
- `skills/using-godmode/hook.nu`
- `skills/writing-plans/hook.nu`
- `skills/verification-before-completion/hook.nu`
- `skills/agent-governance/hook.nu`
- `skills/context-map/hook.nu`
- `skills/mini-context-graph/hook.nu`

1. In each file, replace `.ctx/GODMODE.tasks.yaml` with
   `.ctx/godmode/tasks.yaml` and `.ctx/GODMODE.trace.jsonl` with
   `.ctx/godmode/traces/trace.jsonl`

2. Update `context-map/hook.nu` `.ctx/` containment check:

   ```nu
   if ($file_path | str contains "/.ctx/") {
   ```

   This stays unchanged — `.ctx/godmode/` is still under `.ctx/`.

3. Update parallel-agents helpers:
   - `wave-check.nu`: `.ctx/wave-status.json` → `.ctx/godmode/wave-status.json`
   - `wave-init.nu`: same
   - `agent-prompt-template.md`: same

4. Validate each with `nu -c 'source <file>'`
5. Commit: `feat(skills): migrate hook scripts to .ctx/godmode/ paths`

### Task 16: Migrate SKILL.md output path references

**File(s)**: All SKILL.md files with `.ctx/` references:

- `skills/introspection/SKILL.md`
- `skills/self-reflect/SKILL.md`
- `skills/code-review/SKILL.md`
- `skills/doublecheck/SKILL.md`
- `skills/release-notes/SKILL.md`
- `skills/agent-governance/SKILL.md`
- `skills/decompose/SKILL.md`
- `skills/memory-banking/SKILL.md`
- `skills/memory-banking/helpers/init-memory-bank.nu`
- `skills/memory-banking/hook.nu`
- `skills/observability-as-infrastructure/SKILL.md`
- `skills/task-management/SKILL.md`
- `skills/task-driven-development/SKILL.md`
- `skills/parallel-agents/SKILL.md`
- `skills/writing-plans/SKILL.md`
- `skills/moa/SKILL.md`
- `skills/context-map/SKILL.md`
- `skills/tackle-issues/SKILL.md`
- `skills/pr-author/SKILL.md`
- `skills/using-godmode/SKILL.md`
- `skills/mistake-tracker/SKILL.md`
- `skills/health-score/SKILL.md`
- `skills/pattern-learner/SKILL.md`
- `skills/cap/SKILL.md`
- `skills/ci-fix/SKILL.md`
- `skills/task-management/references/godmode-cli.md`
- `skills/task-driven-development/helpers/task-schema.yaml`
- `skills/decompose/helpers/split-branch.nu`
- `skills/self-reflect/references/retrospective-template.md`
- `skills/moa/helpers/propose.nu`
- `skills/moa/helpers/synthesize.nu`
- `skills/self-reflect/helpers/collect-evidence.nu`

Path substitutions:

- `.ctx/GODMODE.tasks.yaml` → `.ctx/godmode/tasks.yaml`
- `.ctx/GODMODE.trace.jsonl` → `.ctx/godmode/traces/trace.jsonl`
- `.ctx/GODMODE.session.json` → `.ctx/godmode/session.json`
- `.ctx/GODMODE.pipeline.yaml` → `.ctx/godmode/pipeline.yaml`
- `.ctx/GODMODE.hooks.log` → `.ctx/godmode/traces/hooks.log`
- `.ctx/sessions/` → `.ctx/godmode/sessions/`
- `.ctx/traces/` → `.ctx/godmode/traces/`
- `.ctx/insights.jsonl` → `.ctx/godmode/traces/insights.jsonl`
- `.ctx/insights-*.md` → `.ctx/godmode/reports/insights-*.md`
- `.ctx/introspection-*.md` → `.ctx/godmode/reports/introspection-*.md`
- `.ctx/reflect-*.md` → `.ctx/godmode/reports/reflect-*.md`
- `.ctx/wave-status.json` → `.ctx/godmode/wave-status.json`
- `.ctx/workflow-*.json` → `.ctx/godmode/workflow-*.json`
- `.ctx/memory-bank/` → `.ctx/godmode/memory-bank/`
- `.ctx/memory-banking/` → `.ctx/godmode/memory-bank/`
- `.ctx/_WORKING_DIR/` → `.ctx/godmode/_WORKING_DIR/`
- `.ctx/governance-audit.jsonl` → `.ctx/godmode/reports/governance-audit.jsonl`
- `.ctx/.initialized` → `.ctx/godmode/.initialized`
- `.ctx/godmode/decomps/` — stays (already correct namespace)

Also add standardized output path sections to skills that lack them:

- `code-review/SKILL.md`: add "Write report to
  `.ctx/godmode/reports/code-review-YYYY-MM-DD.md`"
- `doublecheck/SKILL.md`: add "Write report to
  `.ctx/godmode/reports/doublecheck-YYYY-MM-DD.md`"
- `release-notes/SKILL.md`: clarify "Write to
  `.ctx/godmode/reports/release-notes-YYYY-MM-DD.md`"

Commit: `docs(skills): standardize all output paths to .ctx/godmode/`

### Task 17: Update CLAUDE.md

**File(s)**: `CLAUDE.md`
**Run**: n/a (documentation only)

1. Update all `.ctx/` path references in the Architecture table and
   State File / Agent Scratch / Trace Events sections

2. Add a "State directory layout" section documenting the `.ctx/godmode/`
   tree structure

3. Commit: `docs: update CLAUDE.md for .ctx/godmode/ path convention`

### Task 18: Move existing state files on disk

**File(s)**: n/a (shell migration)
**Run**: manual

1. Run migration script (one-time, non-destructive):

   ```nu
   let root = (git rev-parse --show-toplevel | str trim)
   let gm = $"($root)/.ctx/godmode"
   mkdir $"($gm)/traces"
   mkdir $"($gm)/sessions"
   mkdir $"($gm)/reports"
   mkdir $"($gm)/memory-bank"
   mkdir $"($gm)/_WORKING_DIR"

   # Tasks
   if ($"($root)/.ctx/GODMODE.tasks.yaml" | path exists) {
       mv $"($root)/.ctx/GODMODE.tasks.yaml" $"($gm)/tasks.yaml"
   }

   # Traces
   if ($"($root)/.ctx/GODMODE.trace.jsonl" | path exists) {
       mv $"($root)/.ctx/GODMODE.trace.jsonl" $"($gm)/traces/trace.jsonl"
   }
   if ($"($root)/.ctx/GODMODE.hooks.log" | path exists) {
       mv $"($root)/.ctx/GODMODE.hooks.log" $"($gm)/traces/hooks.log"
   }
   if ($"($root)/.ctx/traces/activity.jsonl" | path exists) {
       mv $"($root)/.ctx/traces/activity.jsonl" $"($gm)/traces/activity.jsonl"
   }
   if ($"($root)/.ctx/insights.jsonl" | path exists) {
       mv $"($root)/.ctx/insights.jsonl" $"($gm)/traces/insights.jsonl"
   }

   # Sessions
   if ($"($root)/.ctx/sessions" | path exists) {
       glob $"($root)/.ctx/sessions/*.jsonl"
       | each { |f| mv $f $"($gm)/sessions/(($f | path basename))" }
   }

   # Reports
   glob $"($root)/.ctx/introspection-*.md"
   | each { |f| mv $f $"($gm)/reports/(($f | path basename))" }
   glob $"($root)/.ctx/reflect-*.md"
   | each { |f| mv $f $"($gm)/reports/(($f | path basename))" }
   glob $"($root)/.ctx/insights-*.md"
   | each { |f| mv $f $"($gm)/reports/(($f | path basename))" }

   # Memory bank
   if ($"($root)/.ctx/memory-bank" | path exists) {
       glob $"($root)/.ctx/memory-bank/*"
       | each { |f| mv $f $"($gm)/memory-bank/(($f | path basename))" }
   }

   # Wave / pipeline / session.json / .initialized
   for file in [wave-status.json GODMODE.pipeline.yaml GODMODE.session.json .initialized] {
       if ($"($root)/.ctx/($file)" | path exists) {
           let target = ($file | str replace "GODMODE." "")
           mv $"($root)/.ctx/($file)" $"($gm)/($target)"
       }
   }

   # Working dir
   if ($"($root)/.ctx/_WORKING_DIR" | path exists) {
       glob $"($root)/.ctx/_WORKING_DIR/*"
       | each { |f| mv $f $"($gm)/_WORKING_DIR/(($f | path basename))" }
   }
   ```

2. Verify no old files remain:

   ```nu
   ls .ctx/ | where name !~ "godmode|HANDOFF"
   ```

   Expected: empty (only HANDOFF files should remain at `.ctx/` root)

3. Run: `cargo nextest run --workspace` → all green
4. Commit: `chore: migrate existing .ctx/ files to .ctx/godmode/`

### Task 19: Full workspace verification

**Run**: `cargo nextest run --workspace && cargo clippy --workspace -- -D warnings`

1. Run full test suite
2. Run clippy
3. Grep for any remaining stale `.ctx/GODMODE` references:

   ```
   Grep: pattern="\.ctx/GODMODE" path=crates/
   Grep: pattern="\.ctx/GODMODE" path=skills/
   Grep: pattern="\.ctx/GODMODE" path=hooks/
   ```

   Expected: zero matches

4. Grep for remaining `.ctx/memory-banking`:

   ```
   Grep: pattern="\.ctx/memory-banking" path=.
   ```

   Expected: zero matches

5. Commit: `chore: verify no stale .ctx/ path references remain`
