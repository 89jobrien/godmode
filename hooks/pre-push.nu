#!/usr/bin/env nu
# pre-push.nu — godmode pre-push hook
# Blocks push if any tasks are `running` with no commit SHA attached.
# A running task with a commit is acceptable; no commit means orphaned work.
# Install via: nu hooks/install.nu

use lib/godmode-hook-lib.nu [emit-trace]

def git-root [] {
    let r = (do { git rev-parse --show-toplevel } | complete)
    if $r.exit_code != 0 { error make { msg: "not inside a git repo" } }
    $r.stdout | str trim
}

let root = git-root
let pre_hash = (do { git rev-parse --short HEAD } | complete).stdout | str trim

# Degrade gracefully if godmode not installed
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    print "pre-push: godmode not on PATH — skipping task-state check"
    emit-trace --name "pre-push" --kind "hook" --status "skipped" --output "godmode not on PATH" --last-hash $pre_hash --hooks ["pre-push"]
    exit 0
}

let task_file = $"($root)/.ctx/GODMODE.tasks.yaml"
if not ($task_file | path exists) {
    emit-trace --name "pre-push" --kind "hook" --status "skipped" --output "no task file" --last-hash $pre_hash --hooks ["pre-push"]
    exit 0
}

let result = do { godmode task list --json } | complete
if $result.exit_code != 0 {
    # Degrade gracefully
    emit-trace --name "pre-push" --kind "hook" --status "skipped" --output "task list failed, degraded" --last-hash $pre_hash --hooks ["pre-push"]
    exit 0
}

let tasks = (
    try { $result.stdout | str trim | from json }
    catch { [] }
)

let running_tasks = ($tasks | where { |t| ($t | get --optional status | default "") == "running" })
let orphaned = ($running_tasks | where { |t| ($t | get --optional commit | default "") | is-empty } | get id)

if ($orphaned | length) > 0 {
    print $"pre-push: orphaned running tasks (no commit attached): ($orphaned | str join ', ')"
    print "Mark them done with a commit SHA before pushing:"
    print "  godmode task done <id> --commit <sha>"
    emit-trace --name "pre-push" --kind "hook" --status "error" --output $"orphaned tasks: ($orphaned | str join ', ')" --last-hash $pre_hash --hooks ["pre-push"]
    exit 1
}

print "pre-push: task state OK."
emit-trace --name "pre-push" --kind "hook" --status "ok" --output "task state OK" --last-hash $pre_hash --hooks ["pre-push"]
