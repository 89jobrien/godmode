#!/usr/bin/env nu
# review-gate.nu — confirm green baseline before and after applying review fixes.
# Usage:
#   nu skills/receiving-review/helpers/review-gate.nu           # baseline
#   nu skills/receiving-review/helpers/review-gate.nu --after   # post-fix

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--after: bool = false] {
    let label = if $after { "post-fix" } else { "baseline" }
    let tid = (trace-start "receiving-review" "review-gate.nu" $label)
    cargo-gate $tid
    trace-end $tid

    if $after { print "Post-fix gates passed." } else { print "Baseline green. Apply review fixes, then re-run with --after." }
}
