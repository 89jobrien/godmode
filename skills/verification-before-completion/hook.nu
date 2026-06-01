#!/usr/bin/env nu
# hook.nu — Stop hook: warn if godmode verify has not been run since last commit.
# Warn only — always exits 0.

# Degrade gracefully if not in a git repo
let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim
let trace_file = $"($git_root)/.ctx/GODMODE.trace.jsonl"

if not ($trace_file | path exists) {
    print --stderr "[godmode:verify] Verification gate not run since last commit — run `godmode verify` before ending session"
    exit 0
}

# Get the timestamp of the last git commit
let last_commit_result = do { git -C $git_root log -1 --format=%cI } | complete
if $last_commit_result.exit_code != 0 {
    exit 0
}

let last_commit_ts = $last_commit_result.stdout | str trim
if ($last_commit_ts | is-empty) {
    exit 0
}

# Parse last commit time
let commit_time = try { $last_commit_ts | into datetime } catch { exit 0 }

# Look for a verify_passed event after the last commit
let verified = (
    open $trace_file
    | lines
    | each { |line|
        try { $line | from json } catch { null }
    }
    | compact
    | where { |evt|
        ($evt | get event? | default "") == "verify_passed"
        and (
            try { ($evt | get timestamp? | default "" | into datetime) > $commit_time } catch { false }
        )
    }
    | length
) > 0

if not $verified {
    print --stderr "[godmode:verify] Verification gate not run since last commit — run `godmode verify` before ending session"
}

exit 0
