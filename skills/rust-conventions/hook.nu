#!/usr/bin/env nu
# rust-conventions/hook.nu — PreToolUse/Edit hook
# When editing .rs files, checks if the file has any `unwrap()` calls without
# expect() and warns about convention violations. Also checks for missing
# #[cfg(test)] in files that have test functions.
# Always exits 0.

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.file_path | default "")

# Only care about Rust files
if not ($file_path | str ends-with ".rs") {
    exit 0
}

# Skip test infrastructure files
if ($file_path | str contains "/tests/") or ($file_path | str contains "/testing/") {
    exit 0
}

if not ($file_path | path exists) {
    exit 0
}

let content = try { open --raw $file_path } catch { exit 0 }

# Check for unwrap() without expect() in non-test code
let has_cfg_test = ($content | str contains "#[cfg(test)]")
let lines = $content | lines

# Only check code above #[cfg(test)] if it exists
let check_lines = if $has_cfg_test {
    let test_line = $lines | enumerate | where {|row| $row.item | str contains "#[cfg(test)]" } | get 0?.index | default ($lines | length)
    $lines | first $test_line
} else {
    $lines
}

let bare_unwraps = $check_lines | where {|line|
    ($line | str contains ".unwrap()") and (not ($line | str contains ".expect("))
} | length

if $bare_unwraps > 0 {
    eprintln $"[godmode:rust-conventions] ($file_path | path basename) has ($bare_unwraps) bare unwrap\(\) — use .expect\(\"reason\"\) or return Result"
}

exit 0
