#!/usr/bin/env nu
# task-management/hook.nu — SessionStart hook
# If .ctx/godmode/tasks.yaml exists, prints a one-line status summary.
# Always exits 0. Degrades gracefully.

use ../_lib/trace.nu *
let _tid = (trace-start "task-management" "hook.nu")

let input = open --raw /dev/stdin | from json

# Find git root; bail silently if not in a git repo
let git_root_result = do { git rev-parse --show-toplevel } | complete
if $git_root_result.exit_code != 0 {
    exit 0
}

let git_root = $git_root_result.stdout | str trim
let task_file = $"($git_root)/.ctx/godmode/tasks.yaml"
let legacy_task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"

if not (($task_file | path exists) or ($legacy_task_file | path exists)) {
    exit 0
}

# Check godmode is on PATH
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode status --json } | complete
if $result.exit_code != 0 {
    exit 0
}

let status = do { $result.stdout | from json } | complete
if $status.exit_code != 0 {
    exit 0
}

let s = $status.output
let done_count    = try { $s.done    | default 0 } catch { 0 }
let running_count = try { $s.running | default 0 } catch { 0 }
let pending_count = try { $s.pending | default 0 } catch { 0 }
let blocked_count = try { $s.blocked | default 0 } catch { 0 }

print $"[godmode] ($done_count) done / ($running_count) running / ($pending_count) pending / ($blocked_count) blocked"

if $blocked_count > 0 {
    print "  blocked: run `godmode task unblock-all` or `godmode task list` to review"
}

trace-end $_tid
exit 0
