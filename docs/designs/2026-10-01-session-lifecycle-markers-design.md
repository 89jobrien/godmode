# Design: Session Lifecycle Markers In The Trace Log

## Contents

- [Goal](#goal)
- [Approved Approach](#approved-approach)
- [Context Map](#context-map)
- [Crate Ownership](#crate-ownership)
- [Public API](#public-api)
- [Reader Semantics](#reader-semantics)
- [Writer Semantics](#writer-semantics)
- [Data Flow](#data-flow)
- [Verification](#verification)
- [Out Of Scope](#out-of-scope)
- [Compatibility And Risk](#compatibility-and-risk)

## Goal

Mint `session.start`/`session.end` at every session boundary so any session that writes a trace
event is discoverable without running `handon`/`handoff`; make the reader model those markers
consistently so they neither appear as actors named after their own session id nor cause
`godmode trace summary` to list sessions that recorded no work; and ensure a close line names the
session being closed rather than a successor that rotation opened on the way past.

## Approved Approach

Empty-name convention plus a summary filter: a session lifecycle marker becomes a nameless
`Record` with a public `is_marker()` predicate, `summary` skips sessions whose records are all
markers, `session_id` refuses to mint when the id it minted could not be persisted,
`close_session` resolves the stored id without rotating, and the git subprocess that minting
depends on moves behind a `HeadSha` port.

## Context Map

### Files to Modify

| File                                                | Purpose                                | Change                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------- | -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/godmode-core/src/trace_query.rs`            | Trace read layer                       | Marker-aware `from_legacy`, new `is_marker()`, `is_session_marker()`, `Stats::active_sessions`, marker-based `session_starts`/`session_ends`                                                                                                                                                        |
| `crates/godmode-core/src/hooks/trace_log.rs`        | Trace writer, session identity         | Add the `HeadSha` port, `GitHeadSha` adapter, and `Writer<G>`; add `StoredSession` and `read_stored_session`; move the ten writer free functions onto `Writer`; return `None` when the rotated session file cannot be persisted; add `close_session`; correct the rotation rationale comment; tests |
| `crates/godmode-cli/src/commands/trace.rs`          | `tail`, `failures`, `stats`, `summary` | Render marker name in `tail`, print `active_sessions` in `stats`, skip marker-only sessions in `summary`, add `active_sessions` to `stats_json`; update writer call sites to `trace_log::writer()`                                                                                                  |
| `crates/godmode-cli/src/commands/handoff.rs`        | Session close                          | Call `close_session` instead of the writer's `append`, so the close names the session being closed                                                                                                                                                                                                  |
| `crates/godmode-core/src/hooks/parallel_agents.rs`  | Hook emission                          | Update writer call sites to `trace_log::writer()`                                                                                                                                                                                                                                                   |
| `crates/godmode-core/src/hooks/observability.rs`    | Hook emission                          | Update writer call sites to `trace_log::writer()`                                                                                                                                                                                                                                                   |
| `crates/godmode-core/src/hooks/agent_governance.rs` | Hook emission                          | Update writer call sites to `trace_log::writer()`                                                                                                                                                                                                                                                   |

### Dependencies

| File                                                                | Relationship                                                                                                                                   |
| ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| `crates/godmode-core/src/lib.rs`                                    | Declares `pub mod trace_query` and `pub mod trace_store`. No new modules, so no export changes.                                                |
| `crates/godmode-core/src/hooks/trace_log.rs`                        | Writer emission, lifecycle                                                                                                                     | Fourteen call sites become `trace_log::writer()...` |
| `crates/godmode-cli/src/commands/handon.rs`                         | Writes `session.start` through the writer's `append` with a `graph` tally. Unchanged behaviour; one call site updated.                         |
| `skills/_lib/trace.nu`                                              | Funnels every skill event through `godmode trace emit`, so it reaches the writer. Unaffected.                                                  |
| `skills/observability-as-infrastructure/helpers/session-summary.nu` | Reads raw JSONL and filters `event in ["session.start","session.end"]`, then `uniq`s ids. Tolerant of multiple markers per session. No change. |

`Record` has exactly one consumer: the four subcommands in `trace.rs`. `insight.rs` and
`testing/audit.rs` match on unrelated `Record` types of their own.

### Test Coverage

| Location                                              | Covers                                                                    |
| ----------------------------------------------------- | ------------------------------------------------------------------------- |
| `trace_query.rs` unit tests                           | Legacy projection, crux projection, `status_for_event`, `kind_for_event`  |
| `trace_log.rs` unit tests                             | `session_id` rotation, steady-state silence, marker emission, `read_back` |
| `trace.rs` unit tests                                 | Subcommand output over a `LEGACY` fixture                                 |
| `trace.rs` unit tests (new)                           | `summary` row selection, `tail` marker rendering                          |
| `skills/observability-as-infrastructure/helpers/*.nu` | Raw-JSONL readers, separate implementation, no coverage obligation here   |

Gaps this design fills: markers were never asserted to be nameless, `session_starts` and
`session_ends` were never exercised, no test covered a `session.json` write failure, and no test
covered a close line attributed to a rotated-to id.

### Reference Patterns

| File                                      | Pattern to follow                                                              |
| ----------------------------------------- | ------------------------------------------------------------------------------ |
| `trace_query.rs:66` `Record::is_failure`  | Predicate naming, `matches!` body, one-line doc explaining the rule            |
| `trace_query.rs:101-104` `scoped` comment | Stated preference against adding session-filter variants to the core query API |
| `trace_log.rs:27` `append`                | `json!` marker construction, best-effort write with `let _ =`                  |

### Risk

- [x] Public API change: yes — additive plus one breaking move. Additive: `Record::is_marker()`,
      `Stats::active_sessions`, `hooks::trace_log::{close_session, HeadSha, GitHeadSha, Writer}`.
      Breaking in-workspace: the ten writer free functions become `Writer<G>` methods. The crate
      is unpublished and every consumer is in-workspace.
- [x] Generic-bound change: `session_id` and `read_session_id` gain `<G: HeadSha>`. Both private.
- [x] Serialization format change: yes — `godmode trace stats --json` gains one key. No
      consumer exists: `trace-stats.nu` and the other observability helpers read raw JSONL, not
      this output.
- [x] CLI output change: yes — `summary` lists fewer rows; `stats` prints one extra line; `tail`
      renders a different name for markers.
- [x] Trace JSONL on disk: no format change. Rotation adds bare marker lines in the existing
      pre-0.7 `{event, ...}` vocabulary, which `from_legacy` already accepts. Existing logs
      parse unchanged.
- [x] Cross-crate boundary: yes — `godmode-core` and `godmode-cli`, with core strictly ahead of
      cli. `is_marker()` must exist before `trace.rs` compiles. The `HeadSha` port is internal to
      `godmode-core` and adds no cross-crate dependency.
- [x] Trait object or generic? Generic, not `dyn`. `HeadSha` has one production implementation and
      one test double, and the call is not hot — one mint per session boundary. `&dyn HeadSha`
      would add a vtable indirection to an `fn` that already shells out to a subprocess.

## Crate Ownership

- **Owner crate**: `godmode-core` — owns both the trace writer (`hooks::trace_log`) and the trace
  read layer (`trace_query`). The change is confined to the two halves of that single concern.
- **Affected crates**: `godmode-cli` — renders the records. Imports `is_marker()` and
  `active_sessions` from `godmode_core::trace_query`.

No new crate. No new module. No new external dependency.

## Public API

### Methods

```rust
impl Record {
    pub fn is_marker(&self) -> bool;
}
```

True when the record is a session lifecycle marker rather than an actor's step. Derived from
`self.metadata["event"]`, which survives parsing because `from_legacy` strips `event`,
`session_id`, and `ts` into `known`, leaves metadata empty, and reinserts `event` at
trace_query.rs:259-261.

### Traits

```rust
/// The one infrastructure fact session minting cannot compute itself.
///
/// Small and single-method by intent: the trace writer needs a short HEAD sha and
/// nothing else from VCS. Defined beside the domain logic that consumes it;
/// `GitHeadSha` is the adapter.
pub trait HeadSha {
    /// Short HEAD sha for the repository at `root`, or `"unknown"` when there is
    /// none. Never fails: a repo with no HEAD still gets a usable session id.
    fn head_sha(&self, root: &Path) -> String;
}
```

```rust
pub struct GitHeadSha;

impl HeadSha for GitHeadSha {
    fn head_sha(&self, root: &Path) -> String;
}
```

`GitHeadSha` replaces the inline `std::process::Command::new("git")` at trace_log.rs:94-102
verbatim, including the `"unknown"` fallback and the empty-sha rejection. No behaviour change.

**The port threads the whole writer path.** Only two functions reach `session_id` directly:
`append`, which is the legacy path used by `handon`/`handoff`, and `emit`, which is the crux path.
The eight lifecycle helpers — `skill_start`, `skill_complete`, `skill_error`, `decision`,
`tool_use`, `agent_start`, `agent_complete`, `agent_blocked` — all funnel through `emit`, so
they inherit injection by becoming methods rather than by each gaining a parameter:

```rust
pub struct Writer<G: HeadSha> {
    git: G,
}

impl<G: HeadSha> Writer<G> {
    pub fn new(git: G) -> Self;

    pub fn append(&self, root: &Path, event_name: &str, fields: Value);
    pub fn emit(&self, root: &Path, ev: NewEvent);
    pub fn skill_start(&self, root: &Path, skill: &str, helper: &str, args: &[String]) -> String;
    pub fn skill_complete(&self, root: &Path, skill: &str, trace_id: &str, duration_ms: u64);
    pub fn skill_error(
        &self,
        root: &Path,
        skill: &str,
        trace_id: &str,
        exit_code: i32,
        reason: &str,
        duration_ms: u64,
    );
    pub fn decision(&self, root: &Path, skill: &str, helper: &str, kind: &str, value: &str);
    pub fn tool_use(&self, root: &Path, tool: &str, failed: bool);
    pub fn agent_start(&self, root: &Path, agent_id: &str, slot: &str, crate_name: &str);
    pub fn agent_complete(&self, root: &Path, agent_id: &str, slot: &str, commits: &[String]);
    pub fn agent_blocked(&self, root: &Path, agent_id: &str, slot: &str, reason: &str);
}

/// Composition helper: the wiring every production caller uses.
pub fn writer() -> Writer<GitHeadSha>;
```

`to_step`, `new_trace_id`, `parse_trace_id`, and `elapsed_ms` stay free functions. They are pure
and never resolve a session id, so making them methods would attach a dependency they do not
have.

A `Writer<G>` struct rather than a generic parameter on each of ten functions, because ten
generic parameters plus ten concrete wrappers is twenty public items to enable one test seam. The
struct is the dedicated-dependency-bag pattern, and it also reads better at the call sites:
`trace_log::writer().skill_start(...)`.

The trade is a **breaking change to `hooks::trace_log`'s public surface**: the ten flat free
functions become methods. That is acceptable here — the crate is unpublished, and the seventeen
call sites are all in-workspace — and it is recorded under [Compatibility And Risk](#compatibility-and-risk).
Keeping the flat names as delegating wrappers was considered and rejected: it preserves the
public surface but reinstates the twenty-item ceremony this struct exists to avoid, and buys
nothing, since every in-workspace caller would be updated regardless.

### Types

```rust
struct StoredSession {
    session_id: Option<String>,
    started_at: Option<String>,
    claude_session_id: Option<String>,
}

pub struct Stats {
    // existing fields unchanged, plus:
    pub active_sessions: usize,
}
```

`StoredSession` is private and exists so one function owns the shape of `session.json`. Without
it, `close_session` would re-read and re-parse the file to extract a field `session_id` had
already read — duplicating the schema knowledge that previously had exactly one home.

`active_sessions` parallels the existing `distinct_sessions`: both count distinct session ids.
`Stats` derives `Debug, Clone, Default` and has no `Serialize` impl — `trace.rs::stats_json`
builds the JSON by hand — so the new field needs only an entry in the struct literal in
`stats()` and a key in `stats_json`.

`Record::name` keeps its type. Only its value for markers changes, to `""`.

### Functions

```rust
// Private to trace_query; shared by from_legacy and Record::is_marker.
fn is_session_marker(event: &str) -> bool;

// Private to hooks::trace_log.
fn read_stored_session(root: &Path) -> StoredSession;
fn session_id<G: HeadSha>(root: &Path, git: &G) -> Option<String>;
fn read_session_id<G: HeadSha>(root: &Path, git: &G) -> String;

pub fn close_session(root: &Path, fields: Value);

pub fn parse(body: &str) -> Vec<Record>;
pub fn stats(records: &[Record]) -> Stats;
pub fn summarise_session(session_id: &str, records: &[&Record]) -> SessionSummary;
```

`parse`, `stats`, `summarise_session`, `to_step`, `new_trace_id`, `parse_trace_id`, and
`elapsed_ms` keep their signatures. The ten writer functions become `Writer<G>` methods, and the
behaviour changes are described under [Writer Semantics](#writer-semantics).

`close_session` is new and lives in `hooks::trace_log` because that module already owns session
lifecycle, not because the caller needs it. It emits one `session.end` carrying only `graph`,
`dirty_files`, and `ts`, attributed to the `session_id` on `read_stored_session` rather than to a
resolved id. It emits nothing when that field is `None`, since there is no session to close.
`Writer::append` keeps its rotating behaviour.

`close_session` stays a free function rather than a `Writer` method: it deliberately must _not_
rotate, and giving it a `git` field it never uses would attach a dependency to a function whose
entire purpose is to avoid one.

### SRP note

`session_id` reads, compares, mints, persists, and emits boundary markers. That looks like five
responsibilities but is one: _what is the current session identity, and when does it change._
Emitting a boundary marker is inseparable from deciding the boundary occurred, so the function is
left intact and the reasoning is stated here rather than split. Splitting it would put the
boundary decision and the marker emission in different places, where they could drift — which is
the failure this design exists to prevent.

The one exception is the git subprocess, which is not part of that responsibility at all. It now
sits behind `HeadSha`.

## Reader Semantics

### Markers are nameless

`from_legacy` currently derives `name` from `skill`, then `agent_id`, then `tool`, then
`session_id`. The last rung is what names a marker after its own session id. For a marker the
whole chain is skipped and the name is empty.

The `session_id` fallback is retained for every non-marker line. `{"event":"hook.observed",
"session_id":"s1"}` still yields `name == "s1"`.

### Marker detection

`Record::is_marker()` reads `metadata["event"]`. It is the single predicate used by `stats`, by
`trace.rs::summary`, and by `trace.rs::tail`.

### `stats` counters

`session_starts` and `session_ends` currently filter on `r.name.is_empty()`, which was
unreachable: every marker carries `session_id`, so `name` was never empty and both counters have
read `0` since they were introduced in 2f7cffd. The predicate becomes `r.is_marker()`, which
states the intent directly and also covers a marker arriving on the crux path.

> This is a small extension beyond the brainstormed approach, which reached the same place via
> the empty-name convention. The empty name is still required — it is what keeps markers out of
> `stats`'s `skill_durations` rollup (trace_query.rs:391) and out of the `tail` name column — but
> `is_marker()` is the correct predicate for counting markers, and it does not depend on an
> incidental consequence. Flagged for confirmation rather than adopted silently.

### `active_sessions`

`distinct_sessions` keeps its literal meaning: how many ids appear in the log. A session that
was opened and rotated away without recording work is genuinely a session, and `stats` reports
that. `active_sessions` counts ids carrying at least one non-marker record. Both are printed.

This does not protect against the mint-storm in [Writer Semantics](#writer-semantics) — a
storm-minted session carries its own event and so counts as active. Only the write-failure fix
does that.

### `summary` row selection

A session is listed only if at least one of its records is not a marker. The rule is applied in
`trace.rs`, not in `trace_query`, honouring the stated preference at trace.rs:101-104 that the
core API stays free of session-filter variants.

This hides two kinds of row. The obvious one is a session that opened and did nothing. The
subtler one is created by this very change: rotation closes the previous id with `session.end`,
so a session that recorded nothing and then rotated away now exists as a single marker line.
Without the filter both render identically as all-zero rows.

`summarise_session` is unchanged. It derives `started` from the first open record, and the bare
`session.start` is now the first line in its session, so `started` is populated for every session
`summary` lists. A session whose only records are terminal has no open record and would render
`--- s1 @ `; the filter guarantees such a session is not listed, and a test pins the invariant.

## Writer Semantics

### Mint only what persisted

`session_id` currently returns `Some(fresh)` regardless of whether writing
`.ctx/godmode/session.json` succeeded; that write is `let _ = std::fs::write(...)`. If it
fails, the next event reads the same stale file, rotates again, mints a different id, and emits
another marker pair — so every trace event gets its own session id and two markers.

With the fix, a failed write returns `None` and emits no markers. `read_session_id` then falls
back to its existing `"no-session"` sentinel, so correlation collapses into one coherent bucket
rather than one bucket per event, and the log stops growing.

No signature change: `session_id` is already `fn(&Path) -> Option<String>` and is private.

### A close must not open a successor

`handoff` wrote its close line through `append`, which calls `read_session_id`, which rotates.
So when `handoff` ran against a stale session, rotation minted a successor, emitted
`session.end(old)` and `session.start(new)`, and the rich close line — carrying the graph tally
and dirty-file count — was then written under the _new_ id. The session that actually ended got
only a bare marker with no tally; the session that had just begun carried a close it never had.

`append` is the wrong entry point for a close, so `close_session` is added to `trace_log`:

```rust
/// Close the session already on file, without rotating.
///
/// A close must name the session being closed. Routing it through `append`
/// resolves and rotates the id first, so a stale session would open a
/// successor and attribute the rich close line to it.
pub fn close_session(root: &Path, fields: Value);
```

It resolves through `read_stored_session`, which reads `session.json` without rotating, and
emits nothing when no id is on file — there is no session to close, and a `session.end` under the
`"no-session"` sentinel would be a marker for a session that never existed. `handoff` swaps
`trace_log::append` for `trace_log::close_session`; its `graph` and `dirty_files` fields are
unchanged, so the payload is identical.

`handoff` still leaves the id on file, so a later same-day event is attributed to the closed
session. Rotation, not the close, is what mints. That yields both a bare and a rich `session.end`
for one session, which is already the documented situation.

### `session.end` stays best-effort

Nothing observes that a session has ended. A last session closes only when a later one rotates,
which is why the reader change is required rather than optional. Documented on `session_id` and
on `append_lifecycle`.

### Known writer interactions this design does not fix

- A session can carry more than one `session.start` and both a bare and a rich `session.end`, so
  `session_starts` and `session_ends` count marker _lines_, not distinct sessions. The field doc
  must say so.
- Concurrent processes crossing one boundary mint different ids, because `fresh` is
  `format!("{head}-{}", Utc::now().timestamp_millis())`. Last write wins; both markers are
  emitted; events split across both ids. No id is a phantom, so the write-failure fix does not
  touch this.

## Data Flow

1. Source: a trace event reaches `Writer::append` from a hook or the CLI, or `Writer::emit` from
   the crux-typed helpers and `godmode trace emit`. Every production caller reaches them through
   `trace_log::writer()`, which supplies `GitHeadSha`.
2. Transform: both paths resolve the session id through `read_session_id`, which calls the
   generic `session_id`. On rotation, `session_id` asks the `HeadSha` port for the short sha,
   verifies the new id persisted, then writes one bare `session.start` for it and one bare
   `session.end` for the displaced id. `close_session` skips this path and resolves the id
   already on file.
3. Sink: `trace_store::append_line` appends JSONL to `.ctx/godmode/traces/trace.jsonl`.
4. Read: `trace_store::read_body` → `trace_query::parse` → `record_from_value` → `from_step` for
   crux lines, `from_legacy` for the `{event, ...}` vocabulary. Markers land as nameless records
   with `metadata["event"]` preserved.
5. Query: `trace.rs` calls `is_marker()` in `tail` and `summary`, and `stats()` calls it for the
   marker counters and `active_sessions`.

Hexagonal boundary: the only external I/O is the JSONL file and the VCS. The file is already
abstracted behind `trace_store::read_body` / `append_line` and `trace_log::discover_root` — free
functions, not a trait, and introducing one for a single filesystem implementation would violate
the guideline against porting a lone implementation. The VCS was _not_ abstracted: the git
subprocess was domain logic calling infrastructure directly, which is the one real DIP violation
in this module, and it now sits behind `HeadSha` with `GitHeadSha` as its adapter.

- **Port** (trait): `HeadSha` in `godmode-core::hooks::trace_log`
- **Adapter** (impl): `GitHeadSha` in `godmode-core::hooks::trace_log`
- **Test double**: an in-module `struct FixedSha(&'static str)`, co-located with the `trace_log`
  unit tests rather than in a shared fixtures module, since nothing else needs it
- **Composition root**: `hooks::trace_log::writer()`, the single production wiring point. It is
  not `main.rs` — `godmode-core` wires its own adapters today, a pre-existing structural
  deviation this design neither introduces nor fixes.

## Verification

Twelve new tests, plus the four rotation tests already in the working tree. Gates:
`cargo fmt --all`, `cargo clippy --workspace -- -D warnings`, `cargo nextest run --workspace`.

### `trace_query.rs` — five

1. A legacy `session.start` line yields `name == ""` and `is_marker()`.
2. A legacy `session.end` line yields `name == ""`, `is_marker()`, and
   `status == Some(StepStatus::Ok)`, so it still counts as an end.
3. `{"event":"hook.observed","session_id":"s1"}` yields `name == "s1"` and `!is_marker()`. This is
   the guard that the empty-name change is scoped to markers and did not disable the
   `session_id` fallback generally.
4. Over a log with one `session.start` and one `skill.error` under the same id: `session_starts
== 1`, `session_ends == 0`, `active_sessions == 1`, `distinct_sessions == 1`.
5. With `s1` carrying only a marker and `s2` carrying a marker plus an error: `distinct_sessions
== 2`, `active_sessions == 1`.

### `trace_log.rs` — four new, four already present

6. Rotation against an unwritable `session.json` returns `None` and emits no markers. Forcing the
   failure portably by creating `session.json` as a directory, so the write fails with
   `IsADirectory` rather than depending on permissions.
7. `close_session` against a stale `session.json` emits one `session.end` carrying the caller's
   fields under the id already on file, emits no `session.start`, and does not overwrite
   `session.json`. This is the attribution regression, asserted directly in core rather than
   through `handoff`, whose `integrations::handoff` dependency makes it impractical to unit test.
8. Full-path sha injection. `Writer::new(FixedSha("deadbee")).skill_start(...)` writes a crux
   `Step` whose `metadata.session_id` begins with `deadbee-`. Driving the public writer, not the
   private `session_id`, is the point of threading the port — it proves the whole path resolves
   identity through the port.
9. `GitHeadSha` against a directory that is not a repository returns `"unknown"` rather than
   panicking or erroring, pinning the best-effort contract the trait documents.

Existing: `rotation_closes_the_previous_session_and_opens_the_next`,
`first_ever_session_opens_without_closing_a_predecessor`,
`an_unchanged_session_emits_no_further_lifecycle_markers`,
`a_helper_that_never_runs_handon_is_still_visible_in_the_log`.

### `trace.rs` — three

8. `summary` skips a marker-only session and keeps the row for a session that recorded an error.
9. `tail` renders a marker's event name in the `name` column.
10. Every session `summary` lists has a non-empty `started`. This pins the coupling between
    `summarise_session` deriving `started` from the first open record and rotation always emitting
    one, so a future removal of the marker fails loudly instead of printing `--- s1 @ `.

### Documentation

- `Record::name` — state that lifecycle markers are nameless in the legacy vocabulary, and that
  `from_step` does not honour the convention. No writer emits a marker in crux `Step` shape
  today, so the asymmetry must be documented rather than discovered.
- `Record::is_marker`, `Stats::active_sessions`, `session_starts`, `session_ends` — one line each
  stating the distinction drawn.
- `trace_query` module doc — the reader claims to turn "those lines" into typed records; say that
  session lifecycle lines become nameless marker records and why `metadata["event"]` survives.
- `trace.rs::summary` — the skip and its reason.
- `trace_log.rs:121-125` — the existing rationale comment credits `trace.rs summary` with
  grouping sessions by lifecycle markers. It groups by `session_id` at trace.rs:274-283. The
  comment is accurate only for `session-summary.nu:19`. Correcting it is part of this change,
  since this change is what it describes.

## Out Of Scope

- Concurrent minting. Fixing it requires mutual exclusion around the session file's
  read-modify-write. No lock primitive exists in the workspace, and a `create_new` lockfile also
  needs stale-holder recovery for a crashed process. Tracked separately.
- Dropping `session.*` at the `parse` boundary. Rejected during brainstorm: it makes `parse`
  non-faithful to the log and forecloses "which sessions existed" as a query.
- A first-class lifecycle type on `Record`, replacing the inferred `is_marker()`.
- Making `from_step` honour the nameless-marker convention.
- Emitting markers in the crux `Step` shape rather than the pre-0.7 vocabulary.
- Warning output when the `session.json` write fails. `godmode-core` has no warning channel and
  no locking today; introducing one is a separate decision.
- Threading `HeadSha` through the ten public writers so `append` is injectable end-to-end.
  Superseded — the port now reaches the full writer path via `Writer<G>`.
- Keeping the ten flat writer free functions as delegating wrappers over `Writer`. Rejected: it
  reinstates the twenty-item ceremony the struct exists to avoid and preserves a surface no
  in-workspace caller would keep using.
- Moving the `GitHeadSha` wiring into `godmode-cli`'s composition root. `godmode-core` wires its
  own adapters today; changing that is a workspace-wide restructuring well beyond this slice.
- The `git rev-parse --show-toplevel` call in `trace_log::discover_root`. A second VCS
  dependency, in a different function, serving a different concern (root discovery, not
  minting). Bundling it into `HeadSha` would violate the guideline against porting beyond the
  current goal.

## Compatibility And Risk

- **Breaking API changes**: yes, within the workspace. `hooks::trace_log`'s ten writer free
  functions become `Writer<G>` methods, so all seventeen call sites across six files are updated
  in the same change. No external breakage — the crate is unpublished and there are no
  out-of-workspace consumers. Also additive: `is_marker`, `active_sessions`, `close_session`,
  `HeadSha`, `GitHeadSha`, `Writer`.
- **New external dependency**: no. `GitHeadSha` uses `std::process::Command`, already in use.
- **Migration ordering**: `Writer<G>` and the ten methods land before any call site is updated,
  or the crate will not compile mid-change. The `handon`/`handoff` behaviour changes ride along in
  the same commit as the mechanical call-site update, so no commit is left with the port half
  applied.
- **New external dependency**: no.
- **Feature flag required**: no.
- **On-disk trace format**: unchanged. Markers use the `{event, session_id, ts}` shape that
  `from_legacy` already accepts. Existing logs parse unchanged and are not rewritten.
- **Log growth**: rotation now adds up to two lines per boundary that previously added none.
  Negligible against the storm case, which the write-failure fix removes.
- **Regression surface**: `summary` listing fewer rows is the intended behaviour change, not a
  silent loss — a session with no records has nothing to report. Test 7 pins it.
- **Branch**: `develop`. The gates above must pass before commit; `main` is not a target.
