#!/usr/bin/env nu
# testing-philosophy/hook.nu — PreToolUse/Write hook
# If a Write targets a src/ file (not lib.rs or main.rs), check whether a corresponding
# test file or inline test exists. If not, warn.
# Always exits 0 (warn only, never blocks).

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.path | default "")

# Only care about src/ files
if not ($file_path | str contains "/src/") {
    exit 0
}

# Skip lib.rs and main.rs — they are entry points, not logic modules
if ($file_path | str ends-with "src/lib.rs") or ($file_path | str ends-with "src/main.rs") {
    exit 0
}

# Must be a Rust source file
if not ($file_path | str ends-with ".rs") {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim

# Derive the base name for test lookup (e.g. src/foo/bar.rs -> bar)
let base_name = ($file_path | path basename | str replace ".rs" "")

# Check for inline tests in the target file itself
let inline_test = (
    if ($file_path | path exists) {
        try {
            open --raw $file_path | str contains "#[cfg(test)]"
        } catch { false }
    } else {
        false
    }
)

if $inline_test {
    exit 0
}

# Check for a tests/ file matching the module name
let tests_dir = $"($git_root)/tests"
let test_file_exists = (
    if ($tests_dir | path exists) {
        try {
            ls $tests_dir
            | where name =~ $base_name
            | length
            | $in > 0
        } catch { false }
    } else {
        false
    }
)

if $test_file_exists {
    exit 0
}

eprintln $"[godmode:testing-philosophy] No tests for ($file_path) — consult /godmode:testing-philosophy"

exit 0
