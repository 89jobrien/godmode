#!/usr/bin/env nu
# trace-failures.nu — print all skill.error and agent.blocked events.
# Usage: nu trace-failures.nu [--session <id>]

def main [--session: string = ""] {
    let trace = $"(git rev-parse --show-toplevel | str trim)/.ctx/GODMODE.trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }

    let events = (open $trace | lines | each { from json })
    let failures = ($events
        | where { |e| $e.event == "skill.error" or $e.event == "agent.blocked" }
        | if ($session | is-empty) { $in } else { where session_id == $session })

    if ($failures | is-empty) {
        print "No failures."
    } else {
        $failures | select event skill? agent_id? helper? slot? exit_code? reason? ts | table
    }
}
