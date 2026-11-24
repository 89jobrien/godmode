# Phase 6: heartbeat script

Read this before setting up the automated handoff heartbeat in Phase 6. The gate conditions that decide whether a handoff may auto-trigger are in SKILL.md and apply before this script runs.

```nu
#!/usr/bin/env nu
# ~/.local/bin/agent-improvement-heartbeat.nu
# Run every N minutes via cron or launchd

let handoff_dir = $"($env.HOME)/dev/.ctx"
let pattern = "HANDOFF.agent-improvement.*.yaml"

let pending = (ls $"($handoff_dir)/($pattern)" | where modified > ((date now) - 1hr))

if ($pending | length) > 0 {
    for $f in $pending {
        # Trigger Codex with the handoff as context
        codex --context (open $f.name | to json) "Apply the changes in this handoff. Run evals after each change. Commit only if all evals pass."
    }
}
```
