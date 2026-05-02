#!/usr/bin/env nu
# introspection/hook.nu — Stop hook: warn if plugin conformance check fails.
# Warn-only; always exits 0.

let _input = open --raw /dev/stdin | from json

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"

if not ($task_file | path exists) {
    exit 0
}

let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode review self --json } | complete

if $result.exit_code != 0 {
    exit 0
}

let passed = (
    try { $result.stdout | from json | get passed? | default true }
    catch { true }
)

if not $passed {
    print --stderr "[godmode:introspection] Plugin conformance issues detected — run /godmode:introspection to review"
}

exit 0
