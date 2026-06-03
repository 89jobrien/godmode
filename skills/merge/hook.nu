#!/usr/bin/env nu
# hook.nu — PostToolUse/Bash: after a successful git merge, remind about task state sync.
# Always exits 0 (warn only).

use ../_lib/trace.nu *
let _tid = (trace-start "merge" "hook.nu")

let input = open --raw /dev/stdin | from json
let command = $input | get tool_input?.command? | default ""

if not ($command | str contains "git merge") {
    exit 0
}

let exit_code = $input | get tool_result?.exit_code? | default 0
if $exit_code != 0 {
    exit 0
}

let git_root = do { git rev-parse --show-toplevel } | complete
if $git_root.exit_code != 0 { exit 0 }

let task_file = $"($git_root.stdout | str trim)/.ctx/GODMODE.tasks.yaml"
if not ($task_file | path exists) { exit 0 }

let tasks = try { open $task_file | get tasks? | default [] } catch { [] }

let running = (
    $tasks
    | where { |t|
        ($t | get status? | default "") == "running"
        and (($t | get commit? | default "") | is-empty)
    }
    | get id?
    | default []
)

if ($running | length) > 0 {
    let ids = $running | str join ", "
    print --stderr $"[godmode:merge] Merge detected — mark task done: `godmode task done <id> --commit <sha>` (running tasks: ($ids))"
}

trace-end $_tid
exit 0
