#!/usr/bin/env nu
# post-commit.nu — restore .ctx/ after commit and emit trace
#
# Tools like godmode handoff, doob sync, and session traces update files in
# .ctx/ during the pre-commit phase. These mutations show up as unstaged
# changes and block `git push`. This hook discards those changes so the
# working tree stays clean after a commit.

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

emit-trace --name "post-commit" --kind "hook" --status "ok" --output $output --hooks ["post-commit"]
