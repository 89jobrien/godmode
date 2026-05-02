#!/usr/bin/env nu
# pre-bash-nag.nu — PreToolUse/Bash hook
# If no tasks are running but pending tasks exist, warn before any Bash command.
# Always approves — never blocks.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")

# Skip godmode commands themselves to avoid recursion noise
if ($cmd | str starts-with "godmode") {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"

if not ($task_file | path exists) {
    exit 0
}

let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode status --json } | complete
if $result.exit_code != 0 {
    exit 0
}

let status = (
    try { $result.stdout | str trim | from json }
    catch { null }
)

if $status == null {
    exit 0
}

let running = ($status | get --optional running | default 0)
let pending = ($status | get --optional pending | default 0)

if $running == 0 and $pending > 0 {
    let next_ids = ($status | get --optional next | default [] | str join ", ")
    eprintln $"[godmode] No task running. ($pending) pending. Start one: godmode task start ($next_ids)"
}

exit 0
