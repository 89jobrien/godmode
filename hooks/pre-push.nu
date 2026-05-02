#!/usr/bin/env nu
# pre-push.nu — godmode pre-push hook
# Blocks push if any tasks are `running` with no commit SHA attached.
# A running task with a commit is acceptable; no commit means orphaned work.
# Install via: nu hooks/install.nu

def git-root [] {
    let r = (do { git rev-parse --show-toplevel } | complete)
    if $r.exit_code != 0 { error make { msg: "not inside a git repo" } }
    $r.stdout | str trim
}

let root = git-root

# Degrade gracefully if godmode not installed
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    print "pre-push: godmode not on PATH — skipping task-state check"
    exit 0
}

let task_file = $"($root)/.ctx/GODMODE.tasks.yaml"
if not ($task_file | path exists) {
    exit 0
}

let result = do { godmode task list --json } | complete
if $result.exit_code != 0 {
    # Degrade gracefully
    exit 0
}

let tasks = (
    try { $result.stdout | str trim | from json }
    catch { [] }
)

let orphaned = (
    $tasks
    | where { |t|
        ($t | get --optional status | default "") == "running"
        and (($t | get --optional commit | default "") | is-empty)
    }
    | get id
)

if ($orphaned | length) > 0 {
    print $"pre-push: orphaned running tasks (no commit attached): ($orphaned | str join ', ')"
    print "Mark them done with a commit SHA before pushing:"
    print "  godmode task done <id> --commit <sha>"
    exit 1
}

print "pre-push: task state OK."
