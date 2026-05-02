#!/usr/bin/env nu
# moa/hook.nu — PreToolUse/Bash hook.
# Informs when an LLM API call is detected during an active godmode task.
# Always exits 0 (inform only).

let input = open --raw /dev/stdin | from json

let cmd = (
    try { $input | get tool_input.command? | default "" }
    catch { "" }
)

let llm_patterns = ["openai" "anthropic" "claude"]
let curl_api_pattern = ($cmd | str contains "curl") and ($cmd | str contains "api.")

let is_llm_call = (
    ($llm_patterns | any { |pat| $cmd | str contains $pat }) or $curl_api_pattern
)

if not $is_llm_call {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"

if not ($task_file | path exists) {
    exit 0
}

let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode task list --json } | complete
if $result.exit_code != 0 {
    exit 0
}

let has_running = (
    try {
        let tasks = ($result.stdout | from json)
        ($tasks | where status == "running" | length) > 0
    }
    catch { false }
)

if $has_running {
    print --stderr "[godmode:moa] LLM call detected during active task — consider /godmode:moa for multi-model synthesis"
}

exit 0
