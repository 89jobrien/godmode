#!/usr/bin/env nu
# hook.nu — PreToolUse/Edit hook for receiving-review skill
# Fires before every Edit tool call. If there is an open PR for the current branch
# and the file being edited is under src/, warns to process review comments first.

use ../_lib/trace.nu *
let _tid = (trace-start "receiving-review" "hook.nu")

let input = open --raw /dev/stdin | from json

let file_path = ($input | get --optional tool_input.path | default "")

# Only warn on src/ edits
if not ($file_path | str contains "/src/") {
    exit 0
}

# Check that gh is available
let gh_check = do { which gh } | complete
if $gh_check.exit_code != 0 {
    exit 0
}

# Check for an open PR on the current branch
let pr_result = do { gh pr view --json state } | complete
if $pr_result.exit_code != 0 {
    # No PR or gh error — skip silently
    exit 0
}

let pr = (try { $pr_result.stdout | str trim | from json } catch { null })
if $pr == null {
    exit 0
}

let state = ($pr | get --optional state | default "")

if $state == "OPEN" {
    eprintln "[godmode:receiving-review] Editing src/ with open PR — process review comments via /godmode:receiving-review first"
}

trace-end $_tid
exit 0
