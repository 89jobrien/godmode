#!/usr/bin/env nu
# post-write-plan-ingest.nu — PostToolUse/Write hook
# Detects plan files and auto-ingests them into the task graph. Degrades gracefully.

use ../lib/godmode-hook-lib.nu [godmode-hook-context]

let ctx = (godmode-hook-context)
if $ctx == null { exit 0 }

let file_path = ($ctx.input | get tool_input?.file_path? | default "")
if ($file_path | is-empty) { exit 0 }

let is_plan = (
    ($file_path | str ends-with ".plan.md") or
    (($file_path | str contains "_WORKING_DIR/") and ($file_path | str ends-with ".md"))
)
if not $is_plan { exit 0 }
if not ($file_path | path exists) { exit 0 }

let content = (open --raw $file_path)
if not ($content | str contains "### Task") { exit 0 }

let result = do { godmode plan ingest $file_path } | complete
if $result.exit_code == 0 {
    let stdout = ($result.stdout | str trim)
    if not ($stdout | is-empty) {
        print $"[godmode] Auto-ingested plan from ($file_path | path basename):\n($stdout)"
    } else {
        print $"[godmode] Auto-ingested plan from ($file_path | path basename)"
    }
} else {
    print -e $"[godmode] Plan ingest failed for ($file_path | path basename): ($result.stderr | str trim)"
}
exit 0
