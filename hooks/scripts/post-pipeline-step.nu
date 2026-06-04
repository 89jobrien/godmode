#!/usr/bin/env nu
# post-pipeline-step — observability hook for pipeline state transitions.
#
# Fires on PostToolUse/Bash. Detects `godmode pipeline next` or
# `godmode pipeline skip` in tool output and appends a
# `pipeline.step.done` event to the session trace JSONL.
#
# BOUNDARY RULE: This hook MUST NOT call `godmode pipeline next`,
# `skip`, or any other state-mutating command. It is observability only.

let input = (open --raw /dev/stdin | from json)

# Only act on Bash tool results.
let tool_name = ($input | get -i tool_name | default "")
if $tool_name != "Bash" { exit 0 }

let output = ($input | get -i tool_result.stdout | default "")

# Detect pipeline state transitions in the output.
let is_next = ($output | str contains "pipeline next" or ($output | str contains "Advanced to:"))
let is_skip = ($output | str contains "pipeline skip" or ($output | str contains "Pipeline complete."))

if not $is_next and not $is_skip { exit 0 }

# Find the git root.
let git_root = (do { git rev-parse --show-toplevel } | complete)
if $git_root.exit_code != 0 { exit 0 }
let root = ($git_root.stdout | str trim)

let sessions_dir = $"($root)/.ctx/godmode/sessions"
if not ($sessions_dir | path exists) { exit 0 }

# Read current pipeline state for context.
let state_file = $"($root)/.ctx/godmode/pipeline.yaml"
let pipeline_name = if ($state_file | path exists) {
    open $state_file | get -i active | default "unknown"
} else {
    "unknown"
}
let step_index = if ($state_file | path exists) {
    open $state_file | get -i current_step | default 0
} else {
    0
}

# Determine the operation type.
let op = if $is_next { "advance" } else { "skip" }

# Build trace event.
let event = {
    event: "pipeline.step.done"
    pipeline: $pipeline_name
    step_index: $step_index
    operation: $op
    timestamp: (date now | format date "%Y-%m-%dT%H:%M:%SZ")
}

# Append to today's session JSONL.
let today = (date now | format date "%Y-%m-%d")
let trace_file = $"($sessions_dir)/($today).jsonl"
$event | to json --raw | save --append $trace_file

exit 0
