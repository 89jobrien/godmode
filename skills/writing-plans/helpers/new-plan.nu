#!/usr/bin/env nu
# new-plan.nu — scaffold a plan at .ctx/godmode/plans/YYYY-MM-DD-<name>.md.
# Usage: nu ($env.HOME | path join ".agents" "skills" "writing-plans" "helpers" "new-plan.nu") <feature-name>

use ../../_lib/trace.nu *
use ../../_lib/helpers.nu *

def main [feature: string] {
    if ($feature | is-empty) {
        print "Usage: new-plan.nu <feature-name>"
        exit 1
    }

    let root = (repo-root)
    let tid = (trace-start "writing-plans" "new-plan.nu" $feature)
    let date = (date now | format date "%Y-%m-%d")
    let slug = ($feature | str downcase | str replace --all --regex '[^a-z0-9]+' '-' | str trim --char '-')
    if ($slug | is-empty) {
        trace-error $tid 1 "feature name has no usable characters"
        print "ERROR: feature name has no usable characters"
        exit 1
    }
    let plan_dir = $"($root)/.ctx/godmode/plans"
    mkdir $plan_dir
    let out = $"($plan_dir)/($date)-($slug).md"

    if ($out | path exists) {
        trace-error $tid 1 $"($out) already exists"
        print $"ERROR: ($out) already exists"
        exit 1
    }

    $"# Plan: ($feature)

## Goal

One sentence. What does this implement and why.

## Architecture

- Crates affected:
- New traits/types:
- Data flow: source → transform → sink

## Tech Stack

- Rust edition:
- New dependencies:

## Tasks

### Task 1: <name>

**Crate**: `<crate-name>`
**File\(s\)**: `crates/<crate>/src/<file>.rs`
**Run**: `cargo nextest run -p <crate>`

1. Write failing test and confirm FAIL.
2. Implement minimum code to pass. Confirm GREEN.
3. `cargo clippy -p <crate> -- -D warnings` — zero warnings.
4. `git commit -m \"feat\(<crate>\): <summary>\"`
" | save $out

    trace-end $tid
    print $out
}
