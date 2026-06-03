#!/usr/bin/env nu
# testing-philosophy/hook.nu — thin shim; logic lives in godmode-core/src/test_check.rs
# PreToolUse/Write hook: warns if a src/ file has no associated tests.
# Always exits 0 (warn only, never blocks).

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.path | default "")

if ($file_path | is-empty) {
    exit 0
}

# godmode test-check exits 2 if uncovered, 0 if covered or not applicable
do { run-external "godmode" "test-check" $file_path } | complete | ignore
exit 0
