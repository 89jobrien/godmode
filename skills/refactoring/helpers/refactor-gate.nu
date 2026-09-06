#!/usr/bin/env nu
# refactor-gate.nu — confirm green baseline before refactoring, then re-run after.
# Usage:
#   nu skills/refactoring/helpers/refactor-gate.nu          # baseline
#   nu skills/refactoring/helpers/refactor-gate.nu --after  # post-refactor

use ../../_lib/trace.nu *
use ../../_lib/helpers.nu *

def main [--after] {
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
