# Governance Levels

Four graduated levels of agent oversight. Each level is a composable
policy overlay in `policies/levels/<level>.yaml`.

| Level        | Controls                                          | Use Case                          |
| ------------ | ------------------------------------------------- | --------------------------------- |
| **Open**     | Audit only, no tool restrictions                  | Local dev, testing, prototyping   |
| **Standard** | Tool allowlist + content filters + subagent rules | Default for all production agents |
| **Strict**   | Read-only tools + human approval for writes       | PII, financial, infrastructure    |
| **Locked**   | Read-only, no Bash, no Agent, full audit          | Compliance-critical systems       |

## Level Details

### Open

- All tools allowed, no content filters
- Subagent constraints relaxed (no branch verification, no commit requirement)
- Audit still enabled (log everything)
- Max 1000 calls per dispatch
- Use case: local development, exploratory sessions

### Standard

- Tools: Read, Write, Edit, Bash, Glob, Grep (Agent only for agent-category)
- Content filters: blocks `--no-verify`, `--force`, `rm -rf /`, `drop/truncate`
- Subagent: max 5 concurrent, must verify branch, no main commits, 3 retries
- Max 200 calls per dispatch
- Use case: everyday agent work, feature development, CI triage

### Strict

- Tools: Read, Glob, Grep only (Write/Edit/Bash require human approval)
- Agent tool blocked (no sub-delegation)
- Content filters: all Standard patterns + `curl -d`, `op read`, API key patterns
- Subagent: max 2 concurrent, 1 retry
- Max 50 calls per dispatch
- Use case: agents touching sensitive data, infrastructure changes

### Locked

- Tools: Read, Glob, Grep only
- Write, Edit, Bash, Agent all blocked (not just requiring approval)
- All operations require human approval (`"*"`)
- No subagents allowed (max_concurrent: 0)
- Max 25 calls per dispatch
- Full audit (log allowed + denied + reviews)
- Use case: regulatory environments, audit-only observation

## Choosing a Level

1. Start at **Standard** for any agent with tool access.
2. Escalate to **Strict** when the agent handles PII, financial data,
   or infrastructure.
3. Use **Locked** for regulatory environments (SOC2, HIPAA, PCI-DSS)
   or audit-only observation.
4. **Open** is for local dev only — never deploy Open to shared environments.

## Composition Rules

When multiple policies are composed (default + category + level):

- **Blocked lists**: union (most restrictive wins)
- **Allowed lists**: intersection (narrowest scope wins)
- **Rate limits**: minimum (lowest wins)
- **Human approval**: union (any policy requiring review wins)
- **Subagent constraints**: per-field most-restrictive

## Applying a Level

Override the default level for a specific dispatch:

```bash
# Resolve policy at strict level
nu skills/agent-governance/helpers/resolve-policy.nu gm-orchestrator --level strict --json

# Check a specific tool call
nu skills/agent-governance/helpers/check-tool.nu gm-cap-agent Bash --input "git push --force" --level strict
```

The hook reads the level from `policies/default.yaml` unless overridden.
Per-agent level overrides are planned but not yet implemented — currently
the level applies uniformly to all agents.
