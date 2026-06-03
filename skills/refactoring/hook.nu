#!/usr/bin/env nu
# hook.nu — PreToolUse/Edit hook for refactoring skill
# Fires before every Edit tool call. Warns if no task is currently running in the
# godmode task graph — edits without a running task are likely untracked refactors.

use ../_lib/trace.nu *
let _tid = (trace-start "refactoring" "hook.nu")

let input = open --raw /dev/stdin | from json

# Resolve the task file path — degrade gracefully if git root cannot be found
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    # Not a git repo or git not available — skip silently
    exit 0
}

let repo_root = ($git_result.stdout | str trim)
let task_file = ($repo_root | path join ".ctx" "GODMODE.tasks.yaml")

if not ($task_file | path exists) {
    # No task file — plugin not in use for this session, skip silently
    exit 0
}

# Check if godmode is available
let gm_check = do { which godmode } | complete
if $gm_check.exit_code != 0 {
    exit 0
}

let status_result = do { godmode status --json } | complete
if $status_result.exit_code != 0 {
    exit 0
}

let status = (try { $status_result.stdout | str trim | from json } catch { null })
if $status == null {
    exit 0
}

let running = ($status | get --optional running | default 0)

if $running == 0 {
    eprintln "[godmode:refactoring] No task running during edit — start a task before refactoring: `godmode task start <id>`"
}

trace-end $_tid
exit 0
