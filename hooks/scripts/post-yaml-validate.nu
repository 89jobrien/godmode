#!/usr/bin/env nu
# post-yaml-validate.nu — PostToolUse/Write|Edit hook
# Validates YAML files after they are written or edited.
# Exits 2 (blocking) with a diagnostic message if the file is malformed.

let input = try { open --raw /dev/stdin | from json } catch { exit 0 }

let path = $input | get -o tool_input.file_path | default ""
let is_yaml = ($path | str ends-with ".yaml") or ($path | str ends-with ".yml")
if not $is_yaml { exit 0 }
if not ($path | path exists) { exit 0 }

let result = do { open $path | ignore } | complete
if $result.exit_code != 0 {
    let err = $result.stderr | str trim
    print $"[yaml-validate] ($path) — invalid YAML\n($err)"
    exit 2
}
