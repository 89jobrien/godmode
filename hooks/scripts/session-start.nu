#!/usr/bin/env nu
# session-start.nu — PostToolUse/SessionStart hook
# Runs `godmode handon` when .ctx/GODMODE.tasks.yaml exists in the repo root.
# No-ops silently in non-godmode repos.

let _input = open --raw /dev/stdin | from json

# Find git root; bail silently if not in a git repo
let git_root_result = do { git rev-parse --show-toplevel } | complete
if $git_root_result.exit_code != 0 {
    exit 0
}

let git_root = $git_root_result.stdout | str trim

# Emit session.start trace event (unconditional — fires for all git repos)
let trace_script = ($env.CLAUDE_PLUGIN_ROOT | path join "hooks/scripts/godmode-trace.rs")
if ($trace_script | path exists) {
    do { rust-script $trace_script start $git_root } | complete | ignore
}

let init_file = $"($git_root)/.ctx/.initialized"

if not ($init_file | path exists) {
    exit 0
}

# Check godmode is on PATH
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

do { godmode handon } | complete | ignore

# Version check — warn if plugin version doesn't match installed binary
let bin_version = (do { godmode --version } | complete | get stdout | str trim | split row " " | last | default "")
let plugin_version = (
    try { open ($git_root + "/.claude-plugin/plugin.json") | get version } catch { "" }
)
if not ($bin_version | is-empty) and not ($plugin_version | is-empty) and $bin_version != $plugin_version {
    print $"[godmode] Version mismatch: binary=($bin_version) plugin=($plugin_version) — run `cargo install --path crates/godmode-cli --root ~/.local` to update"
}
