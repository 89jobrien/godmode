# Design: Replayable Godmode Command Traces

## Contents

- [Goal](#goal)
- [Approved Approach](#approved-approach)
- [Context Map](#context-map)
- [Crate Ownership](#crate-ownership)
- [Public API](#public-api)
- [Trace Shape And Replay](#trace-shape-and-replay)
- [Data Flow](#data-flow)
- [Storage Layout](#storage-layout)
- [Security Invariants](#security-invariants)
- [Failure Behavior](#failure-behavior)
- [Verification](#verification)
- [Out Of Scope](#out-of-scope)
- [Compatibility And Risk](#compatibility-and-risk)

## Goal

Persist every parsed Godmode command as a private, project-local, replayable
`Crux<CommandOutcome>` without changing the command's observable output, exit status, or side
effects.

## Approved Approach

Use a central CLI dispatch boundary with an injectable tee-style I/O port, one immutable trace per
command invocation, explicit `handon`/`handoff` session correlation, and strict cached-outcome
replay that never repeats command side effects.

## Context Map

### Files to Modify

| File                                                             | Purpose                              | Change                                                                                            |
| ---------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- |
| `crates/godmode-core/src/command_trace.rs`                       | Command trace domain and persistence | Add invocation types, redaction, Crux orchestration, replay, secure storage, and fallback records |
| `crates/godmode-core/src/command_io.rs`                          | Output port                          | Add the byte-oriented output trait used by CLI and subprocess adapters                            |
| `crates/godmode-core/src/session_identity.rs`                    | Trace-session lifecycle              | Add begin/current/close operations while preserving pinned-root state                             |
| `crates/godmode-core/src/lib.rs`                                 | Core exports                         | Export command tracing, command I/O, and session identity modules                                 |
| `crates/godmode-core/src/detect.rs`                              | Root pinning                         | Read and update the typed session state without discarding trace-session fields                   |
| `crates/godmode-core/src/hooks/trace_log.rs`                     | Fallback JSONL                       | Reuse canonical session identity and append bounded trace-persistence failures                    |
| `crates/godmode-core/src/integrations/crux.rs`                   | Crux integration                     | Retain task-step helpers and share command-trace naming conventions                               |
| `crates/godmode-core/src/integrations/rx.rs`                     | Subprocess execution                 | Tee child output through the command I/O port                                                     |
| `crates/godmode-core/src/pipeline.rs`                            | Pipeline execution                   | Thread the output port into `rx` calls                                                            |
| `crates/godmode-core/src/workflow.rs`                            | Workflow execution                   | Replace inherited subprocess streams with concurrent tee capture                                  |
| `crates/godmode-core/src/session.rs`                             | Session domain                       | Return output data instead of printing directly                                                   |
| `crates/godmode-core/src/memory_banking.rs`                      | Memory-bank domain                   | Return output data instead of printing directly                                                   |
| `crates/godmode-core/src/{builder,insights,release,worktree}.rs` | Remaining core output                | Route observable messages through returned values or the output port                              |
| `crates/godmode-cli/src/main.rs`                                 | Composition root                     | Parse without early exit, establish tracing, finalize exit status, and handle replay              |
| `crates/godmode-cli/src/commands/mod.rs`                         | Central dispatch                     | Accept `CommandIo`, return `CommandResult`, and guarantee one traced boundary                     |
| `crates/godmode-cli/src/commands/*.rs`                           | Command handlers                     | Replace direct printing and process exits with `CommandIo` and returned statuses                  |
| `hooks/scripts/godmode-trace.rs`                                 | Host trace producer                  | Use the canonical session-state schema                                                            |
| `skills/_lib/trace.nu`                                           | Skill trace producer                 | Use the canonical session-state schema                                                            |
| `crates/godmode-cli/tests/command_traces.rs`                     | CLI regression tests                 | Cover output parity, exit parity, persistence, replay, and sessions                               |
| `crates/godmode-core/tests/command_trace_integration.rs`         | Core integration tests               | Cover secure persistence, redaction, fallback, concurrency, and replay                            |
| `tests/conformance/src/cli_trace_tests.rs`                       | Dispatch conformance                 | Ensure every command variant crosses the trace boundary                                           |
| `tests/conformance/src/lib.rs`                                   | Conformance registration             | Register command-trace tests                                                                      |

### Dependencies

| File                                             | Relationship                                                                      |
| ------------------------------------------------ | --------------------------------------------------------------------------------- |
| `crates/godmode-cli/Cargo.toml`                  | May require workspace Tokio for concurrent subprocess stream draining             |
| `crates/godmode-core/src/session_trace.rs`       | Existing task-graph Crux wrapper remains separate and must not own command traces |
| `crates/godmode-core/src/integrations/output.rs` | Existing structured handon/handoff output is the rendering reference pattern      |
| `crates/godmode-core/tests/crux_integration.rs`  | Existing Crux serialization and task-step test pattern                            |
| `tests/conformance/src/crux_tests.rs`            | Existing Crux conformance coverage remains valid                                  |

### Existing Obstacles

- Command handlers write directly to stdout/stderr and contain direct `process::exit` calls.
- Some subprocess adapters inherit terminal streams, bypassing in-process capture.
- Clap parse failures and shell-completion output currently occur outside normal dispatch.
- `.ctx/godmode/session.json` is independently read or mutated by root pinning, hooks, and skill
  helpers.
- Crux's step redactor does not redact the top-level typed `Crux::value`; the outcome must be
  redacted before finalization.

### Risk

- The CLI output refactor spans most command handlers and must preserve byte-for-byte `--json`
  stdout compatibility.
- Replacing inherited subprocess streams can alter TTY detection, buffering, color, prompts, and
  stdout/stderr interleaving.
- Command output and arguments can contain credentials; no unredacted bytes may reach persistent
  storage or fallback logs.
- The current working tree contains extensive unrelated staged changes in many affected files.
  Implementation must be isolated or coordinated before editing.

## Crate Ownership

- **Owner crate:** `godmode-core` owns command outcome types, trace-session identity, redaction,
  Crux orchestration, replay validation, secure persistence, and fallback records.
- **Adapter crate:** `godmode-cli` owns parsed Clap types, output formatting, the terminal tee
  adapter, command dispatch, and mapping returned command status to `ExitCode`.
- **Affected test crate:** `godmode-conformance` verifies that every parsed command passes through
  the tracing boundary.

Dependency direction remains `godmode-cli -> godmode-core -> crux-runtime`. Core must not import
Clap types, CLI command enums, or rendering flags.

## Public API

### Command I/O Port

```rust
pub trait CommandIo: Send {
    fn write_stdout(&mut self, bytes: &[u8]) -> std::io::Result<()>;
    fn write_stderr(&mut self, bytes: &[u8]) -> std::io::Result<()>;
}
```

The CLI provides a tee adapter that writes bytes immediately to the terminal and retains bounded
in-memory captures. Core subprocess adapters depend only on this port.

### Session Identity

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SessionId {
    value: String,
}

impl SessionId {
    pub fn as_str(&self) -> &str;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SessionState {
    pub session_id: Option<SessionId>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub pinned_root: Option<std::path::PathBuf>,
}

pub fn begin_session(root: &std::path::Path) -> anyhow::Result<SessionId>;

pub fn current_or_begin_session(root: &std::path::Path) -> anyhow::Result<SessionId>;

pub fn close_session(
    root: &std::path::Path,
    session_id: &SessionId,
) -> anyhow::Result<()>;
```

`handon` begins a fresh session before running integrations. Commands without an active session
create one lazily. `handoff` remains associated with the active session and closes it only after
its command trace has been persisted.

### Command Trace Types

```rust
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommandIdentity {
    pub name: String,
    pub arguments: Vec<String>,
    pub project_relative_cwd: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum CapturedEncoding {
    Utf8,
    EscapedBytes,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CapturedStream {
    pub content: String,
    pub encoding: CapturedEncoding,
    pub original_bytes: u64,
    pub truncated: bool,
    pub redaction_failed: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommandResult {
    pub exit_code: i32,
    pub structured: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommandCapture {
    pub result: CommandResult,
    pub stdout: CapturedStream,
    pub stderr: CapturedStream,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommandOutcome {
    pub schema_version: u32,
    pub session_id: SessionId,
    pub invocation_id: String,
    pub command: CommandIdentity,
    pub result: CommandResult,
    pub stdout: CapturedStream,
    pub stderr: CapturedStream,
}

pub enum InvocationMode<'a> {
    Live,
    Replay { trace_path: &'a std::path::Path },
}

pub struct InvocationReport {
    pub outcome: CommandOutcome,
    pub trace_id: String,
    pub trace_path: Option<std::path::PathBuf>,
    pub replayed: bool,
    pub persistence_warning: Option<String>,
}

pub async fn invoke_command<F, Fut>(
    root: &std::path::Path,
    identity: CommandIdentity,
    mode: InvocationMode<'_>,
    execute: F,
) -> anyhow::Result<InvocationReport>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = anyhow::Result<CommandCapture>> + Send;
```

Expected nonzero statuses, including the existing empty-result status `2`, are represented by
`CommandResult` and are not tracing errors. `anyhow::Error` is reserved for failures that prevent a
command outcome from being formed.

### CLI-Private API

```rust
trait TerminalIo: godmode_core::command_io::CommandIo {
    fn finish(self, result: CommandResult) -> CommandCapture;
}

fn dispatch(
    cmd: Cmd,
    root: &std::path::Path,
    format: OutputFormat,
    io: &mut dyn godmode_core::command_io::CommandIo,
) -> anyhow::Result<CommandResult>;
```

`OutputFormat` remains CLI-owned and represents human, JSON, and SARIF rendering without changing
existing stdout contracts.

The CLI adds a global `--replay-trace <path>` option. Replay still requires the original command
shape on the command line so the raw, in-memory `CommandIdentity` can be passed to
`CruxCtx::step_keyed`. The identity persisted in `CommandOutcome` is redacted; raw arguments are
never serialized. The CLI composition root runs on the workspace Tokio runtime so it can await the
Crux step while existing synchronous handlers are migrated incrementally.

## Trace Shape And Replay

Each invocation produces exactly one keyed Crux step:

```text
Crux<CommandOutcome>
└── step_keyed("execute-command", stable command identity)
```

Live execution runs the command closure once, redacts the resulting capture, records the redacted
outcome as the step output, finalizes the Crux value, and persists it. The raw capture exists only
in process memory and is used solely for live terminal output.

Replay loads the typed trace, converts it with `Crux::to_snapshot`, and seeds `CruxCtx` in strict
mode. A matching trace returns the recorded redacted outcome without calling the command closure,
subprocesses, integrations, or state mutations. Godmode does not offer lenient command replay
because an unmatched command could repeat destructive effects. A replay invocation may write its
own observability trace; that write is not a replayed command side effect.

## Data Flow

1. **Parse:** `Cli::try_parse` produces `Cmd` and output format without terminating the process.
2. **Identify:** the CLI converts the parsed command into a stable, redacted `CommandIdentity`.
3. **Correlate:** core begins or loads the project session ID.
4. **Execute:** `invoke_command` runs dispatch through the terminal tee adapter for live mode, or
   resolves a strict cached outcome for replay mode.
5. **Capture:** handlers and subprocess adapters send observable stdout/stderr bytes through
   `CommandIo`; output is streamed live and retained within configured bounds.
6. **Redact:** arguments, structured result, stdout, stderr, and errors are redacted before they
   enter a Crux step, final value, logger, or file.
7. **Finalize:** core finalizes one `Crux<CommandOutcome>` with the keyed command step.
8. **Persist:** the trace store writes the envelope atomically under the active project session.
9. **Fallback:** a failed primary write appends a bounded, redacted failure record and returns the
   original command status plus a persistence warning.

## Storage Layout

```text
.ctx/godmode/crux-traces/
├── <session-id>/
│   └── <crux-id>.json
└── fallback.jsonl
```

Filenames use generated Crux IDs and never include command-controlled text. The existing
`.ctx/godmode/session.json` remains the canonical active-session and pinned-root state file.

## Security Invariants

1. All observable command and subprocess output must pass through `CommandIo`; interactive
   commands must explicitly opt out of capture rather than silently changing TTY behavior.
2. Arguments are stored as arrays, never reconstructed shell strings. Environment variables are
   not recorded.
3. Raw arguments, output, structured values, and errors are redacted before any persistent or log
   write. Redaction failure omits the affected field and sets `redaction_failed`; it never fails
   open.
4. Captures are bounded by bytes, nesting depth, and collection count. Truncation and original byte
   counts are recorded.
5. The canonical project root is fixed before command execution. Trace paths must be owned
   directories, not symlinks.
6. Trace directories use mode `0700`; trace, lock, temporary, and fallback files use mode `0600`
   independent of umask.
7. Primary writes create an unpredictable same-directory file with `create_new`, write only
   redacted bytes, call `sync_all`, rename without overwriting, and best-effort sync the parent.
8. Fallback records use the same permission, ownership, symlink, redaction, locking, size, and
   retention controls. They contain only event ID, command label, status, timestamps, truncation
   flags, and a sanitized persistence error; they omit full command output.
9. Primary and fallback records share an invocation ID and are idempotent so consumers can
   deduplicate concurrent retries.
10. Trace persistence success or failure does not alter command exit code, signal, timeout, or
    cancellation classification. Trace diagnostics use stderr only, preserving `--json` stdout.
11. Trace files are audit data, not executable specifications. Replay never constructs or invokes
    a shell command from stored text and never re-executes mutating command logic.
12. Project-local permissions provide privacy, not tamper evidence. Compliance-grade integrity
    would require a separate keyed or append-only external sink.

## Failure Behavior

- Command failures with a formed outcome are persisted like successes.
- Primary persistence failure preserves the original command status, emits a sanitized warning on
  stderr, and attempts the bounded fallback JSONL record.
- Failure of both primary and fallback persistence remains non-fatal to the command and is reported
  only through a sanitized stderr warning.
- Structured-result serialization failure does not replace the command status; the trace records
  an omitted structured value when safe to do so.
- Panics, aborts, and unhandled signals cannot guarantee a finalized Crux envelope. No design claim
  is made for those process-termination cases.

## Verification

- Unit tests cover identity normalization, nested redaction, secret-bearing flags, URLs, multiline
  output, invalid UTF-8, truncation, and redaction failure.
- Canary-secret tests prove no canary reaches primary traces, fallback records, warnings, or errors.
- Filesystem tests cover permissions, symlinks, hard links, pre-existing targets, interrupted
  writes, cleanup checks, and concurrent fallback writers.
- CLI integration tests cover success, expected nonzero, execution error, read-only command,
  mutating command, JSON output, SARIF output, and subprocess output.
- Replay tests prove the command closure, task graph mutations, session mutations, subprocesses,
  and external integrations do not execute on a strict cache hit.
- Session tests prove `handon` starts one session, ordinary commands correlate to it, and `handoff`
  persists before closing it.
- Static conformance rejects direct `println!`, `eprintln!`, and `process::exit` in command handlers.
- Required gates are `cargo fmt --all --check`, `cargo check --workspace`,
  `cargo clippy --workspace -- -D warnings`, `cargo nextest run --workspace`, and
  `just conformance`.

## Out Of Scope

- Capturing complete Claude Code, OpenCode, or other agent-host conversation transcripts. Host hooks
  may later emit interaction traces using the same session identity and store.
- Re-executing commands from persisted arguments.
- Lenient replay of Godmode commands.
- External tamper-evident or remote trace storage.
- Replacing existing task-transition JSONL and governance audit logs in the first implementation.
- Guaranteeing trace finalization after aborts, unhandled signals, or process termination outside
  Rust unwinding.

## Compatibility And Risk

- **Breaking public API:** additive core API, but broad internal CLI handler signature changes.
- **Serialization format:** new versioned `CommandOutcome`; no migration of existing JSONL files.
- **CLI output:** no intended change; JSON and SARIF stdout must remain byte-compatible.
- **New external dependency:** none required by the design. Reuse workspace Crux, Serde, Chrono,
  Tokio, and standard-library filesystem primitives.
- **Feature flag:** none. Command tracing is standard Godmode behavior; persistence failures remain
  non-fatal.
- **Cross-crate boundary:** core owns domain/storage, CLI owns rendering/terminal adaptation, and
  conformance tests enforce coverage.
