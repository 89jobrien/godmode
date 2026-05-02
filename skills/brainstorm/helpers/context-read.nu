#!/usr/bin/env nu
# context-read.nu — read the key files for brainstorming context in a Rust crate.
# Usage: nu skills/brainstorming/helpers/context-read.nu [crate]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [crate: string = ""] {
    let root = (repo-root)
    let tid = (trace-start "brainstorm" "context-read.nu" $crate)

    for f in [$"($root)/CLAUDE.md" $"($root)/Cargo.toml"] {
        if ($f | path exists) { open $f | print }
    }

    if not ($crate | is-empty) {
        let crate_dir = $"($root)/crates/($crate)"
        if not ($crate_dir | path exists) {
            trace-error $tid 1 $"crates/($crate) not found"
            print $"ERROR: crates/($crate) not found"
            exit 1
        }
        for f in [$"($crate_dir)/Cargo.toml" $"($crate_dir)/src/lib.rs" $"($crate_dir)/src/main.rs"] {
            if ($f | path exists) { open $f | print }
        }
    }

    trace-end $tid
}
