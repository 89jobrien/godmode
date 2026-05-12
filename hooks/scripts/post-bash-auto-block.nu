#!/usr/bin/env nu
# post-bash-auto-block.nu — PostToolUse/Bash hook
# Auto-blocks the running task when cargo nextest/test exits nonzero.
# Degrades gracefully. Never blocks tool execution.

use ../lib/godmode-hook-lib.nu [godmode-hook-context]

let ctx = (godmode-hook-context)
if $ctx == null { exit 0 }
if ($ctx.running | length) == 0 { exit 0 }

let cmd = ($ctx.input | get tool_input?.command? | default "")
let exit_code = ($ctx.input | get tool_result?.exit_code? | default 0)

# Only act on test commands that failed
let is_test = (
    ($cmd | str contains "nextest run") or
    ($cmd | str contains "cargo test")
)
if not $is_test { exit 0 }
if $exit_code == 0 { exit 0 }

# Skip if command includes --auto-done (task run handles its own lifecycle)
if ($cmd | str contains "--auto-done") { exit 0 }

# Pick the right task: match -p flag to crate_name, else first running
let crate_flag = (
    try {
        $cmd | parse --regex '-p\s+(?P<crate>\S+)' | get crate?.0? | default ""
    } catch { "" }
)

let target_task = if not ($crate_flag | is-empty) {
    let matched = ($ctx.running | where { |t|
        ($t | get crate_name? | default "") == $crate_flag
    })
    if ($matched | length) > 0 { $matched | first } else { $ctx.running | first }
} else {
    $ctx.running | first
}

let tid = ($target_task | get id? | default "")
if ($tid | is-empty) { exit 0 }

# Extract first failing test name from stdout
let stdout = ($ctx.input | get tool_result?.stdout? | default "")
let failure_line = (
    try {
        $stdout | lines
        | where { |l| ($l | str contains "FAIL") or ($l | str contains "FAILED") }
        | first
        | str trim
    } catch { "test failure" }
)

let reason = $"($failure_line) (exit ($exit_code))"
do { godmode task block $tid $reason } | complete | ignore
print $"[godmode] Auto-blocked task ($tid): ($reason)"

exit 0
