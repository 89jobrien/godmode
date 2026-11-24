# Governance hook behavior

Read this before changing `hook.nu`, the `PreToolUse/Agent` registration in `hooks/hooks.json`, or any check that decides whether a dispatch is approved or blocked.

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
