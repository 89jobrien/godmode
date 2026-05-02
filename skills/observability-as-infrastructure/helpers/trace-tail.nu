#!/usr/bin/env nu
# trace-tail.nu — print the last N trace events.
# Usage: nu trace-tail.nu [--n 20] [--session <id>]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--n: int = 20, --session: string = ""] {
    let events = (open-trace)
    let filtered = if ($session | is-empty) {
        $events
    } else {
        $events | where session_id == $session
    }

    $filtered | last $n | table
}
