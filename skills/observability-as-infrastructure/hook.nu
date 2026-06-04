#!/usr/bin/env nu
# observability-as-infrastructure/hook.nu — PostToolUse/Bash hook.
# Appends a JSONL trace event when godmode state-transition commands are detected.
# Always exits 0 (non-blocking).

use ../_lib/trace.nu *
let _tid = (trace-start "observability-as-infrastructure" "hook.nu")

let input = open --raw /dev/stdin | from json

let cmd = (
    try { $input | get tool_input.command? | default "" }
    catch { "" }
)

let watched = [
    "godmode task start"
    "godmode task done"
    "godmode wave"
    "godmode worktree"
]

let matched = $watched | any { |pat| $cmd | str contains $pat }
if not $matched {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let trace_file = $"($git_root)/.ctx/godmode/traces/trace.jsonl"

let exit_code = (
    try { $input | get tool_response.exit_code? | default 0 }
    catch { 0 }
)

let cmd_short = ($cmd | str substring 0..80)
let ts = (date now | format date "%Y-%m-%dT%H:%M:%S%z")

let session_file = $"($git_root)/.ctx/godmode/session.json"
let sid = if ($session_file | path exists) {
    try { (open $session_file).session_id } catch { "" }
} else { "" }

let event = {
    event: "hook_observed"
    cmd: $cmd_short
    session_id: $sid
    ts: $ts
    exit_code: $exit_code
}

try {
    $event | to json --raw | $"($in)\n" | save --append $trace_file
}

trace-end $_tid
exit 0
