#!/usr/bin/env nu
# hook.nu — PostToolUse/Bash hook for systematic-debugging skill
# Fires after every Bash tool call. If the command failed (non-zero exit) and
# does not involve godmode or git, nudges the user to run /godmode:systematic-debugging.

let input = open --raw /dev/stdin | from json

let exit_code = ($input | get --optional tool_response.exit_code | default null)
let cmd = ($input | get --optional tool_input.command | default "")

# Only act on genuine failures — null means the tool didn't report an exit code
if $exit_code == null or $exit_code == 0 {
    exit 0
}

# Skip godmode and git commands — they have their own failure handling
if ($cmd | str contains "godmode") or ($cmd | str contains "git") {
    exit 0
}

eprintln $"[godmode:debug] Command failed \(exit ($exit_code)\) — run /godmode:systematic-debugging before guessing a fix"

exit 0
