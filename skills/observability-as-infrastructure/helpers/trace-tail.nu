#!/usr/bin/env nu
# trace-tail.nu — print the last N trace events.
# Usage: nu trace-tail.nu [--n 20] [--session <id>]

def main [--n: int = 20, --session: string = ""] {
    let trace = $"(git rev-parse --show-toplevel | str trim)/.ctx/GODMODE.trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }

    let events = (open $trace | lines | each { from json })
    let filtered = if ($session | is-empty) {
        $events
    } else {
        $events | where session_id == $session
    }

    $filtered | last $n | table
}
