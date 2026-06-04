#!/usr/bin/env nu
# PreToolUse/Bash hook: warn when `cd` targets a directory outside the pinned root.
#
# Reads pinned_root from .ctx/godmode/session.json. If no pin exists, passes silently.
# When a pin is active, any `cd <path>` where <path> is not under pinned_root is blocked
# with a message suggesting `git -C <path>` instead.

let input = open --raw /dev/stdin | from json

let command = ($input | get input.command? | default "")

# Only inspect commands that start with cd
if not ($command | str starts-with "cd ") {
    exit 0
}

# Extract the target path from `cd <path>`
let target = ($command | str replace "cd " "" | str trim | str trim --char '"' | str trim --char "'")

if ($target | is-empty) {
    exit 0
}

# Find the git root to locate session.json
let git_root = (do { git rev-parse --show-toplevel } | complete)
if $git_root.exit_code != 0 {
    exit 0
}
let root = ($git_root.stdout | str trim)
let session_file = $"($root)/.ctx/godmode/session.json"

if not ($session_file | path exists) {
    exit 0
}

let session = (open $session_file)
let pinned = ($session | get pinned_root? | default "")

if ($pinned | is-empty) {
    exit 0
}

# Resolve the target to an absolute path
let abs_target = if ($target | str starts-with "/") {
    $target
} else if ($target | str starts-with "~") {
    ($target | str replace "~" $env.HOME)
} else if ($target | str starts-with "$HOME") {
    ($target | str replace "$HOME" $env.HOME)
} else {
    $"(pwd)/($target)"
}

# Check if the target is under the pinned root
if not ($abs_target | str starts-with $pinned) {
    print $"[cd-guard] Session pinned to ($pinned)"
    print $"[cd-guard] Use `git -C ($abs_target)` instead of `cd ($target)`"
    print "[cd-guard] Or run `godmode unpin` to clear the pin."
    exit 2
}

exit 0
