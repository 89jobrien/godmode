#!/usr/bin/env nu
# pre-skill-trace.nu — PreToolUse/Skill hook
# Emits skill.start into .ctx/godmode/traces/trace.jsonl for every Skill
# invocation (gm-* commands included), so tracing no longer depends on the
# invoked skill having its own instrumented .nu helper. Degrades gracefully.

use ../../skills/_lib/trace.nu [trace-start]

let input = (try { open --raw /dev/stdin | from json } catch { exit 0 })
let skill = ($input | get --optional tool_input.skill | default "")
let tool_use_id = ($input | get --optional tool_use_id | default "" | into string)
if ($skill | is-empty) or ($tool_use_id | is-empty) { exit 0 }

let root_result = do { git rev-parse --show-toplevel } | complete
if $root_result.exit_code != 0 { exit 0 }
let root = ($root_result.stdout | str trim)

let key = ($tool_use_id | str replace --all "/" "_")
let tid = (trace-start $skill "dispatch")

let marker_dir = $"($root)/.ctx/godmode/traces/.pending"
try { mkdir $marker_dir }
try { $"($tid)\n" | save --append $"($marker_dir)/($key).trace_id" }
exit 0
