---
name: "godmode:agent-governance"
description: >
  Patterns and techniques for adding governance, safety, and trust controls to AI agent
  systems in Rust. Use this skill when building agents that call external tools, implementing
  policy-based access controls, adding semantic intent classification to detect dangerous
  prompts, creating trust scoring systems for multi-agent workflows, building audit trails,
  or enforcing rate limits and content filters. Covers policy composition, tool wrappers,
  trust decay, and JSONL audit trails — all in Rust.
requires: []
next: []
---

# Agent Governance

Policy-driven controls for AI agent tool access, content filtering, subagent
constraints, and audit trails.

## Reference files

Read the file named in a row before doing the task in its right column. These
files are not loaded automatically.

| File                                             | Read before                                                                                     |
| ------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| `references/policy-layout.md`                    | creating, moving, or reviewing a policy file under `policies/`, or choosing an agent's category |
| `references/policy-schema.md`                    | writing or editing a policy YAML file                                                           |
| `references/hook-behavior.md`                    | changing `hook.nu`, its `PreToolUse/Agent` registration, or any approve/block check             |
| `references/audit-trail.md`                      | reading, aggregating, or changing the governance audit log, or the `audit:` block               |
| `references/adding-policies.md`                  | adding a new agent category or a one-off custom policy                                          |
| `references/rust-patterns.md`                    | building a policy struct for a Rust agent system (Pattern 1)                                    |
| `references/rust-patterns-policy-composition.md` | composing policies or adding a policy to a Rust agent (Pattern 2)                               |
| `references/rust-patterns-governed-tool.md`      | wrapping a tool call in a governance check (Pattern 3)                                          |
| `references/rust-patterns-trust-and-audit.md`    | adding trust scoring or an audit trail to an agent (Patterns 4 and 5)                           |
| `references/governance-levels.md`                | choosing a governance level for an agent                                                        |
| `references/policy-example.yaml`                 | seeing a filled-in policy before writing one                                                    |

## How It Works

```text
Agent Dispatch
  → hook.nu (PreToolUse/Agent)
    → detect-agent-name (from tool_input)
    → resolve-policy.nu (default + category + level)
    → check blocked_tools, allowed_tools, blocked_patterns
    → check subagent constraints (max_concurrent, no_commit_to_main)
    → emit governance-audit.jsonl event
    → approve or block with reason
```

Every agent dispatch passes through the governance hook. The hook resolves
the effective policy by composing three layers:

1. **Default** (`policies/default.yaml`) — baseline all agents inherit
2. **Category** (`policies/by-category/<cat>.yaml`) — per-category overrides
3. **Level** (`policies/levels/<level>.yaml`) — governance level overlay

Composition follows most-restrictive-wins: blocked lists union, allowed lists
intersect, rate limits take minimum, human-approval lists union.

## Policies

The directory layout, category principles, and schema are in
`references/policy-layout.md` and `references/policy-schema.md`. Read them
before touching `policies/`.

## CLI (`godmode policy`)

> `godmode policy` subcommands (`resolve`, `check`, `list`, `audit`) are available in
> the CLI. The examples below document the API; the Nushell helpers remain as an
> alternative for scripting.

All subcommands support `--json`.

### godmode policy resolve

Resolve the effective policy for an agent by composing default + category + level.

```text
godmode policy resolve gm-orchestrator
godmode policy resolve gm-cap-agent --json
godmode policy check gm-cap-agent Bash --input "..."
godmode policy list [--json]
godmode policy audit [--date YYYY-MM-DD] [--json]
```

## Nushell Helpers (fallback)

When the `godmode` binary isn't on PATH (e.g., during plugin development),
the hook falls back to the nushell helpers in `helpers/`:

- `resolve-policy.nu <agent> [--level <level>] [--json]`
- `check-tool.nu <agent> <tool> [--input <content>] [--level <level>]`
- `audit-report.nu [--date YYYY-MM-DD] [--json]`

These implement the same logic as the Rust module but run as standalone scripts.

## Hook Behavior

The `hook.nu` runs as a `PreToolUse/Agent` hook (registered in `hooks/hooks.json`).
Its dispatch steps, the checks it runs, and the reminders it injects on approval
are in `references/hook-behavior.md`.

## Audit Trail

All governance decisions are logged to
`.ctx/godmode/traces/governance-audit.jsonl`. The record format and the
`audit-report.nu` aggregation are in `references/audit-trail.md`.

## Governance Levels

See `references/governance-levels.md` for full details.

| Level    | Tools             | Subagents | Approval    | Max calls |
| -------- | ----------------- | --------- | ----------- | --------- |
| Open     | all               | 5         | none        | 1000      |
| Standard | R/W/E/Bash/Glob/G | 5         | force-push  | 200       |
| Strict   | R/Glob/Grep       | 2         | writes+bash | 50        |
| Locked   | R/Glob/Grep       | 0         | everything  | 25        |

## Adding Categories and Custom Policies

Steps for a new category and the (not yet implemented) custom policy path are in
`references/adding-policies.md`.

## Implementation Patterns (Rust)

The Rust reference implementations are split by pattern. Pattern 1 (policy struct)
is in `references/rust-patterns.md`. Pattern 2 (composition) is in
`references/rust-patterns-policy-composition.md`. Pattern 3 (governed tool wrapper)
is in `references/rust-patterns-governed-tool.md`. Patterns 4 and 5 (trust scoring,
append-only audit) are in `references/rust-patterns-trust-and-audit.md`. They are not
used by the hook, which is Nushell, but document the architectural approach.

## Best Practices

| Practice                       | Rationale                                                    |
| ------------------------------ | ------------------------------------------------------------ |
| Policy as configuration        | YAML, not code — change without rebuilding                   |
| Most-restrictive-wins          | When composing, deny always overrides allow                  |
| Pre-flight, not post-hoc       | Check before execution, not after side effects               |
| Fail closed                    | If governance check errors, deny rather than allow           |
| Separate policy from logic     | Governance is independent of agent business logic            |
| Append-only audit              | Never modify audit entries — immutability enables compliance |
| Inject reminders, not just log | Agents see governance rules in stderr before acting          |

## Related

- `skills/observability-as-infrastructure/SKILL.md` — trace events for audit
- `skills/parallel-agents/SKILL.md` — subagent dispatch protocol
- `references/governance-levels.md` — level detail and choosing guide
- `helpers/policy-template.yaml` — blank policy for new categories
