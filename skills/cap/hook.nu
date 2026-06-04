#!/usr/bin/env nu
# hook.nu — PostToolUse/Bash hook: after git push, warn if running tasks have no commit SHA.
# Always exits 0 (warn only).

use ../_lib/trace.nu *
let _tid = (trace-start "cap" "hook.nu")

let input = open --raw /dev/stdin | from json

let command = $input | get tool_input?.command? | default ""

# Only act on git push commands
if not ($command | str contains "git push") {
    exit 0
}

# Degrade gracefully if not in a git repo
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let task_file = $"($git_root)/.ctx/godmode/tasks.yaml"
let legacy_task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"
let actual_task_file = if ($task_file | path exists) { $task_file } else if ($legacy_task_file | path exists) { $legacy_task_file } else { exit 0 }

# Parse the task file and find running tasks with no commit field
let tasks = try { open $actual_task_file | get tasks? | default [] } catch { [] }

let unrecorded = (
    $tasks
    | where { |t|
        ($t | get status? | default "") == "running"
        and (($t | get commit? | default "") | is-empty)
    }
    | get id?
    | default []
)

if ($unrecorded | length) > 0 {
    let ids = $unrecorded | str join ", "
    print --stderr $"[godmode:cap] Push detected but running tasks have no commit — run `godmode task done <id> --commit <sha>` (tasks: ($ids))"
}

trace-end $_tid
exit 0
