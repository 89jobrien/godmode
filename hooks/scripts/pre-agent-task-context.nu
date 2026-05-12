#!/usr/bin/env nu
# pre-agent-task-context.nu — PreToolUse/Agent hook
# Injects the current running task context into the agent's prompt so subagents
# know what they're working on. Always approves. Degrades gracefully.

use ../lib/godmode-hook-lib.nu [godmode-hook-context]

let ctx = (godmode-hook-context)
if $ctx == null { exit 0 }
if ($ctx.running | length) == 0 { exit 0 }

let context_lines = ($ctx.running | each { |t|
    let id = ($t | get id? | default "?")
    let title = ($t | get title? | default "untitled")
    let crate_name = ($t | get crate_name? | default "")
    mut line = $"- ($id): ($title)"
    if not ($crate_name | is-empty) {
        $line = $"($line) [crate: ($crate_name)]"
    }
    $line
})

print $"[godmode] Active task context for this agent:\n($context_lines | str join "\n")"
exit 0
