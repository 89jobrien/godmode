#!/usr/bin/env nu
# hook.nu — PostToolUse/Bash hook: after a successful git push, notify if a CI run started.
# Always exits 0 (warn only).

let input = open --raw /dev/stdin | from json

let command = $input | get tool_input?.command? | default ""

# Only act on git push commands
if not ($command | str contains "git push") {
    exit 0
}

let exit_code = $input | get tool_response?.exit_code? | default 1

if $exit_code != 0 {
    exit 0
}

# Check if gh is available
let gh_found = (which gh | length) > 0
if not $gh_found {
    exit 0
}

# Wait briefly for GitHub to register the run
sleep 5sec

let run_result = do { gh run list --limit 1 --json status,conclusion } | complete
if $run_result.exit_code != 0 {
    exit 0
}

let runs = try { $run_result.stdout | from json } catch { [] }
if ($runs | length) == 0 {
    exit 0
}

let run = $runs | first
let status = $run | get status? | default ""

if $status == "queued" or $status == "in_progress" {
    print --stderr "[godmode:ci-fix] CI run started — check status with `gh run list --limit 3`"
}

exit 0
