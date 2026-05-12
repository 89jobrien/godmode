#!/usr/bin/env nu
# stop-guard.nu — Stop hook: block session end if tasks are still running.

use ../lib/godmode-hook-lib.nu [godmode-hook-context]

let ctx = (godmode-hook-context)

# Emit session.end trace event unconditionally
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code == 0 {
    let git_root = $git_result.stdout | str trim
    let trace_script = ($env.CLAUDE_PLUGIN_ROOT | path join "hooks/scripts/godmode-trace.rs")
    if ($trace_script | path exists) {
        do { rust-script $trace_script end $git_root } | complete | ignore
    }
}

if $ctx == null { exit 0 }

# Check .ctx/.initialized exists (session was started via godmode)
let init_file = $"($ctx.git_root)/.ctx/.initialized"
if not ($init_file | path exists) { exit 0 }

if ($ctx.running | length) > 0 {
    let ids = ($ctx.running | each { |t| $t | get id? | default "?" } | str join ", ")
    print $"[godmode] Session blocked: tasks still running: ($ids)"
    print "Mark them done or blocked before ending the session:"
    print "  godmode task done <id> --commit <sha>"
    print "  godmode task block <id> <reason>"
    exit 1
}

if ($ctx.blocked | length) > 0 {
    let ids = ($ctx.blocked | each { |t|
        let id = ($t | get id? | default "?")
        let reason = ($t | get reason? | default "")
        if ($reason | is-empty) { $id } else { $"($id): ($reason)" }
    })
    print "[godmode] Session blocked: blocked tasks must be resolved:"
    for id in $ids { print $"  - ($id)" }
    print "Use `godmode task unblock <id>` or `godmode task remove <id>` to clear them."
    exit 1
}

exit 0
