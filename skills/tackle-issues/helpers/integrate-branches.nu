#!/usr/bin/env nu
# integrate-branches.nu — sequentially merge issue branches into main and clean up.
# Usage: nu skills/tackle-issues/helpers/integrate-branches.nu <issue-number>...

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [...issues: string] {
    if ($issues | is-empty) {
        print "Usage: integrate-branches.nu <issue-number>..."
        exit 1
    }

    let tid = (trace-start "tackle-issues" "integrate-branches.nu" ...$issues)
    let root = (repo-root)

    for issue in $issues {
        let wt_path = $"($root)/.worktrees/issue-($issue)"
        if not (worktree-has-commits $wt_path) {
            trace-decision "tackle-issues" "integrate-branches.nu" "agent_incomplete" $issue
            trace-error $tid 1 $"no commits for issue #($issue)"
            print $"ERROR: no commits for issue #($issue)"
            exit 1
        }
        if (is-blocked $wt_path) {
            trace-agent-blocked $"issue-($issue)" $issue "BLOCKED.md found"
            trace-error $tid 1 $"BLOCKED.md for issue #($issue)"
            print $"ERROR: BLOCKED.md for issue #($issue) — escalate to user"
            exit 1
        }
        let commits = (run-external "git" "-C" $wt_path "log" "--oneline" "-3" | complete).stdout | lines | each { str trim }
        trace-agent-complete $"issue-($issue)" $issue $commits
    }

    run-checked $tid "git" "-C" $root "checkout" "main"

    for issue in $issues {
        run-checked $tid "git" "-C" $root "merge" "--no-ff" $"issue/($issue)" "-m" $"merge: issue #($issue)"
        trace-decision "tackle-issues" "integrate-branches.nu" "merged" $issue
    }

    run-checked $tid "cargo" "nextest" "run" "--workspace" $"--manifest-path=($root)/Cargo.toml"
    run-checked $tid "cargo" "clippy" "--workspace" $"--manifest-path=($root)/Cargo.toml" "--" "-D" "warnings"

    for issue in $issues {
        run-external "git" "-C" $root "worktree" "remove" $"($root)/.worktrees/issue-($issue)"
        run-external "git" "-C" $root "branch" "-d" $"issue/($issue)"
    }

    trace-end $tid
    run-external "git" "-C" $root "log" "--oneline" "-5"
}
