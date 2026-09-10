#!/usr/bin/env nu
# Deprecated compatibility wrapper; prefer `godmode trace summary`.
use _forward.nu [forward-trace]

def main [--sessions: int = 3, --previous, --json] {
    mut args = if $json { ["--json" "trace" "summary" "--sessions" ($sessions | into string)] } else { ["trace" "summary" "--sessions" ($sessions | into string)] }
    if $previous { $args = ($args | append "--previous") }
    forward-trace $args
}
