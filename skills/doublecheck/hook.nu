#!/usr/bin/env nu
# doublecheck/hook.nu — PostToolUse/Bash hook
# After web searches (WebSearch/WebFetch), reminds to verify claims.
# Detects curl/wget/web-fetch patterns in bash commands.
# Always exits 0.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")
let exit_code = ($input | get --optional tool_response.exit_code | default 0)

# Only trigger on successful web-fetching commands
if $exit_code != 0 {
    exit 0
}

let is_web_fetch = (
    ($cmd | str contains "curl ") or
    ($cmd | str contains "wget ") or
    ($cmd | str contains "web_search") or
    ($cmd | str contains "http")
)

if not $is_web_fetch {
    exit 0
}

eprintln "[godmode:doublecheck] Web content fetched — verify factual claims before committing to a plan"

exit 0
