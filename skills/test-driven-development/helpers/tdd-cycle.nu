#!/usr/bin/env nu
# tdd-cycle.nu — red/green/refactor gate runner.
# Usage: nu skills/test-driven-development/helpers/tdd-cycle.nu <crate> <test-name>

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [crate: string, test_name: string] {
    let tid = (trace-start "test-driven-development" "tdd-cycle.nu" $crate $test_name)

    # RED — must fail before impl
    let red = (run-external "env" "RUST_BACKTRACE=1" "cargo" "nextest" "run"
        "-p" $crate "-E" $"test\(($test_name)\)" | complete)
    if $red.exit_code == 0 {
        trace-decision "test-driven-development" "tdd-cycle.nu" "red_failed" "test passed before impl"
        trace-error $tid 1 "test passed before implementation"
        print "ERROR: test passed before implementation — write a real failing test first"
        exit 1
    }

    trace-decision "test-driven-development" "tdd-cycle.nu" "red_confirmed" $test_name
    print "RED confirmed. Implement, then press enter."
    input

    # GREEN
    run-checked $tid "cargo" "nextest" "run" "-p" $crate
    trace-decision "test-driven-development" "tdd-cycle.nu" "green_confirmed" $crate

    # REFACTOR gate
    cargo-gate $tid --crate $crate

    trace-end $tid
    print $"Green and clean. Commit with: git commit -m \"feat\(($crate)\): <what it does>\""
}
