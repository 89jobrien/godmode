#!/usr/bin/env nu
# pre-commit.nu — godmode pre-commit hook
#
# Blocks commits when any task is in `running` state, then runs cargo gates.
# Install via: nu hooks/install.nu

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def git-root [] {
    let r = (run-external "git" "rev-parse" "--show-toplevel" | complete)
    if $r.exit_code != 0 { error make { msg: "not inside a git repo" } }
    $r.stdout | str trim
}

def run-gate [...args: string] {
    let r = (run-external ...$args | complete)
    if $r.exit_code != 0 {
        print $r.stdout
        print $r.stderr
        exit $r.exit_code
    }
}

# ---------------------------------------------------------------------------
# Step 1: godmode handoff check (degrade gracefully if binary absent)
# ---------------------------------------------------------------------------

let godmode_path = (which godmode | get path? | first | default "")

if ($godmode_path | is-empty) {
    print "pre-commit: godmode not on PATH — skipping task-state check"
} else {
    let result = (run-external "godmode" "handoff" "--json" | complete)

    if $result.exit_code != 0 {
        # handoff exits non-zero when tasks are still running
        let raw = $result.stdout | str trim
        let msg = if ($raw | is-empty) { $result.stderr | str trim } else { $raw }

        # Try to parse JSON for a nicer error; fall back to raw output
        let running_ids = (
            try { $raw | from json | get running_task_ids? | default [] }
            catch { [] }
        )

        if ($running_ids | length) > 0 {
            print $"pre-commit: tasks still running: ($running_ids | str join ', ')"
        } else {
            print $"pre-commit: godmode handoff failed:\n($msg)"
        }

        print "Resolve running tasks before committing (`godmode task done <id>` or `godmode task block <id> <reason>`)."
        exit 1
    }
}

# ---------------------------------------------------------------------------
# Step 2: cargo gates — fmt-check, clippy, nextest
# ---------------------------------------------------------------------------

print "pre-commit: running cargo fmt --check ..."
run-gate "cargo" "fmt" "--all" "--check"

print "pre-commit: running cargo clippy ..."
run-gate "cargo" "clippy" "--workspace" "--" "-D" "warnings"

print "pre-commit: running cargo nextest ..."
run-gate "cargo" "nextest" "run" "--workspace"

print "pre-commit: all checks passed."
