#!/usr/bin/env nu
# cap.nu — validate, stage, commit, and push.
# Usage: nu skills/cap/helpers/cap.nu "<commit message>"

use ../../_lib/trace.nu *
use ../../_lib/helpers.nu *

def main [msg: string = ""] {
    assert-not-main

    let tid = (trace-start "cap" "cap.nu" $msg)

    # No gate pre-flight here. The global git hooks gate every commit and push
    # (format check, clippy on affected crates, tests, secret scan). Stage and
    # commit; a failing hook blocks the commit and reports what to fix.

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
