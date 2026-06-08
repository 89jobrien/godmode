#!/usr/bin/env nu
# _lib/tasks.nu — Direct task graph reader for hooks.
#
# Reads .ctx/godmode/tasks.yaml without requiring the godmode binary on PATH.
# Source in hooks:
#   use ../_lib/tasks.nu *

# Load the task graph from the repo root. Returns a list of task records,
# or an empty list if the file is absent or unparseable.
export def load-tasks [] {
    let git_result = do { git rev-parse --show-toplevel } | complete
    if $git_result.exit_code != 0 { return [] }
    let root = $git_result.stdout | str trim

    let task_file = $"($root)/.ctx/godmode/tasks.yaml"
    let legacy_file = $"($root)/.ctx/GODMODE.tasks.yaml"

    let path = if ($task_file | path exists) {
        $task_file
    } else if ($legacy_file | path exists) {
        $legacy_file
    } else {
        return []
    }

    let content = try { open $path } catch { return [] }
    # The file is a record with a `tasks` key containing a list
    let tasks = try { $content | get tasks? | default [] } catch { [] }
    $tasks
}

# Count tasks by status. Returns a record: { done, running, pending, blocked }
export def task-counts [] {
    let tasks = load-tasks
    {
        done: ($tasks | where status == "done" | length)
        running: ($tasks | where status == "running" | length)
        pending: ($tasks | where status == "pending" | length)
        blocked: ($tasks | where status == "blocked" | length)
    }
}

# Return tasks that are running but have no commit recorded.
export def running-no-commit [] {
    let tasks = load-tasks
    $tasks | where {|t|
        (($t | get status? | default "") == "running") and (($t | get commit? | default "") | is-empty)
    }
}

# Return true if any task is currently running.
export def has-running [] {
    let tasks = load-tasks
    ($tasks | where status == "running" | length) > 0
}

# Load wave state from .ctx/godmode/wave-status.json.
# Returns null if absent or unparseable.
export def load-wave [] {
    let git_result = do { git rev-parse --show-toplevel } | complete
    if $git_result.exit_code != 0 { return null }
    let root = $git_result.stdout | str trim
    let wave_file = $"($root)/.ctx/godmode/wave-status.json"
    if not ($wave_file | path exists) { return null }
    try { open $wave_file } catch { null }
}
