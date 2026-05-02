#!/usr/bin/env nu
# task-done-sync.nu — PostToolUse/Bash hook
# Fires after every Bash tool call. If the command invoked `godmode task done`
# or `godmode task run ... --auto-done`, automatically runs `godmode task push-done`
# to sync completions back to doob.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get -i tool_input.command | default "")

let matched = (
    ($cmd | str contains "godmode task done") or
    ($cmd | str contains "godmode task run" and $cmd | str contains "--auto-done")
)

if not $matched {
    exit 0
}

# Find the repo root from the working directory in the tool input, falling back to cwd.
let work_dir = ($input | get -i tool_input.cwd | default (pwd))

let result = do { ^godmode task push-done } | complete

if $result.exit_code != 0 {
    # Degrade gracefully — push-done failure must not abort the session.
    eprintln $"[task-done-sync] push-done exited ($result.exit_code): ($result.stderr)"
}
