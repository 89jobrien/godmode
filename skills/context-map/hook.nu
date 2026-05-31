#!/usr/bin/env nu
# context-map/hook.nu — PreToolUse/Edit hook
# Warns if editing a src/ file without a context map written this session.
# A context map is evidenced by .ctx/_WORKING_DIR/context-map-*.md existing.
# Always exits 0.

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.file_path | default "")

# Only care about src/ edits
if not ($file_path | str contains "/src/") {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let working_dir = $"($git_root)/.ctx/_WORKING_DIR"

if not ($working_dir | path exists) {
    eprintln "[godmode:context-map] Editing src/ without a context map — run /godmode:context-map first"
    exit 0
}

# Check for any context-map file written today
let today = (date now | format date "%Y-%m-%d")
let maps = try {
    ls $working_dir
    | where name =~ "context-map"
    | where modified > (date now | $in - 4hr)
    | length
} catch { 0 }

if $maps == 0 {
    eprintln "[godmode:context-map] Editing src/ without a recent context map — run /godmode:context-map first"
}

exit 0
