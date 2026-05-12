#!/usr/bin/env nu
# pre-bash-nag.nu — PreToolUse/Bash hook
# If no tasks are running but pending tasks exist, warn before any Bash command.
# Always approves — never blocks.

use ../lib/godmode-hook-lib.nu [godmode-hook-context]

let ctx = (godmode-hook-context)
if $ctx == null { exit 0 }

let cmd = ($ctx.input | get tool_input?.command? | default "")
# Skip godmode commands themselves to avoid recursion noise
if ($cmd | str starts-with "godmode") { exit 0 }

if ($ctx.running | length) == 0 and ($ctx.pending | length) > 0 {
    let next_result = do { godmode task next --json } | complete
    let next_ids = if $next_result.exit_code == 0 {
        try {
            $next_result.stdout | str trim | from json
            | each { |t| $t | get id? | default "" }
            | str join ", "
        } catch { "" }
    } else { "" }
    eprintln $"[godmode] No task running. ($ctx.pending | length) pending. Start one: godmode task start ($next_ids)"
}

exit 0
