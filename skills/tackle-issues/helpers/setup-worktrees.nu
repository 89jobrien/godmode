#!/usr/bin/env nu
# setup-worktrees.nu — prepare worktrees for parallel issue dispatch.
# Usage: nu skills/tackle-issues/helpers/setup-worktrees.nu <issue-number>...

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [...issues: string] {
    if ($issues | is-empty) {
        print "Usage: setup-worktrees.nu <issue-number>..."
        exit 1
    }

    let tid = (trace-start "tackle-issues" "setup-worktrees.nu" ...$issues)
    let root = (repo-root)
    let gitignore = $"($root)/.gitignore"

    if not (open $gitignore | str contains ".worktrees/") {
        $".worktrees/\n" | save --append $gitignore
    }

    run-checked $tid "git" "-C" $root "fetch" "origin" "main"

    for issue in $issues {
        let wt_path = (worktree-path $root $issue)
        if ($wt_path | path exists) {
            trace-decision "tackle-issues" "setup-worktrees.nu" "removed_stale_worktree" $issue
            run-external "git" "-C" $root "worktree" "remove" "--force" $wt_path
        }
        run-checked $tid "git" "-C" $root "worktree" "add" $wt_path "-b" $"issue/($issue)" "origin/main"
        trace-agent-start $"issue-($issue)" $issue ""
    }

    trace-end $tid
    run-external "git" "-C" $root "worktree" "list"
}
