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

    # Check for blocked tasks — blocked tasks must be resolved or unblocked before committing.
    let list_result = (run-external "godmode" "task" "list" "--json" | complete)
    if $list_result.exit_code == 0 {
        let raw_tasks = (try { $list_result.stdout | str trim | from json } catch { [] })
        # Ensure we have a list — degrade gracefully if godmode returned a record (e.g. error shape)
        let tasks = if ($raw_tasks | describe | str starts-with "list") { $raw_tasks } else { [] }
        let blocked = ($tasks | where { |t| ($t | get --optional status | default "") == "blocked" })
        if ($blocked | length) > 0 {
            let ids = ($blocked | each { |t|
                let reason = ($t | get --optional reason | default "")
                if ($reason | is-empty) { $t.id } else { $"($t.id): ($reason)" }
            })
            print $"pre-commit: blocked tasks must be resolved before committing:"
            for id in $ids { print $"  - ($id)" }
            print "Use `godmode task unblock <id>` or `godmode task remove <id>` to clear them."
            exit 1
        }
    }
}

# ---------------------------------------------------------------------------
# Step 2: plugin.json version bump validation
# ---------------------------------------------------------------------------
#
# Enforces that vX.Y.Z increments by exactly 1 in at most one component.
# Prevents accidental jumps like 0.5.0 → 1.1.0 or 0.5.0 → 0.7.0.

let plugin_json_path = $"($env.PWD)/.claude-plugin/plugin.json"

if ($plugin_json_path | path exists) {
    # Version in the working tree (staged)
    let staged_raw = (
        try { open $plugin_json_path | get version? | default "" }
        catch { "" }
    )

    # Version in HEAD (last commit)
    let head_result = (run-external "git" "show" "HEAD:.claude-plugin/plugin.json" | complete)
    let head_raw = if $head_result.exit_code == 0 {
        try { $head_result.stdout | from json | get version? | default "" }
        catch { "" }
    } else {
        # No previous commit — skip check (initial commit)
        ""
    }

    if not ($staged_raw | is-empty) and not ($head_raw | is-empty) and ($staged_raw != $head_raw) {
        def parse-version [v: string] {
            let parts = ($v | split row "." | each { |p| $p | into int })
            if ($parts | length) != 3 {
                error make { msg: $"pre-commit: invalid version format: ($v)" }
            }
            $parts
        }

        let prev = (parse-version $head_raw)
        let next = (parse-version $staged_raw)

        let major_diff = ($next.0 - $prev.0)
        let minor_diff = ($next.1 - $prev.1)
        let patch_diff = ($next.2 - $prev.2)

        # Each component must not decrease, and no component may jump by more than 1
        let any_decrease = ($major_diff < 0) or ($minor_diff < 0) or ($patch_diff < 0)
        let any_jump     = ($major_diff > 1) or ($minor_diff > 1) or ($patch_diff > 1)

        # When a higher component increments, lower components must reset to 0
        let major_bumped = $major_diff == 1
        let minor_bumped = $minor_diff == 1
        let bad_reset = (
            ($major_bumped and ($next.1 != 0 or $next.2 != 0))
            or ($minor_bumped and not $major_bumped and $next.2 != 0)
        )

        if $any_decrease or $any_jump or $bad_reset {
            print $"pre-commit: invalid version bump ($head_raw) → ($staged_raw)"
            print "  Rules: increment exactly one component by 1; reset lower components to 0."
            print "  Examples: 0.5.0 → 0.5.1  |  0.5.0 → 0.6.0  |  0.5.0 → 1.0.0"
            exit 1
        }

        print $"pre-commit: version bump ($head_raw) → ($staged_raw) ok"
    }
}

# ---------------------------------------------------------------------------
# Step 3: cargo gates — fmt-check, clippy, nextest
# ---------------------------------------------------------------------------

print "pre-commit: running cargo fmt --check ..."
run-gate "cargo" "fmt" "--all" "--check"

print "pre-commit: running cargo clippy ..."
run-gate "cargo" "clippy" "--workspace" "--" "-D" "warnings"

print "pre-commit: running cargo nextest ..."
run-gate "cargo" "nextest" "run" "--workspace"


print "pre-commit: all checks passed."
