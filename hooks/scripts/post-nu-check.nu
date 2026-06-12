#!/usr/bin/env nu
# post-nu-check.nu — PostToolUse/Write|Edit hook
# Syntax-checks Nushell scripts after they are written or edited.
# Uses `nu --ide-check` which reports errors without executing the file.
# Exits 2 (blocking) with diagnostics if syntax errors are found.

let input = try { open --raw /dev/stdin | from json } catch { exit 0 }

let path = $input | get -o tool_input.file_path | default ""
if not ($path | str ends-with ".nu") { exit 0 }
if not ($path | path exists) { exit 0 }

let result = do { nu --ide-check $path } | complete
if $result.exit_code != 0 {
    let out = ($result.stdout | str trim)
    let err = ($result.stderr | str trim)
    let diag = if ($out | is-empty) { $err } else { $out }
    print $"[nu-check] ($path) — syntax error\n($diag)"
    exit 2
}
