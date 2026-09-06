#!/usr/bin/env nu
# Runs the using-doob conformance suite, but guards against a runaway
# retry loop: if the *exact same invocation* (same script + args) is
# launched more than once in a session (tracked via a state file), it
# aborts loudly instead of silently spinning.
#
# Usage: nu harness.nu [--max-repeats N]

def state-file [] {
    let tmp_root = ($env | get -o CLAUDE_JOB_TMP | default "/tmp")
    $tmp_root | path join "doob-skill-harness-invocations.jsonl"
}

def record-invocation [key: string] {
    let f = (state-file)
    if not ($f | path exists) {
        "" | save -f $f
    }
    $"($key)\n" | save -a $f
}

def count-invocations [key: string] {
    let f = (state-file)
    if not ($f | path exists) {
        return 0
    }
    open $f | lines | where {|l| $l == $key } | length
}

def main [--max-repeats: int = 1] {
    let script = (($env.FILE_PWD) | path join "run-conformance.nu")
    let key = $"run-conformance.nu"

    let prior = (count-invocations $key)
    if $prior > $max_repeats {
        print $"ABORT: identical invocation \(($key)\) already ran ($prior) time\(s\) this session -- refusing to loop."
        exit 2
    }
    record-invocation $key

    print $"Invocation #(($prior) + 1) of ($key)"
    nu $script
    let code = $env.LAST_EXIT_CODE
    exit $code
}
