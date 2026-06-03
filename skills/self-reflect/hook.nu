#!/usr/bin/env nu
# hook.nu — UserPromptSubmit hook: nudge self-reflect when user signals session end.
# Always exits 0 (non-blocking).

use ../_lib/trace.nu *
let _tid = (trace-start "self-reflect" "hook.nu")

let input = open --raw /dev/stdin | from json

let prompt = ($input | get --optional prompt | default "" | str downcase)

let triggers = ["reflect", "self-reflect", "what did we do", "session summary", "end of session", "wrap up"]

let matched = ($triggers | any { |t| $prompt | str contains $t })

if not $matched {
    exit 0
}

eprintln "[godmode:self-reflect] Session close detected — consider running /godmode:self-reflect"

trace-end $_tid
exit 0
