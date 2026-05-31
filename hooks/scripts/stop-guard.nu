#!/usr/bin/env nu
# stop-guard.nu — Stop hook: delegates to `godmode hook run stop-guard`.

let input = open --raw /dev/stdin | from json

# Emit session.end trace event unconditionally
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code == 0 {
    let git_root = $git_result.stdout | str trim
    let trace_script = ($env | get -i CLAUDE_PLUGIN_ROOT | default "" | path join "hooks/scripts/godmode-trace.rs")
    if ($trace_script | path exists) {
        do { rust-script $trace_script end $git_root } | complete | ignore
    }
}

# Delegate to Rust implementation
let result = do { godmode hook run stop-guard } | complete
if $result.exit_code != 0 {
    print -e $result.stderr
    exit $result.exit_code
}
exit 0
