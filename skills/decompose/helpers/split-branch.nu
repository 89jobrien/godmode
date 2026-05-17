#!/usr/bin/env nu
# split-branch.nu — Creates a split branch from a source branch, stages the specified files,
# verifies with cargo check + nextest, and commits.
#
# Usage:
#   nu split-branch.nu \
#     --source feat/big-pr \
#     --base main \
#     --split-id 1 \
#     --branch feat/big-pr-split-1-core \
#     --files "crates/foo/src/lib.rs crates/foo/src/bar.rs" \
#     [--output-dir .ctx/godmode/decomps/my-decomp] \
#     [--dry-run]

def main [
    --source: string    # Source branch (read-only — never modified)
    --base: string = "main"
    --split-id: int = 1
    --branch: string    # New branch name to create
    --files: string     # Space-separated list of files to stage
    --output-dir: string = ".ctx/godmode/decomps"  # Where to write split state
    --dry-run           # Print plan without executing
] {
    let file_list = $files | split row " " | where { |f| not ($f | is-empty) }

    if ($file_list | length) == 0 {
        error make { msg: "No files specified for split" }
    }

    # Confirm source branch exists
    let source_check = do { git rev-parse --verify $source } | complete
    if $source_check.exit_code != 0 {
        error make { msg: $"Source branch '($source)' not found" }
    }

    if $dry_run {
        print $"[dry-run] Would create branch: ($branch)"
        print $"[dry-run] From base: ($base)"
        print $"[dry-run] Files to stage from ($source):"
        for f in $file_list { print $"  ($f)" }
        return
    }

    # Safety: ensure we are not on the source branch before mutating anything
    let current = do { git branch --show-current } | complete | get stdout | str trim
    if $current == $source {
        git checkout $base
    }

    # Create the split branch from base
    let create = do { git checkout -b $branch $base } | complete
    if $create.exit_code != 0 {
        error make { msg: $"Failed to create branch ($branch): ($create.stderr)" }
    }

    # Stage files from source branch
    for file in $file_list {
        let stage = do { git checkout $source -- $file } | complete
        if $stage.exit_code != 0 {
            # Roll back and report
            git checkout $base
            git branch -D $branch
            error make { msg: $"Failed to stage ($file) from ($source): ($stage.stderr)" }
        }
    }

    # Verify: cargo check
    let check = do { cargo check --workspace } | complete
    if $check.exit_code != 0 {
        git checkout $base
        git branch -D $branch
        error make { msg: $"cargo check failed for split ($split_id):\n($check.stderr)" }
    }

    # Verify: nextest (scoped to affected crates if possible)
    let test = do { cargo nextest run --workspace } | complete
    if $test.exit_code != 0 {
        git checkout $base
        git branch -D $branch
        error make { msg: $"cargo nextest failed for split ($split_id):\n($test.stderr)" }
    }

    # Commit
    let staged = do { git diff --cached --name-only } | complete | get stdout | str trim
    if ($staged | is-empty) {
        git checkout $base
        git branch -D $branch
        error make { msg: $"Split ($split_id): no staged changes after checkout — files may already match base" }
    }

    git commit -m $"chore(decompose): split ($split_id) — ($branch)"

    let sha = do { git rev-parse HEAD } | complete | get stdout | str trim

    # Write state record
    let state_dir = $output_dir
    mkdir $state_dir
    {
        split_id: $split_id
        branch: $branch
        source: $source
        base: $base
        files: $file_list
        sha: $sha
        status: "ok"
    } | to json | save --force $"($state_dir)/split-($split_id).json"

    # Return to base
    git checkout $base

    print $"[split ($split_id)] ($branch) @ ($sha) — ($file_list | length) files — OK"
}
