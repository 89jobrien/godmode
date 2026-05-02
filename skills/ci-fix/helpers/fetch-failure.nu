#!/usr/bin/env nu
# fetch-failure.nu — fetch the latest failed CI run logs for classification.
# Usage: nu skills/ci-fix/helpers/fetch-failure.nu [run-id]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [run_id: string = ""] {
    let tid = (trace-start "ci-fix" "fetch-failure.nu" $run_id)

    let id = if ($run_id | is-empty) {
        let branch = (git-branch)
        let runs = (run-external "gh" "run" "list" "--branch" $branch "--status" "failure"
            "--limit" "1" "--json" "databaseId" | complete).stdout | from json
        if ($runs | is-empty) {
            trace-decision "ci-fix" "fetch-failure.nu" "no_failures" $branch
            trace-end $tid
            print "No failed runs on current branch."
            exit 0
        }
        $runs | first | get databaseId | into string
    } else {
        $run_id
    }

    trace-decision "ci-fix" "fetch-failure.nu" "run_id" $id
    run-checked $tid "gh" "run" "view" $id "--log-failed"
    trace-end $tid
}
