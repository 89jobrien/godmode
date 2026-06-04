#!/usr/bin/env nu
# using-godmode/hook.nu — SessionStart hook
# If no task graph exists, prints an orientation hint.
# Always exits 0. Degrades gracefully.

use ../_lib/trace.nu *
let _tid = (trace-start "using-godmode" "hook.nu")

let input = open --raw /dev/stdin | from json

# Find git root; bail silently if not in a git repo
let git_root_result = do { git rev-parse --show-toplevel } | complete
if $git_root_result.exit_code != 0 {
    exit 0
}

let git_root = $git_root_result.stdout | str trim
let task_file = $"($git_root)/.ctx/godmode/tasks.yaml"
let legacy_task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"

# Only print if the task graph does NOT exist
if (($task_file | path exists) or ($legacy_task_file | path exists)) {
    exit 0
}

# Check godmode is on PATH before mentioning its commands
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

print "[godmode] No task graph found. Run `godmode task add <id> <title>` or `godmode plan ingest <path>` to start."

trace-end $_tid
exit 0
