#!/usr/bin/env nu
# stop-guard.nu — Stop hook: block session end if tasks are still running.
# Registered as a `command` type Stop hook in hooks/hooks.json.
# Exits 0 (approve) when safe, 1 (block) when running tasks exist.

let _input = open --raw /dev/stdin | from json

# Degrade gracefully if not in a git repo
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim

# Emit session.end trace event (unconditional — fires for all git repos)
let trace_script = ($env.CLAUDE_PLUGIN_ROOT | path join "hooks/scripts/godmode-trace.rs")
if ($trace_script | path exists) {
    do { rust-script $trace_script end $git_root } | complete | ignore
}

let init_file = $"($git_root)/.ctx/.initialized"

if not ($init_file | path exists) {
    exit 0
}

let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode handoff --json } | complete

if $result.exit_code == 0 {
    exit 0
}

# Non-zero exit from handoff means running tasks exist — parse and report
let raw = $result.stdout | str trim
let running_ids = (
    try { $raw | from json | get running_task_ids? | default [] }
    catch { [] }
)

if ($running_ids | length) > 0 {
    print $"[godmode] Session blocked: tasks still running: ($running_ids | str join ', ')"
    print "Mark them done or blocked before ending the session:"
    print "  godmode task done <id> --commit <sha>"
    print "  godmode task block <id> <reason>"
} else {
    print $"[godmode] Session blocked: ($raw)"
}

exit 1
