---
name: "godmode:observability-as-infrastructure"
description: >
  Structured tracing for instrumented godmode helpers and agent lifecycles. Skill
  dispatch hooks record start/completion/error events; helpers record internal
  decisions only when they call the shared trace library. Use when diagnosing session failures, auditing agent
  convergence, or reviewing cross-session continuity.
requires: []
next: []
---

# Observability as Infrastructure

Godmode's native trace reader uses `.ctx/godmode/traces/trace.jsonl`. Claude Skill
hooks record dispatched skill outcomes, session hooks record lifecycle events, and
helpers that import `skills/_lib/trace.nu` can add decision and agent events. Scripts
run outside those hooks are not traced unless they explicitly use that library.

## Event Schema

All events share a common envelope:

| Field        | Type   | Description                                                                  |
| ------------ | ------ | ---------------------------------------------------------------------------- |
| `event`      | string | Event kind (see below)                                                       |
| `session_id` | string | `<git-short-sha>-<epoch-ms>` — stable for the lifetime of one Claude session |
| `ts`         | string | ISO-8601 timestamp                                                           |

### Event kinds

| Event            | Emitted by                        | Key fields                             |
| ---------------- | --------------------------------- | -------------------------------------- |
| `skill.start`    | Skill hook or instrumented helper | `skill`, `helper`, `args`, `trace_id`  |
| `skill.complete` | Skill hook or instrumented helper | `trace_id`, `duration_ms`              |
| `skill.error`    | Skill hook or instrumented helper | `trace_id`, `exit_code`, `stderr_tail` |
| `decision`       | branching helpers                 | `skill`, `helper`, `kind`, `value`     |
| `agent.approved` | agent-governance                  | `agent_id`, `reason`                   |
| `agent.denied`   | agent-governance                  | `agent_id`, `reason`                   |
| `agent.start`    | parallel-agents                   | `agent_id`, `slot`, `crate`            |
| `agent.complete` | parallel-agents                   | `agent_id`, `slot`, `commits`          |
| `agent.blocked`  | parallel-agents                   | `agent_id`, `slot`, `reason`           |

### Session identity

Session state lives at `.ctx/godmode/session.json`. It is created on the first
trace write and reused for all subsequent events. `session_id` combines the git
short SHA and epoch milliseconds. `godmode trace summary` uses it to correlate
work across sessions.

## Shared library

Instrumented Nushell helpers source `skills/_lib/trace.nu`:

```nushell
use ($"(repo-root)/skills/_lib/trace.nu") *

def main [...] {
    let tid = (trace-start "skill-name" "helper.nu" ...$args)
    # ... do work, capture results via | complete ...
    if $result.exit_code != 0 {
        trace-error $tid $result.exit_code $result.stderr
        exit $result.exit_code
    }
    trace-end $tid
}
```

Branching decisions:

```nushell
trace-decision "ci-fix" "fetch-failure.nu" "ci_class" "test_failure"
trace-decision "tackle-issues" "integrate-branches.nu" "skip_reason" "BLOCKED.md found for #7"
```

Agent lifecycle (called by the orchestrator, not the subagent):

```nushell
trace-agent-start    $agent_id $slot $crate
trace-agent-complete $agent_id $slot $commits
trace-agent-blocked  $agent_id $slot $reason
```

## Built-in queries

| Command                    | Purpose                                                       |
| -------------------------- | ------------------------------------------------------------- |
| `godmode trace tail --n N` | Last N events; `--session <id>` scopes the query              |
| `godmode trace failures`   | All `skill.error`, `agent.blocked`, and `agent.denied` events |
| `godmode trace stats`      | Duration histogram, agent convergence, decision log           |
| `godmode trace summary`    | Cross-session triage: errors, blocked agents, unresolved work |

Every query supports the global `--json` flag. The Rust reader ignores legacy
rows without an `event` field without aborting; `trace stats` reports legacy and
malformed row counts. Tail, failures, and stats accept `--current` or an explicit
`--session <id>`.

### Deprecated helper paths

`helpers/trace-tail.nu`, `trace-failures.nu`, `trace-stats.nu`, and
`session-summary.nu` remain as compatibility wrappers. They forward arguments and
exit status to the corresponding native `godmode trace` command; new callers
should invoke the CLI directly.

## Instrumentation contract

Any helper claiming internal instrumentation **must**:

1. Call `trace-start` before any work; capture the returned `trace_id`.
2. Wrap every `run-external` call with `| complete` and check `exit_code`.
3. Call `trace-error` on non-zero exit before calling `exit`.
4. Call `trace-end` on success.
5. Emit `trace-decision` at every branching point that affects outcome.

Trace writes are non-fatal — a failed append must never abort the helper.
Wrap in `try { ... }` if the `.ctx/` directory may not exist.

## Guardrails

- Never delete `.ctx/godmode/traces/trace.jsonl` — archive it instead.
- `session_id` is read-only after creation. Never mutate `.ctx/godmode/session.json`.
- Trace writes must not block the main workflow — keep event payloads small.
- `stderr_tail` is capped at 10 lines to avoid bloating the trace file.
