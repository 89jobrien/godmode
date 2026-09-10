#!/usr/bin/env nu
# Deprecated compatibility wrapper; prefer `godmode trace tail`.
use _forward.nu [forward-trace]

def main [--n: int = 20, --session: string = "", --current, --json] {
    mut args = if $json { ["--json" "trace" "tail" "--n" ($n | into string)] } else { ["trace" "tail" "--n" ($n | into string)] }
    if not ($session | is-empty) { $args = ($args | append ["--session" $session]) }
    if $current { $args = ($args | append "--current") }
    forward-trace $args
}
