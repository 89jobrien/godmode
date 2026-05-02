#!/usr/bin/env nu
# tackle-issues/hook.nu — PostToolUse/Bash hook
# If the completed command contained `gh issue close`, prints a reminder to verify the merge.
# Always exits 0. Degrades gracefully.

let input = open --raw /dev/stdin | from json

# Extract the command string from tool input
let cmd = try { $input.tool_input.command | default "" } catch { "" }

if not ($cmd | str contains "gh issue close") {
    exit 0
}

# Try to extract the issue number — first numeric token after "close"
let parts = $cmd | split row " "
let close_idx = $parts | enumerate | where {|it| $it.item == "close"} | get 0?.index | default -1

let issue_num = if $close_idx >= 0 and ($close_idx + 1) < ($parts | length) {
    $parts | get ($close_idx + 1)
} else {
    ""
}

if ($issue_num | is-empty) {
    print "[godmode:tackle-issues] Issue closed — verify merge in git log: `git log --oneline main | head -5`"
} else {
    print $"[godmode:tackle-issues] Issue #($issue_num) closed — verify merge in git log: `git log --oneline main | head -5`"
}

exit 0
