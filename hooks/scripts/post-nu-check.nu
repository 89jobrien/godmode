#!/usr/bin/env nu
# post-nu-check.nu — PostToolUse/Write|Edit hook
# Syntax-checks Nushell scripts after they are written or edited.
# Uses `nu --ide-check` which reports errors without executing the file.
# Exits 2 (blocking) with diagnostics if syntax errors are found.

let input = try { open --raw /dev/stdin | from json } catch { exit 0 }

let path = $input | get -o tool_input.file_path | default ""
if not ($path | str ends-with ".nu") { exit 0 }
if not ($path | path exists) { exit 0 }

let result = do { nu --ide-check 100 $path } | complete
let errors = $result.stdout
    | lines
    | each { |line| try { $line | from json } catch { null } }
    | where { |r| $r != null and ($r | get -o severity) == "Error" }

if ($errors | length) > 0 {
    let count = ($errors | length)
    let diag = $errors | each { |e| $"  ($e.message) at ($e.span)" } | str join "\n"
    print $"[nu-check] ($path) — ($count) syntax errors\n($diag)"
    exit 2
}
