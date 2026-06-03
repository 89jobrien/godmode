#!/usr/bin/env nu
# wave-integration/hook.nu — PostToolUse/Bash hook
# If the completed command contained `godmode wave done` or `godmode wave check`,
# prints a wave status summary. Always exits 0. Degrades gracefully.

use ../_lib/trace.nu *
let _tid = (trace-start "wave-integration" "hook.nu")

let input = open --raw /dev/stdin | from json

let cmd = try { $input.tool_input.command | default "" } catch { "" }

let relevant = ($cmd | str contains "godmode wave done") or ($cmd | str contains "godmode wave check")
if not $relevant {
    exit 0
}

# Check godmode is on PATH
let godmode_found = (which godmode | length) > 0
if not $godmode_found {
    exit 0
}

let result = do { godmode wave status --json } | complete
if $result.exit_code != 0 {
    exit 0
}

let status = do { $result.stdout | from json } | complete
if $status.exit_code != 0 {
    exit 0
}

let s = $status.output
let wave_num  = try { $s.wave    | default "?" } catch { "?" }
let done_count    = try { $s.done    | default 0 } catch { 0 }
let pending_count = try { $s.pending | default 0 } catch { 0 }
let blocked_count = try { $s.blocked | default 0 } catch { 0 }

print $"[godmode:wave] Wave ($wave_num): ($done_count) done / ($pending_count) pending / ($blocked_count) blocked"

if $pending_count == 0 and $blocked_count == 0 {
    print "  All agents settled — ready for integration"
}

trace-end $_tid
exit 0
