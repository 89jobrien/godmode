#!/usr/bin/env nu
# post-skill-trace.nu — PostToolUse/Skill hook
# Closes out the skill.start emitted by pre-skill-trace.nu with skill.complete
# or skill.error, so every Skill invocation shows up in `/gm-trace` even when
# the underlying skill has no instrumented helper of its own.

use ../../skills/_lib/trace.nu [trace-end trace-error]

let input = (try { open --raw /dev/stdin | from json } catch { exit 0 })
let skill = ($input | get --optional tool_input.skill | default "")
let tool_use_id = ($input | get --optional tool_use_id | default "" | into string)
if ($skill | is-empty) or ($tool_use_id | is-empty) { exit 0 }

let root_result = do { git rev-parse --show-toplevel } | complete
if $root_result.exit_code != 0 { exit 0 }
let root = ($root_result.stdout | str trim)

let key = ($tool_use_id | str replace --all "/" "_")
let marker = $"($root)/.ctx/godmode/traces/.pending/($key).trace_id"
if not ($marker | path exists) { exit 0 }
let tids = (try { open --raw $marker | lines | where { not ($in | str trim | is-empty) } } catch { [] })
if ($tids | is-empty) { try { rm $marker }; exit 0 }
let tid = ($tids | first | str trim)
let remaining = ($tids | skip 1)
if ($remaining | is-empty) {
    try { rm $marker }
} else {
    try { $"($remaining | str join (char newline))(char newline)" | save --force $marker }
}

let hook_error = ($input | get --optional error | default "" | into string)
let is_error = (
    ($input | get --optional tool_response.is_error | default false) or
    not ($hook_error | is-empty)
)
if $is_error {
    let stderr_tail = if ($hook_error | is-empty) {
        $input | get --optional tool_response.content | default "" | into string
    } else {
        $hook_error
    }
    trace-error $tid 1 $stderr_tail
} else {
    trace-end $tid
}
exit 0
