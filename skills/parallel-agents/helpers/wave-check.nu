#!/usr/bin/env nu
# wave-check.nu — verify all agents completed and run the post-merge workspace gate.
# Usage: nu skills/parallel-agents/helpers/wave-check.nu

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [] {
    let root = (repo-root)
    let state_path = $"($root)/.ctx/wave-status.json"

    if not ($state_path | path exists) {
        print "ERROR: .ctx/wave-status.json not found — run wave-init.nu first"
        exit 1
    }

    let tid = (trace-start "parallel-agents" "wave-check.nu")
    let agents = (open $state_path).agents | transpose key value

    mut blocking = []
    for agent in $agents {
        let s = $agent.value.status
        let commits = $agent.value.commits
        if $s == "pending" {
            $blocking = ($blocking | append $"($agent.key): still pending")
            trace-decision "parallel-agents" "wave-check.nu" "agent_pending" $agent.key
        } else if ($commits | is-empty) {
            $blocking = ($blocking | append $"($agent.key): no commits — incomplete")
            trace-decision "parallel-agents" "wave-check.nu" "agent_no_commits" $agent.key
        } else if $s == "blocked" {
            $blocking = ($blocking | append $"($agent.key): BLOCKED")
            trace-agent-blocked $"wave-($agent.key)" $agent.key "reported blocked in wave-status"
        } else {
            trace-agent-complete $"wave-($agent.key)" $agent.key $commits
        }
    }

    if not ($blocking | is-empty) {
        trace-error $tid 1 ($blocking | str join "\n")
        for b in $blocking { print $b }
        exit 1
    }

    run-checked $tid "cargo" "nextest" "run" "--workspace"
    run-checked $tid "cargo" "clippy" "--workspace" "--" "-D" "warnings"

    trace-end $tid
    print "All agents complete. Workspace gates passed."
}
