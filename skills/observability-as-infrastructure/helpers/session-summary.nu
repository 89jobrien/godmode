#!/usr/bin/env nu
# session-summary.nu — cross-session triage: what the last session left behind.
# Usage: nu session-summary.nu [--sessions 3]

def open-trace [] {
    let r = (do { run-external "git" "rev-parse" "--show-toplevel" } | complete)
    if $r.exit_code != 0 { print "Not inside a git repo."; exit 1 }
    let trace = $"($r.stdout | str trim)/.ctx/godmode/traces/trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }
    open $trace | lines | each { from json }
}

def main [--sessions: int = 3] {
    let events = (open-trace)

    # Collect unique session_ids in order of first appearance
    let session_ids = ($events
        | get session_id
        | uniq
        | last $sessions)

    for sid in $session_ids {
        let evts = ($events | where session_id == $sid)
        let started = ($evts | first | get ts)
        let errors  = ($evts | where event == "skill.error"   | length)
        let blocked = ($evts | where event == "agent.blocked" | length)
        let complete = ($evts | where event == "agent.complete" | length)
        let completed_ids = ($evts | where event == "agent.complete" | get agent_id)
        let blocked_ids  = ($evts | where event == "agent.blocked"  | get agent_id)
        let resolved_ids = ($completed_ids | append $blocked_ids)
        let running_agents = ($evts
            | where event == "agent.start"
            | where { |e| not ($resolved_ids | any { |id| $id == $e.agent_id }) })
        let running = ($running_agents | length)
        let decisions = ($evts | where event == "decision" | length)

        print $"--- ($sid) @ ($started)"
        print $"    errors=($errors)  blocked=($blocked)  agents: ($complete) complete / ($running) still-running / ($decisions) decisions"

        if $running > 0 {
            for a in $running_agents {
                print $"    UNRESOLVED agent: ($a.agent_id) slot=($a.slot)"
            }
        }
    }
}
