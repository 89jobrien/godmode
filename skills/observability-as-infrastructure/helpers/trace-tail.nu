#!/usr/bin/env nu
# trace-tail.nu — print the last N trace events.
# Usage: nu trace-tail.nu [--n 20] [--session <id>]

def open-trace [] {
    let r = (do { run-external "git" "rev-parse" "--show-toplevel" } | complete)
    if $r.exit_code != 0 { print "Not inside a git repo."; exit 1 }
    let trace = $"($r.stdout | str trim)/.ctx/godmode/traces/trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }
    open $trace | lines | each { from json }
}

def main [--n: int = 20, --session: string = ""] {
    let events = (open-trace)
    let filtered = if ($session | is-empty) {
        $events
    } else {
        $events | where session_id == $session
    }

    $filtered | last $n | table
}
