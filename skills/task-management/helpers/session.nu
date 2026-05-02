#!/usr/bin/env nu
# session.nu — session start/end wrapper for godmode task management.
# Usage:
#   nu skills/task-management/helpers/session.nu          # start
#   nu skills/task-management/helpers/session.nu --end    # end

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--end: bool = false] {
    if (which godmode | is-empty) {
        print "ERROR: godmode not found — install with: cargo install --path crates/godmode-cli"
        exit 1
    }

    let label = if $end { "end" } else { "start" }
    let tid = (trace-start "task-management" "session.nu" $label)

    if $end {
        run-checked $tid "godmode" "handoff"
    } else {
        run-checked $tid "godmode" "handon"
        run-checked $tid "godmode" "task" "next"
    }

    trace-end $tid
}
