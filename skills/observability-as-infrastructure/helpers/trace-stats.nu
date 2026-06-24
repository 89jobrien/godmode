#!/usr/bin/env nu
# trace-stats.nu — duration histogram per skill, agent convergence summary.
# Usage: nu trace-stats.nu [--session <id>]

def open-trace [] {
    let r = (do { run-external "git" "rev-parse" "--show-toplevel" } | complete)
    if $r.exit_code != 0 { print "Not inside a git repo."; exit 1 }
    let trace = $"($r.stdout | str trim)/.ctx/godmode/traces/trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }
    open $trace | lines | each { from json }
}

def main [--session: string = ""] {
    let events = (open-trace)
    let scoped = if ($session | is-empty) { $events } else { $events | where session_id == $session }

    # Duration histogram — skill.complete events only
    print "=== skill durations (ms) ==="
    let durations = ($scoped
        | where event == "skill.complete"
        | group-by skill
        | transpose skill runs
        | each { |row|
            let ms = ($row.runs | get duration_ms)
            { skill: $row.skill, runs: ($ms | length), avg_ms: ($ms | math avg | into int), max_ms: ($ms | math max) }
        })
    if ($durations | is-empty) { print "(none)" } else { $durations | table }

    # Agent convergence
    print "\n=== agent convergence ==="
    let agent_starts   = ($scoped | where event == "agent.start"    | get agent_id)
    let agent_complete = ($scoped | where event == "agent.complete"  | get agent_id)
    let agent_blocked  = ($scoped | where event == "agent.blocked"   | get agent_id)

    for id in $agent_starts {
        let status = if ($agent_blocked | any { |a| $a == $id }) {
            "blocked"
        } else if ($agent_complete | any { |a| $a == $id }) {
            "complete"
        } else {
            "running"
        }
        print $"  ($id): ($status)"
    }

    if ($agent_starts | is-empty) { print "(no agents)" }

    # Decision log
    print "\n=== decisions ==="
    let decisions = ($scoped | where event == "decision")
    if ($decisions | is-empty) {
        print "(none)"
    } else {
        $decisions | select skill helper kind value ts | table
    }
}
