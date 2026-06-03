#!/usr/bin/env nu
# hook.nu — PostToolUse/Bash hook: after gh pr create, suggest running code-review skill.
# Always exits 0 (warn only).

use ../_lib/trace.nu *
let _tid = (trace-start "code-review" "hook.nu")

let input = open --raw /dev/stdin | from json

let command = $input | get tool_input?.command? | default ""

if not ($command | str contains "gh pr create") {
    exit 0
}

print --stderr "[godmode:code-review] PR created — run /godmode:code-review for a systematic quality pass before requesting review"

trace-end $_tid
exit 0
