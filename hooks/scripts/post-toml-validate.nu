#!/usr/bin/env nu
# post-toml-validate.nu — PostToolUse/Write|Edit hook
# Validates TOML files after they are written or edited.
# Exits 2 (blocking) with a diagnostic message if the file is malformed.

let input = try { open --raw /dev/stdin | from json } catch { exit 0 }

let path = $input | get -o tool_input.file_path | default ""
if not ($path | str ends-with ".toml") { exit 0 }
if not ($path | path exists) { exit 0 }

try {
    open $path | ignore
} catch {|e|
    print $"[toml-validate] ($path) — invalid TOML\n($e.msg)"
    exit 2
}
