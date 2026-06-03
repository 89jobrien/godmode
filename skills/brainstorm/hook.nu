#!/usr/bin/env nu
# brainstorm/hook.nu — PreToolUse/Write hook
# If writing to src/ and no design doc exists in docs/plans/ dated today, warn.
# Always exits 0 (warn only, never blocks).

use ../_lib/trace.nu *
let _tid = (trace-start "brainstorm" "hook.nu")

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.path | default "")

# Only care about writes targeting src/
if not ($file_path | str contains "/src/") {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let plans_dir = $"($git_root)/docs/plans"

if not ($plans_dir | path exists) {
    eprintln "[godmode:brainstorm] Writing src/ without a design doc — run /godmode:brainstorm first"
    exit 0
}

let today = (date now | format date "%Y-%m-%d")
let today_docs = (
    try {
        ls $plans_dir
        | where name =~ $"($today)"
        | length
    } catch { 0 }
)

if $today_docs == 0 {
    eprintln "[godmode:brainstorm] Writing src/ without a design doc — run /godmode:brainstorm first"
}

trace-end $_tid
exit 0
