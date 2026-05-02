#!/usr/bin/env nu
# trace-failures.nu — print all skill.error and agent.blocked events.
# Usage: nu trace-failures.nu [--session <id>]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--session: string = ""] {
    let events = (open-trace)
    let failures = ($events
        | where { |e| $e.event == "skill.error" or $e.event == "agent.blocked" }
        | if ($session | is-empty) { $in } else { where session_id == $session })

    if ($failures | is-empty) {
        print "No failures."
    } else {
        $failures | select event skill? agent_id? helper? slot? exit_code? reason? ts | table
    }
}
