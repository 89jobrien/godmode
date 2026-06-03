#!/usr/bin/env nu
# task-driven-development/hook.nu — PreToolUse/Bash hook
# If the command is `cargo build` or `cargo run` (not test/nextest/check/clippy)
# and no test files were modified in the last git commit, warn.
# Always exits 0 (warn only, never blocks).

use ../_lib/trace.nu *
let _tid = (trace-start "task-driven-development" "hook.nu")

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")

# Only care about cargo build / cargo run invocations
let is_build = ($cmd | str contains "cargo build") or ($cmd | str contains "cargo run")
if not $is_build {
    exit 0
}

# Exclude test/check/clippy variants — they are fine
let is_test_like = (
    ($cmd | str contains "cargo test") or
    ($cmd | str contains "cargo nextest") or
    ($cmd | str contains "cargo check") or
    ($cmd | str contains "cargo clippy")
)
if $is_test_like {
    exit 0
}

# Check git availability
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

# Get files changed in the last commit
let diff_result = do { git diff HEAD --name-only } | complete
if $diff_result.exit_code != 0 {
    exit 0
}

let changed_files = ($diff_result.stdout | str trim | lines)

# Check whether any test files were modified
let has_test_changes = (
    $changed_files | any { |f|
        ($f | str contains "test") or ($f | str ends-with "_test.rs")
    }
)

if not $has_test_changes {
    eprintln "[godmode:tdd] Building without tests — write a failing test first (see tdd-tasks.yaml)"
}

trace-end $_tid
exit 0
