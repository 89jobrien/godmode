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

## How It Works

```
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

### Directory layout

```
policies/
  default.yaml                # baseline — Standard level
  by-category/
    agent.yaml                # dispatchers (tdd-crate, moa, parallel-agents)
    plan.yaml                 # planners (brainstorm, planner, writing-plans)
    issue.yaml                # issue handlers (cross-issue, triage, tackle)
    qual.yaml                 # quality gates (code-review, health-score, dead-code)
    ops.yaml                  # operations (changelog, dep-audit, dep-bump, orchestrator)
    git.yaml                  # git agents (cap, pr-author, wave-integration)
    refac.yaml                # refactoring (refactoring, doc-maintainer, workspace-refactor)
    dbg.yaml                  # debugging (ci-fix, systematic-debugging, mistake-tracker)
    meta.yaml                 # meta-analysis (pattern-learner)
    gov.yaml                  # governance/observability (observability-as-infrastructure)
  levels/
    open.yaml                 # audit only — local dev
    standard.yaml             # tool allowlist + content filters — default
    strict.yaml               # read-only + human approval — sensitive ops
    locked.yaml               # read-only, no Bash, no Agent — compliance
```

### Category design principles

| Category | Can Write | Can Bash | Can Agent | Rationale                                   |
| -------- | --------- | -------- | --------- | ------------------------------------------- |
| agent    | yes       | yes      | **yes**   | Only category with Agent tool — dispatchers |
| plan     | yes       | no       | no        | Planners propose, not execute               |
| issue    | yes       | yes      | no        | Need gh CLI but no sub-delegation           |
| qual     | reports   | yes      | no        | Analyze and report, not fix                 |
| ops      | yes       | yes      | no        | Broad access, destructive ops gated         |
| git      | yes       | yes      | no        | Git operations, force-push gated            |
| refac    | yes       | yes      | no        | Source edits, tests required after          |
| dbg      | yes       | yes      | no        | Full diagnostic access                      |
| meta     | reports   | no       | no        | Read-only analysis                          |
| gov      | reports   | no       | no        | Observers don't modify                      |

### Policy schema

```yaml
name: "<policy-name>"
level: standard # open | standard | strict | locked
category: "<category>" # matches agents/cfg/*.cfg.yaml
inherits: default

allowed_tools: [Read, Write, ...] # empty = no restriction
blocked_tools: [Agent, ...] # always denied
blocked_patterns: # regex, any match = deny
  - "(?i)--no-verify"
max_calls_per_dispatch: 200 # rate limit per agent dispatch

require_human_approval: # operations needing user OK
  - cargo publish

subagent: # constraints on sub-agents
  max_concurrent: 5
  must_verify_branch: true
  no_commit_to_main: true
  max_retries_on_failure: 3
  require_commit_before_done: true
  blocked_flags: ["--no-verify"]

audit:
  enabled: true
  format: jsonl
  path: .ctx/godmode/traces/governance-audit.jsonl
  log_allowed: false
  log_denied: true
  log_reviews: true
