#!/usr/bin/env nu
# task-done-sync.nu — PostToolUse/Bash hook
# Fires after every Bash tool call. If the command invoked `godmode task done`
# or `godmode task run ... --auto-done`, automatically runs `godmode task push-done`
# to sync completions back to doob.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")

let is_task_done = (
    ($cmd | str contains "godmode task done") or
    (($cmd | str contains "godmode task run") and ($cmd | str contains "--auto-done"))
)

let is_git_commit = (
    ($cmd | str contains "git commit") and
    not ($cmd | str contains "--no-verify")
)

if not $is_task_done and not $is_git_commit {
    exit 0
}

# Find the repo root from the working directory in the tool input, falling back to cwd.
let work_dir = ($input | get --optional tool_input.cwd | default (pwd))

# If triggered by git commit: find first running task and mark it done with the commit SHA.
if $is_git_commit and not $is_task_done {
    let task_result = do { godmode task list --json } | complete
    if $task_result.exit_code == 0 {
        let tasks = (try { $task_result.stdout | str trim | from json } catch { [] })
        let running = ($tasks | where { |t| ($t | get --optional status | default "") == "running" })
        if ($running | length) > 0 {
            let tid = ($running | first | get id)
            let sha_result = do { git log -1 --format=%H } | complete
            let sha = if $sha_result.exit_code == 0 { $sha_result.stdout | str trim } else { "" }
            if not ($sha | is-empty) {
                do { godmode task done $tid --commit $sha } | complete | ignore
            }
        }
    }
}

let result = do { ^godmode task push-done } | complete

if $result.exit_code != 0 {
    # Degrade gracefully — push-done failure must not abort the session.
    eprintln $"[task-done-sync] push-done exited ($result.exit_code): ($result.stderr)"
}
