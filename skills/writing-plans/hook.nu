#!/usr/bin/env nu
# writing-plans/hook.nu — PostToolUse/Write hook
# If a Write creates a file matching docs/plans/*.md, append a trace event to
# .ctx/godmode/traces/trace.jsonl. Always exits 0 (degrades gracefully).

use ../_lib/trace.nu *
let _tid = (trace-start "writing-plans" "hook.nu")

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.path | default "")

# Only care about docs/plans/*.md files
if not ($file_path | str contains "docs/plans/") {
    exit 0
}
if not ($file_path | str ends-with ".md") {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let trace_dir = $"($git_root)/.ctx/godmode/traces"
let trace_file = $"($trace_dir)/trace.jsonl"

# Ensure trace dir exists
if not ($trace_dir | path exists) {
    try { mkdir $trace_dir } catch { exit 0 }
}

let ts = (date now | format date "%Y-%m-%dT%H:%M:%SZ")
let event = ($"{"event":"plan_created","file":"($file_path)","ts":"($ts)"}" | str trim)

try {
    $event | save --append $trace_file
} catch {
    # Degrade gracefully — trace write failures are non-fatal
}

trace-end $_tid
exit 0
