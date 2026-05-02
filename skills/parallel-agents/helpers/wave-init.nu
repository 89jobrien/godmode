#!/usr/bin/env nu
# wave-init.nu — initialise .ctx/wave-status.json for a parallel dispatch wave.
# Usage: nu skills/parallel-agents/helpers/wave-init.nu <crate-or-domain>...

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [...slots: string] {
    if ($slots | is-empty) {
        print "Usage: wave-init.nu <slot>..."
        exit 1
    }

    let tid = (trace-start "parallel-agents" "wave-init.nu" ...$slots)
    let root = (repo-root)
    let ctx_dir = $"($root)/.ctx"
    mkdir $ctx_dir

    let agents = ($slots | reduce --fold {} { |slot, acc|
        $acc | insert $slot { status: "pending", branch: "", commits: [] }
    })

    { wave: 1, agents: $agents } | to json | save --force $"($ctx_dir)/wave-status.json"

    for slot in $slots {
        trace-agent-start $"wave-($slot)" $slot $slot
    }

    trace-end $tid
    print $"($ctx_dir)/wave-status.json written with ($slots | length) slot(s)."
}
