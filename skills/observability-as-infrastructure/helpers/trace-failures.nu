#!/usr/bin/env nu
# Deprecated compatibility wrapper; prefer `godmode trace failures`.
use _forward.nu [forward-trace]

def main [--session: string = "", --current, --json] {
    mut args = if $json { ["--json" "trace" "failures"] } else { ["trace" "failures"] }
    if not ($session | is-empty) { $args = ($args | append ["--session" $session]) }
    if $current { $args = ($args | append "--current") }
    forward-trace $args
}
