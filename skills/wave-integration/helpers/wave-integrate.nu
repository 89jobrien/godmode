#!/usr/bin/env nu
# wave-integrate — sequential rebase + test + merge loop for parallel agent branches
# Usage: wave-integrate [--branches "feat/a feat/b feat/c"] [--base main] [--dry-run]
#
# Reads branches from --branches (space-separated) or from stdin (one per line).
# Rebases each onto --base, runs cargo nextest run, then merges to base.
# Writes .ctx/godmode/conflict-resolution-log.md on real integrations.

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

    if $dry_run {
        let base_check = (do { git rev-parse --verify $base } | complete)
        if $base_check.exit_code != 0 {
            fail $"Base branch ($base) does not exist"
            exit 1
        }

        for branch in $branch_list {
            let branch_check = (do { git rev-parse --verify $branch } | complete)
            if $branch_check.exit_code != 0 {
                fail $"Branch ($branch) does not exist"
                continue
            }
            let has_merges = not (do { git log --merges --format=%H $"($base)..($branch)" } | complete | get stdout | str trim | is-empty)
            let update_kind = if $has_merges { "merge base into branch" } else { "rebase onto base" }
            warn $"[dry-run] Would ($update_kind), test, then merge ($branch) into ($base)"
        }

        print "Dry run complete. No fetch, checkout, stash, rebase, merge, test, or file write was performed."
        return
    }

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

    mut integrated: list<string> = []
    mut integrated_shas: record = {}
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

        # Preserve merge history: branches with merge commits must not be rebased.
        let has_merges = not (do { git log --merges --format=%H $"($base)..($branch)" } | complete | get stdout | str trim | is-empty)
        let update_kind = if $has_merges { "merge" } else { "rebase" }
        step $"Updating ($branch) onto ($base) via ($update_kind)"
        let update_r = if $has_merges {
            do { git merge --no-edit $base } | complete
        } else {
            do { git rebase $base } | complete
        }

        if $update_r.exit_code != 0 {
            warn $"($update_kind) conflicts detected — inspect and resolve manually"
            print $update_r.stderr
            # Check for conflict markers
            let conflicts = (do { git diff --name-only --diff-filter=U } | complete | get stdout | lines | where { |l| ($l | str trim) != "" })
            if ($conflicts | is-empty) {
                fail $"($update_kind) failed with no detectable conflict files — aborting"
                if $has_merges { do { git merge --abort } | complete | ignore } else { do { git rebase --abort } | complete | ignore }
                $failed = ($failed | append $branch)
                continue
            }
            print $"\nConflicted files:"
            for f in $conflicts { print $"  - ($f)" }
            print $"\nResolve conflicts, then continue the ($update_kind) manually."
            print "Then re-run wave-integrate with remaining branches."
            if $has_merges { do { git merge --abort } | complete | ignore } else { do { git rebase --abort } | complete | ignore }
            $failed = ($failed | append $branch)
            continue
        }
        ok $"($update_kind) clean"

        # Run tests
        step "Running cargo nextest run --workspace"
        let test_r = (do { cargo nextest run --workspace } | complete)
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
        let sha = (do { git rev-parse --short HEAD } | complete | get stdout | str trim)

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

        $integrated = ($integrated | append $branch)
        $integrated_shas = ($integrated_shas | insert $branch $sha)
    }

    # Final test run on base
    if ($integrated | length) > 0 {
        step $"Final test run on ($base)"
        let final_r = (do { cargo nextest run --workspace } | complete)
        if $final_r.exit_code != 0 {
            fail "Final tests failed on integration branch — do not proceed"
            exit 1
        }
        ok "All tests pass on integrated branch"
    }

    # Write conflict log template
    let log_dir = $"($repo_root)/.ctx/godmode"
    mkdir $log_dir
    let log_path = $"($log_dir)/conflict-resolution-log.md"
    let timestamp = (date now | format date "%Y-%m-%d %H:%M")
    let shas = $integrated_shas
    let branch_summary = if ($integrated | is-empty) {
        "none"
    } else {
        $integrated | each { |b|
            let sha = ($shas | get -o $b | default "unknown")
            $"- ($b) \(($sha)\)"
        } | str join "\n"
    }
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
