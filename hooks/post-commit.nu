#!/usr/bin/env nu
# post-commit.nu — restore hook-written files after commit and emit trace
#
# Tools like godmode handoff, doob sync, and session traces update files
# during the pre-commit phase. These mutations show up as unstaged changes
# and block `git push`. This hook discards those changes so the working tree
# stays clean after a commit.
#
# Two locations need restoring:
#   - .ctx/       session state, writes the traces and the handoff YAML record
#   - HANDOFF.md  repo root, written by `godmode handoff` so it is tracked
#
# HANDOFF.md is restored only when tracked. In a repo where it is untracked or
# ignored, `git checkout` has nothing to restore and the call is skipped.

use lib/godmode-hook-lib.nu [emit-trace]

let git_root = (run-external "git" "rev-parse" "--show-toplevel" | complete).stdout | str trim
let ctx_dir = $"($git_root)/.ctx"
mut output = ""

if ($ctx_dir | path exists) {
    # Restore tracked .ctx/ files to their committed state.
    let restore = (run-external "git" "checkout" "--" $"($ctx_dir)/" | complete)
    if $restore.exit_code == 0 {
        $output = "restored .ctx/ to committed state"
    } else {
        # Not fatal — .ctx/ may be fully gitignored, in which case checkout
        # returns non-zero because there's nothing to restore.
        $output = "no tracked .ctx/ files to restore"
    }
} else {
    $output = "no .ctx/ directory present"
}

# Restore the tracked root snapshot the pre-commit handoff check rewrote.
let handoff_md = $"($git_root)/HANDOFF.md"
let tracked = (run-external "git" "ls-files" "--error-unmatch" "--" "HANDOFF.md" | complete)
if $tracked.exit_code == 0 {
    let restore_md = (run-external "git" "checkout" "--" $handoff_md | complete)
    if $restore_md.exit_code == 0 {
        $output = $"($output); restored HANDOFF.md to committed state"
    } else {
        $output = $"($output); HANDOFF.md restore failed"
    }
}

emit-trace --name "post-commit" --kind "hook" --status "ok" --output $output --hooks ["post-commit"]
