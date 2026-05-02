#!/usr/bin/env nu
# _lib/helpers.nu — shared utility functions for godmode helper scripts.
#
# Source alongside trace.nu:
#   use ($"(repo-root)/skills/_lib/helpers.nu") *

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------

# Return the git repository root, or error.
export def repo-root [] {
    let r = (run-external "git" "rev-parse" "--show-toplevel" | complete)
    if $r.exit_code != 0 { error make { msg: "not inside a git repo" } }
    $r.stdout | str trim
}

# ---------------------------------------------------------------------------
# Command execution
# ---------------------------------------------------------------------------

# Run an external command and return its `complete` record.
# Exits with the command's exit code and prints stderr on failure.
# Pass `trace_id` to emit a trace-error event on failure.
export def run-checked [
    trace_id: string   # pass "" to skip tracing
    ...args: string
] {
    let r = (run-external ...$args | complete)
    if $r.exit_code != 0 {
        if not ($trace_id | is-empty) {
            use ($"(repo-root)/skills/_lib/trace.nu") *
            trace-error $trace_id $r.exit_code $r.stderr
        }
        print $r.stderr
        exit $r.exit_code
    }
    $r
}

# ---------------------------------------------------------------------------
# Cargo gates
# ---------------------------------------------------------------------------

# Run a standard workspace cargo gate (nextest + clippy + fmt-check).
# Exits on first failure. Pass trace_id or "" to skip tracing.
export def cargo-gate [trace_id: string, --crate: string = ""] {
    let pkg = if ($crate | is-empty) { ["--workspace"] } else { ["-p" $crate] }

    for gate in [
        (["cargo" "nextest" "run"] | append $pkg)
        (["cargo" "clippy"] | append $pkg | append ["--" "-D" "warnings"])
        (["cargo" "fmt"] | append $pkg | append ["--check"])
    ] {
        run-checked $trace_id ...$gate
    }
}

# Run only cargo check + nextest (lighter gate — no fmt/clippy).
export def cargo-test-gate [trace_id: string, --crate: string = ""] {
    let pkg = if ($crate | is-empty) { ["--workspace"] } else { ["-p" $crate] }
    run-checked $trace_id "cargo" "check" ...$pkg
    run-checked $trace_id "cargo" "nextest" "run" ...$pkg
}

# ---------------------------------------------------------------------------
# Git helpers
# ---------------------------------------------------------------------------

# Return the current branch name.
export def git-branch [] {
    (run-external "git" "branch" "--show-current" | complete).stdout | str trim
}

# Assert the current branch is NOT main. Exits with error if it is.
export def assert-not-main [] {
    let b = (git-branch)
    if $b == "main" {
        print "ERROR: on main — switch to a feature branch first"
        exit 1
    }
    $b
}

# ---------------------------------------------------------------------------
# Trace helpers
# ---------------------------------------------------------------------------

# Load .ctx/GODMODE.trace.jsonl as a list<record>. Exits 0 with message if absent.
export def open-trace [] {
    let trace = $"(repo-root)/.ctx/GODMODE.trace.jsonl"
    if not ($trace | path exists) { print "No trace file."; exit 0 }
    open $trace | lines | each { from json }
}

# ---------------------------------------------------------------------------
# Worktree helpers
# ---------------------------------------------------------------------------

# Return true if BLOCKED.md exists at the given path.
export def is-blocked [wt_path: string] {
    ($"($wt_path)/BLOCKED.md" | path exists)
}

# Return true if a worktree path has at least one commit.
export def worktree-has-commits [wt_path: string] {
    let log = (run-external "git" "-C" $wt_path "log" "--oneline" "-1" | complete).stdout | str trim
    not ($log | is-empty)
}
