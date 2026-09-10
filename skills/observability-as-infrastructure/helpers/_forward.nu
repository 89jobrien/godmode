#!/usr/bin/env nu

export def forward-trace [args: list<string>] {
    let result = (do { run-external "godmode" ...$args } | complete)
    if not ($result.stdout | is-empty) { print --no-newline $result.stdout }
    if not ($result.stderr | is-empty) { print --stderr --no-newline $result.stderr }
    if $result.exit_code != 0 { exit $result.exit_code }
}
