#!/usr/bin/env nu
# wave-integrate — sequential rebase + test + merge loop for parallel agent branches
# Usage: wave-integrate [--branches "feat/a feat/b feat/c"] [--base main] [--dry-run]
#
# Reads branches from --branches (space-separated) or from stdin (one per line).
# Rebases each onto --base, runs cargo test --workspace, then merges to base.
# Writes conflict-resolution-log.md to cwd on completion.

def ok   [msg: string] { print $"  (ansi green)✓(ansi reset)  ($msg)" }
def fail [msg: string] { print $"  (ansi red)✗(ansi reset)  ($msg)" }
def step [msg: string] { print $"\n  (ansi cyan_bold)▸(ansi reset)  (ansi attr_bold)($msg)(ansi reset)" }
def warn [msg: string] { print $"  (ansi yellow)!(ansi reset)  ($msg)" }

def main [
    --branches: string = ""   # Space-separated branch list
    --base: string = "main"   # Integration target branch
    --dry-run                 # Rebase and test but do not merge or commit
] {
    let repo_root = (git rev-parse --show-toplevel | str trim)
    cd $repo_root

    # Resolve branch list
    let branch_list = if ($branches | str trim | is-empty) {
        $in | lines | where { |l| ($l | str trim) != "" }
    } else {
        $branches | split row " " | where { |l| ($l | str trim) != "" }
    }

    if ($branch_list | is-empty) {
        print "No branches provided. Use --branches 'feat/a feat/b' or pipe branch names."
        exit 1
    }

    print $"\nWave integration: ($branch_list | length) branches onto ($base)"
    print $"Branches: ($branch_list | str join ', ')\n"

    # Stash any dirty worktree
    let dirty_unstaged = (do { git diff --quiet } | complete).exit_code != 0
    let dirty_staged   = (do { git diff --cached --quiet } | complete).exit_code != 0
    mut stashed = false
    if $dirty_unstaged or $dirty_staged {
        step "Stashing uncommitted changes"
        do { git stash push -m "wave-integrate: auto-stash" } | complete | ignore
        $stashed = true
        ok "Stashed"
    }

    # Pull latest base
    step $"Updating ($base)"
    let pull_r = (do { git checkout $base } | complete)
    if $pull_r.exit_code != 0 { fail $"Failed to checkout ($base)"; exit 1 }
    let pull_r2 = (do { git pull } | complete)
    if $pull_r2.exit_code != 0 { fail $"Failed to pull ($base)"; exit 1 }
    ok $"($base) up to date"

    mut log_rows: list<record> = []
    mut integrated: list<string> = []
    mut failed: list<string> = []

    for branch in $branch_list {
        print $"\n━━━ ($branch) ━━━"

        # Checkout branch
        let co_r = (do { git checkout $branch } | complete)
        if $co_r.exit_code != 0 {
            fail $"Cannot checkout ($branch) — skipping"
            $failed = ($failed | append $branch)
            continue
        }

        # Fetch latest
        do { git fetch origin $branch } | complete | ignore

        # Stash any dirty state before rebase (can accumulate from prior branches)
        let pre_unstaged = (do { git diff --quiet } | complete).exit_code != 0
        let pre_staged   = (do { git diff --cached --quiet } | complete).exit_code != 0
        if $pre_unstaged or $pre_staged {
            do { git stash push -m $"wave-integrate: pre-rebase stash for ($branch)" } | complete | ignore
        }

        # Rebase onto base
        step $"Rebasing ($branch) onto ($base)"
        let rebase_r = (do { git rebase $base } | complete)

        if $rebase_r.exit_code != 0 {
            warn "Rebase conflicts detected — inspect and resolve manually"
            print $rebase_r.stderr
            # Check for conflict markers
            let conflicts = (do { git diff --name-only --diff-filter=U } | complete | get stdout | lines | where { |l| ($l | str trim) != "" })
            if ($conflicts | is-empty) {
                fail "Rebase failed with no detectable conflict files — aborting rebase"
                do { git rebase --abort } | complete | ignore
                $failed = ($failed | append $branch)
                continue
            }
            print $"\nConflicted files:"
            for f in $conflicts { print $"  - ($f)" }
            print "\nResolve conflicts, then run: git add <files> && git rebase --continue"
            print "Then re-run wave-integrate with remaining branches."
            do { git rebase --abort } | complete | ignore
            $failed = ($failed | append $branch)
            continue
        }
        ok "Rebase clean"

        # Run tests (xtask test-unit skips Linux-only e2e tests on macOS)
        step "Running cargo xtask test-unit"
        let test_r = (do { cargo xtask test-unit } | complete)
        if $test_r.exit_code != 0 {
            fail "Tests failed after rebase"
            print ($test_r.stdout | lines | last 30 | str join "\n")
            warn "Fix tests on this branch before continuing"
            $failed = ($failed | append $branch)
            do { git checkout $base } | complete | ignore
            continue
        }
        ok "Tests pass"

        # Get final SHA
        let sha = (git rev-parse --short HEAD | str trim)

        if not $dry_run {
            # Merge to base
            step $"Merging ($branch) into ($base)"
            do { git checkout $base } | complete | ignore
            let merge_r = (do { git merge --no-ff $branch -m $"integrate: merge ($branch)" } | complete)
            if $merge_r.exit_code != 0 {
                fail $"Merge failed for ($branch)"
                $failed = ($failed | append $branch)
                continue
            }
            ok $"Merged ($branch) -> ($base) at ($sha)"
        } else {
            warn $"[dry-run] Would merge ($branch) at ($sha) into ($base)"
            do { git checkout $base } | complete | ignore
        }

        $integrated = ($integrated | append $branch)
        $log_rows = ($log_rows | append {branch: $branch, sha: $sha, status: "integrated"})
    }

    # Final test run on base
    if not $dry_run and ($integrated | length) > 0 {
        step $"Final test run on ($base)"
        let final_r = (do { cargo test --workspace } | complete)
        if $final_r.exit_code != 0 {
            fail "Final tests failed on integration branch — do not proceed"
            exit 1
        }
        ok "All tests pass on integrated branch"
    }

    # Write conflict log template
    let log_path = $"($repo_root)/conflict-resolution-log.md"
    let timestamp = (date now | format date "%Y-%m-%d %H:%M")
    let branch_summary = ($integrated | each { |b| $"- ($b)" } | str join "\n")
    let failed_summary = if ($failed | is-empty) { "none" } else { $failed | str join ", " }

    let log_content = $"# Wave Integration Log — ($timestamp)

## Branches Integrated

($branch_summary)

## Failed / Skipped

($failed_summary)

## Conflict Resolution Log

| File | Branch | Main-side intent | Branch-side intent | Resolution |
|------|--------|-----------------|-------------------|------------|
| _fill in_ | | | | |

## Notes

_Add any manual resolution notes here._
"
    $log_content | save --force $log_path
    ok $"Conflict log template written to ($log_path)"

    # Restore stash if we stashed at the start
    if $stashed {
        step "Restoring stashed changes"
        do { git checkout $base } | complete | ignore
        let pop_r = (do { git stash pop } | complete)
        if $pop_r.exit_code != 0 {
            warn "Could not restore stash — run 'git stash pop' manually"
        } else {
            ok "Stash restored"
        }
    }

    # Summary
    print $"\n━━━ Summary ━━━"
    print $"  Integrated : ($integrated | length) branches"
    print $"  Failed     : ($failed | length) branches"
    if ($failed | length) > 0 {
        print $"  Failed list: ($failed | str join ', ')"
    }
}
