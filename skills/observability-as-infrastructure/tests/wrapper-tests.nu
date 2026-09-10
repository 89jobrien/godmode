use std/assert

let root = ($env.PWD | path join $".ctx/godmode/_WORKING_DIR/trace-wrapper-test-(random uuid)")
mkdir ($root | path join "bin")
let fake = ($root | path join "bin/godmode")
"#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$FAKE_TRACE_LOG\"\nprintf 'forwarded\\n'\n" | save --force $fake
chmod +x $fake
let log = ($root | path join "calls.log")
let helpers = ($env.PWD | path join "skills/observability-as-infrastructure/helpers")
with-env {
    PATH: ($env.PATH | prepend ($root | path join "bin"))
    FAKE_TRACE_LOG: $log
} {
    let tail = (do { ^nu ($helpers | path join "trace-tail.nu") --n 7 --session sid --json } | complete)
    assert equal $tail.exit_code 0
    let summary = (do { ^nu ($helpers | path join "session-summary.nu") --sessions 2 --previous } | complete)
    assert equal $summary.exit_code 0
}
let calls = (open $log | lines)
assert equal $calls.0 "--json trace tail --n 7 --session sid"
assert equal $calls.1 "trace summary --sessions 2 --previous"
rm -rf $root
