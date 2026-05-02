# Guardrails: branch guard runner, prose generators, blocked-rule and no-verify-rule prose.

# Verify current branch matches expected. Exit 1 if not.
export def check-branch [expected: string] {
    let current = (do { git branch --show-current } | complete | get stdout | str trim)
    if $current != $expected {
        print $"ERROR: expected branch ($expected), got ($current). Stopping."
        exit 1
    }
}

# Return branch-guard prose for embedding in agent prompts.
export def branch-guard-cmds [expected: string] {
    $"Run: git branch --show-current
   Verify output = ($expected). Stop immediately if not."
}

# Return the 3-attempt / BLOCKED rule prose.
export def blocked-rule [] {
    "If stuck after 3 attempts on any item: write BLOCKED.md at the worktree root. Stop.
Do not retry with identical parameters."
}

# Return the never-no-verify rule prose.
export def no-verify-rule [] {
    "Never use --no-verify on commits. Pre-commit hooks always run."
}
