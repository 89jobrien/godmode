#!/usr/bin/env nu
# debug-session.nu — Phase 1 reproduction helper for systematic-debugging.
# Usage: nu skills/systematic-debugging/helpers/debug-session.nu <crate> [test_name]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [crate: string, test_name: string = ""] {
    let tid = (trace-start "systematic-debugging" "debug-session.nu" $crate $test_name)

    run-external "git" "log" "--oneline" "-10"
    run-external "git" "diff" "HEAD~1" "--stat"

    run-checked $tid "cargo" "check" "-p" $crate

    let repro_args = if ($test_name | is-empty) {
        ["cargo" "nextest" "run" "-p" $crate]
    } else {
        ["cargo" "nextest" "run" "-p" $crate "-E" $"test\(($test_name)\)"]
    }

    # Reproduction is expected to fail — don't exit on non-zero, just report
    let repro = (run-external "env" "RUST_BACKTRACE=1" ...$repro_args | complete)
    if $repro.exit_code != 0 {
        trace-decision "systematic-debugging" "debug-session.nu" "reproduced" $"exit=($repro.exit_code)"
        print $repro.stdout
        print $repro.stderr
    }

    trace-end $tid
}
