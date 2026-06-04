---
name: "godmode:observability-as-infrastructure"
description: >
  Structured tracing for all godmode helpers and subagents. Every script
  invocation, branching decision, and agent lifecycle transition is recorded
  as a JSONL event. Use when diagnosing session failures, auditing agent
  convergence, or reviewing cross-session continuity.
---

# Observability as Infrastructure

The trace file is the infrastructure. No external monitoring needed — every
helper and agent emits structured events to `.ctx/godmode/traces/trace.jsonl`, the
same append-only file used by the core CLI's `start_traced`/`complete_traced`.

## Event Schema

All events share a common envelope:

| Field        | Type   | Description                                                                  |
| ------------ | ------ | ---------------------------------------------------------------------------- |
| `event`      | string | Event kind (see below)                                                       |
| `session_id` | string | `<git-short-sha>-<epoch-ms>` — stable for the lifetime of one Claude session |
| `ts`         | string | ISO-8601 timestamp                                                           |

### Event kinds

| Event            | Emitted by        | Key fields                             |
| ---------------- | ----------------- | -------------------------------------- |
| `skill.start`    | every helper      | `skill`, `helper`, `args`, `trace_id`  |
| `skill.complete` | every helper      | `trace_id`, `duration_ms`              |
| `skill.error`    | every helper      | `trace_id`, `exit_code`, `stderr_tail` |
| `decision`       | branching helpers | `skill`, `helper`, `kind`, `value`     |
| `agent.start`    | parallel-agents   | `agent_id`, `slot`, `crate`            |
| `agent.complete` | parallel-agents   | `agent_id`, `slot`, `commits`          |
| `agent.blocked`  | parallel-agents   | `agent_id`, `slot`, `reason`           |

### Session identity

Session state lives at `.ctx/godmode/session.json`. It is created on the first
trace write and reused for all subsequent events. `session_id` = git short SHA

- epoch ms. `session-summary.nu` uses this to correlate work across sessions.

## Shared library

All helpers source `skills/_lib/trace.nu`:

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

## Query helpers

| Helper               | Purpose                                                       |
| -------------------- | ------------------------------------------------------------- |
| `trace-tail.nu`      | Last N events; `--session <id>` to scope                      |
| `trace-failures.nu`  | All `skill.error` and `agent.blocked` events                  |
| `trace-stats.nu`     | Duration histogram, agent convergence, decision log           |
| `session-summary.nu` | Cross-session triage: errors, blocked agents, unresolved work |

## Instrumentation contract

Every helper **must**:

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
