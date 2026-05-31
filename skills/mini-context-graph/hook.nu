#!/usr/bin/env nu
# mini-context-graph/hook.nu — PostToolUse/Write hook
# After writing markdown/documentation files, reminds to ingest into kgx
# if a .kgx/ directory exists (knowledge graph is active for this project).
# Always exits 0.

let input = open --raw /dev/stdin | from json
let file_path = ($input | get --optional tool_input.file_path | default "")

# Only care about markdown files outside .ctx/
if not ($file_path | str ends-with ".md") {
    exit 0
}
if ($file_path | str contains "/.ctx/") {
    exit 0
}
# Skip skill/plugin markdown
if ($file_path | str contains "/skills/") or ($file_path | str contains "/agents/") {
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let kgx_dir = $"($git_root)/.kgx"

# Only relevant if kgx is active for this project
if not ($kgx_dir | path exists) {
    exit 0
}

let basename = ($file_path | path basename)
eprintln $"[godmode:mini-context-graph] Wrote ($basename) — consider ingesting into kgx: `kgx ingest`"

exit 0
