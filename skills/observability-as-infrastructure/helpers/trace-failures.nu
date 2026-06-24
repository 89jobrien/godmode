#!/usr/bin/env nu
# trace-failures.nu — print all skill.error and agent.blocked events.
# Usage: nu trace-failures.nu [--session <id>]

def open-trace [] {
    let r = (do { run-external "git" "rev-parse" "--show-toplevel" } | complete)
    if $r.exit_code != 0 { print "Not inside a git repo."; exit 1 }
    let trace = $"($r.stdout | str trim)/.ctx/godmode/traces/trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }
    open $trace | lines | each { from json }
}

def main [--session: string = ""] {
    let events = (open-trace)
    let failures = ($events
        | where { |e| $e.event == "skill.error" or $e.event == "agent.blocked" }
        | if ($session | is-empty) { $in } else { where session_id == $session })

    if ($failures | is-empty) {
        print "No failures."
    } else {
        $failures | select event skill? agent_id? slot? exit_code? reason? ts | table
    }
}
