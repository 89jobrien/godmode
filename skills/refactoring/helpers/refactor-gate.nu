#!/usr/bin/env nu
# refactor-gate.nu — confirm green baseline before refactoring, then re-run after.
# Usage:
#   nu skills/refactoring/helpers/refactor-gate.nu          # baseline
#   nu skills/refactoring/helpers/refactor-gate.nu --after  # post-refactor

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--after: bool = false] {
    let label = if $after { "post-refactor" } else { "baseline" }
    let tid = (trace-start "refactoring" "refactor-gate.nu" $label)
    cargo-gate $tid
    trace-end $tid

    if not $after {
        print "Baseline green. Safe to refactor. Re-run with --after when done."
    } else {
        print "All post-refactor gates passed."
    }
}