```

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

**On dispatch:**

1. Extracts agent identity from `tool_input` (subagent_type, description, prompt)
2. Matches against `agents/cfg/*.cfg.yaml` to find the agent's category
3. Resolves the effective policy via `resolve-policy.nu`
4. Checks:
   - Is the Agent tool in `allowed_tools`? (only `agent` category allows it)
   - Is the Agent tool in `blocked_tools`?
   - Is `max_concurrent` exceeded (locked = 0)?
   - Does the prompt/description contain any `blocked_patterns`?
5. Emits a governance event to `governance-audit.jsonl`
6. If all checks pass: approves and injects governance reminders to stderr
7. If any check fails: blocks with reason

**Governance reminders injected on approval:**

- Branch verification requirement
- No-commit-to-main rule
- Max retries on failure
- Commit-before-done requirement
- Blocked flags list

## Audit Trail

All governance decisions are logged to
`.ctx/godmode/traces/governance-audit.jsonl`:

```json
{
  "ts": "2026-06-04T14:30:00+0000",
  "event": "governance.check",
  "action": "denied",
  "agent_id": "gm-dispatch",
  "tool_name": "Agent",
  "reason": "content matches blocked pattern: (?i)--no-verify",
  "pattern": "(?i)--no-verify",
  "session_id": "abc1234-1717500000000"
}
```

Use `audit-report.nu` to aggregate and summarize.

## Governance Levels

See `references/governance-levels.md` for full details.

| Level    | Tools             | Subagents | Approval    | Max calls |
| -------- | ----------------- | --------- | ----------- | --------- |
| Open     | all               | 5         | none        | 1000      |
| Standard | R/W/E/Bash/Glob/G | 5         | force-push  | 200       |
| Strict   | R/Glob/Grep       | 2         | writes+bash | 50        |
| Locked   | R/Glob/Grep       | 0         | everything  | 25        |

## Adding a New Category

1. Add the category field to your agent's `agents/cfg/<name>.cfg.yaml`
2. Copy `helpers/policy-template.yaml` to `policies/by-category/<category>.yaml`
3. Fill in allowed/blocked tools based on the agent's responsibilities
4. The hook picks it up automatically — no registration needed

## Adding a Custom Policy

For one-off agents that don't fit a category:

1. Create `policies/custom/<agent-name>.yaml`
2. Update `resolve-policy.nu` to check `custom/` before `by-category/`
3. The custom policy composes on top of default, overriding category

(Not yet implemented — use category policies for now.)

---

## Implementation Patterns (Rust)

The patterns below are reference implementations for building governance
into Rust agent systems. They are not used by the hook (which is Nushell)
but document the architectural approach.

### Pattern 1: Policy Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub name: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub blocked_tools: Vec<String>,
    #[serde(default)]
    pub blocked_patterns: Vec<String>,
    #[serde(default = "default_max_calls")]
    pub max_calls_per_request: usize,
    #[serde(default)]
    pub require_human_approval: Vec<String>,
}
```

### Pattern 2: Policy Composition

```rust
/// Merge policies: blocked lists union, allowed lists intersect,
/// rate limits take minimum.
pub fn compose_policies(policies: &[GovernancePolicy]) -> GovernancePolicy {
    let mut combined = GovernancePolicy::default();
    for policy in policies {
        combined.blocked_tools.extend(policy.blocked_tools.clone());
        combined.blocked_patterns.extend(policy.blocked_patterns.clone());
        combined.require_human_approval.extend(
            policy.require_human_approval.clone()
        );
        combined.max_calls_per_request =
            combined.max_calls_per_request.min(policy.max_calls_per_request);
        // Intersect allowed_tools if both specify
        if !policy.allowed_tools.is_empty() {
            combined.allowed_tools = if combined.allowed_tools.is_empty() {
                policy.allowed_tools.clone()
            } else {
                combined.allowed_tools.iter()
                    .filter(|t| policy.allowed_tools.contains(t))
                    .cloned()
                    .collect()
            };
        }
    }
    combined
}
```

### Pattern 3: Governed Tool Wrapper

Wraps any tool function with policy check + rate limit + audit:

```rust
pub async fn call(&self, input: String) -> Result<String> {
    // 1. Check tool allowlist/blocklist
    match self.policy.check_tool(&self.name) {
        PolicyAction::Deny => bail!("blocked by policy"),
        PolicyAction::Review => bail!("requires human approval"),
        PolicyAction::Allow => {}
    }
    // 2. Rate limit
    let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
    if count > self.policy.max_calls_per_request {
        bail!("rate limit exceeded");
    }
    // 3. Content filter
    if let Some(pattern) = self.policy.check_content(&input) {
        bail!("blocked content: {pattern}");
    }
    // 4. Execute and audit
    let result = (self.inner)(input).await;
    self.audit.lock().unwrap().append(/* ... */);
    result
}
```

### Pattern 4: Trust Scoring

Track agent reliability with exponential decay:

```rust
impl TrustScore {
    pub fn current(&self, decay_rate: f64) -> f64 {
        let elapsed = now() - self.last_updated;
        self.score * (-decay_rate * elapsed).exp()
    }
}

// Gate on trust level
match trust.current(0.001) {
    t if t >= 0.7 => { /* autonomous */ }
    t if t >= 0.4 => { /* with oversight */ }
    _             => { /* deny */ }
}
```

### Pattern 5: Audit Trail

Append-only JSONL — never modify entries after write:

```rust
pub fn export_jsonl(&self, path: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true).append(true).open(path)?;
    for entry in &self.entries {
        writeln!(file, "{}", serde_json::to_string(entry)?)?;
    }
    Ok(())
}
```

---

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
