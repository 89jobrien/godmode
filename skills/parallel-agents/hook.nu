#!/usr/bin/env nu
# parallel-agents/hook.nu — PostToolUse/Agent hook
# After any Agent tool call, checks for running tasks with no commit.
# Always exits 0. Degrades gracefully.

let input = open --raw /dev/stdin | from json

# Find git root; bail silently if not in a git repo
let git_root_result = do { git rev-parse --show-toplevel } | complete
if $git_root_result.exit_code != 0 {
    exit 0
}

let git_root = $git_root_result.stdout | str trim
let task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"

if not ($task_file | path exists) {
    exit 0
}

# Check godmode is on PATH
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode task list --json } | complete
if $result.exit_code != 0 {
    exit 0
}

let tasks = do { $result.stdout | from json } | complete
if $tasks.exit_code != 0 {
    exit 0
}

# Find running tasks with no commit recorded
let running_no_commit = $tasks.output | where {|t|
    let status = try { $t.status | default "" } catch { "" }
    let commit = try { $t.commit | default "" } catch { "" }
    $status == "running" and ($commit | is-empty)
}

if ($running_no_commit | length) > 0 {
    print "[godmode:parallel-agents] Running tasks detected with no commit — verify subagents committed their work: `godmode task list`"
}

exit 0
