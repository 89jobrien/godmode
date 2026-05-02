#!/usr/bin/env nu
# run-review.nu — run the code-review gate and print a structured report scaffold.
# Usage: nu skills/code-review/helpers/run-review.nu [--crate <name>]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--crate: string = ""] {
    if not ($crate | is-empty) {
        if ($crate | str contains " ") or ($crate | str contains ";") or ($crate | str contains "&") {
            print "ERROR: invalid crate name"; exit 1
        }
    }

    let tid = (trace-start "code-review" "run-review.nu" $crate)
    cargo-gate $tid --crate $crate
    run-external "git" "diff" "--stat"
    run-external "git" "diff" "--cached" "--stat"
    trace-end $tid

    print "
## Code Review — <file or feature>

### Blocking
- (none)

### Suggestions
- (none)

### Nitpicks
- (none)
"
}
