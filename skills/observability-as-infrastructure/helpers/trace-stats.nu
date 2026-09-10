#!/usr/bin/env nu
# Deprecated compatibility wrapper; prefer `godmode trace stats`.
use _forward.nu [forward-trace]

def main [--session: string = "", --current, --json] {
    mut args = if $json { ["--json" "trace" "stats"] } else { ["trace" "stats"] }
    if not ($session | is-empty) { $args = ($args | append ["--session" $session]) }
    if $current { $args = ($args | append "--current") }
    forward-trace $args
}
