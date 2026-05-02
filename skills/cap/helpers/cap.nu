#!/usr/bin/env nu
# cap.nu — validate, stage, commit, and push.
# Usage: nu skills/cap/helpers/cap.nu "<commit message>"

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [msg: string = ""] {
    assert-not-main

    let tid = (trace-start "cap" "cap.nu" $msg)

    # Check + test first (cheap); fmt auto-fix if needed
    cargo-test-gate $tid
    let fmt = (run-external "cargo" "fmt" "--all" "--" "--check" | complete)
    if $fmt.exit_code != 0 {
        trace-decision "cap" "cap.nu" "auto_fmt" "ran cargo fmt"
        run-external "cargo" "fmt" "--all"
    }
    run-checked $tid "cargo" "clippy" "--workspace" "--" "-D" "warnings"

    run-external "git" "add" "-A"
    run-external "git" "diff" "--cached" "--stat"

    let commit_msg = if ($msg | is-empty) {
        let diff = (run-external "git" "diff" "--cached" "--stat" | complete).stdout
        $"chore: ($diff | lines | first | str trim)"
    } else {
        $msg
    }

    run-checked $tid "git" "commit" "-m" $commit_msg
    run-checked $tid "git" "push"
    trace-end $tid
    run-external "git" "log" "--oneline" "-3"
}
