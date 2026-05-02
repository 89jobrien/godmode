# Dispatch: worktree setup/teardown, agent prompt builder, wave-state init, integration loop.

# Create a git worktree for an issue/task branch.
export def setup-worktree [repo_root: string, issue: int] {
    let path = $"($repo_root)/.worktrees/issue-($issue)"
    let branch = $"issue/($issue)"
    git -C $repo_root worktree add $path -b $branch
}

# Remove a worktree and delete its branch.
export def teardown-worktree [repo_root: string, issue: int] {
    let path = $"($repo_root)/.worktrees/issue-($issue)"
    let branch = $"issue/($issue)"
    git -C $repo_root worktree remove $path
    git -C $repo_root branch -d $branch
}

# Build a self-contained agent prompt for a worktree-based issue agent.
export def agent-prompt [repo_root: string, issue: int, title: string, body: string] {
    let wt = $"($repo_root)/.worktrees/issue-($issue)"
    let branch = $"issue/($issue)"
    $"You are implementing GitHub issue #($issue): ($title)

Worktree absolute path: ($wt)
Branch: ($branch)

Issue body:
($body)

Workflow:
1. Run: git -C ($wt) branch --show-current
   Verify output = ($branch). Stop immediately if not.
2. Read all files listed in the issue body before writing anything.
   Use absolute paths — do NOT use cd.
3. For each file to create/modify:
   a. Write a FAILING test if applicable.
      Run: cargo nextest run --workspace --manifest-path ($repo_root)/Cargo.toml
      Confirm FAIL.
   b. Implement minimum code to pass.
      Run: cargo nextest run --workspace --manifest-path ($repo_root)/Cargo.toml
      All green.
   c. Run: cargo clippy --workspace --manifest-path ($repo_root)/Cargo.toml -- -D warnings
      Zero warnings.
4. Commit:
   git -C ($wt) add -A
   git -C ($wt) commit -m \"feat: <summary> fixes #($issue)\"
5. Final check:
   cargo nextest run --workspace --manifest-path ($repo_root)/Cargo.toml
   cargo clippy --workspace --manifest-path ($repo_root)/Cargo.toml -- -D warnings
6. Report: files created, tests added, commit SHA, any blockers.

If stuck after 3 attempts: write BLOCKED.md at ($wt)/BLOCKED.md. Stop.
Do NOT modify files outside ($wt)."
}

# Emit initial wave-status JSON for a list of agent names.
export def wave-state-init [agents: list<string>] {
    let entries = ($agents | each { |a|
        $"    \"($a)\": {\"status\": \"pending\", \"branch\": \"\", \"commits\": []}"
    } | str join ",\n")
    $"{\n  \"wave\": 1,\n  \"agents\": {\n($entries)\n  }\n}"
}

# Merge a list of branches into main sequentially with --no-ff.
# Prints conflict message and exits on first failure.
export def integrate-branches [repo_root: string, branches: list<string>] {
    git -C $repo_root checkout main
    for branch in $branches {
        let result = (do { git -C $repo_root merge --no-ff $branch -m $"merge: ($branch)" } | complete)
        if $result.exit_code != 0 {
            print $"CONFLICT merging ($branch) — resolve manually before continuing."
            exit 1
        }
    }
    let suite = (do { cargo nextest run --workspace --manifest-path $"($repo_root)/Cargo.toml" } | complete)
    if $suite.exit_code != 0 { exit $suite.exit_code }
}
