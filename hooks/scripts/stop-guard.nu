#!/usr/bin/env nu
# stop-guard.nu — Stop hook: delegates to `godmode hook run stop-guard`.

# Stop hooks run in a restricted PATH; prepend user binary dirs
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.cargo/bin" | prepend $"($env.HOME)/.local/bin")

# Emit session.end trace event unconditionally
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code == 0 {
    let git_root = $git_result.stdout | str trim
    let trace_script = ($env | get -o CLAUDE_PLUGIN_ROOT | default "" | path join "hooks/scripts/godmode-trace.rs")
    if ($trace_script | path exists) and ((which rust-script | length) > 0) {
        do { rust-script $trace_script end $git_root } | complete | ignore
    }
}

# No further delegation needed — godmode hook has no 'run' subcommand
exit 0
