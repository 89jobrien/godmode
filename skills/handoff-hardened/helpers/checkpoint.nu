#!/usr/bin/env nu
# Checkpoint store for the hardened handoff state machine.
# File: ~/dev/.ctx/godmode/handoff-checkpoint.json
# Shape: { run_date: str, repos: { <repo>: { status, attempts, claims, error, updated } } }
#
# Statuses: PENDING RUNNING RETRY PARTIAL COMPLETE VERIFIED FLAGGED COMMITTED

const CHECKPOINT_PATH = "/Users/joe/dev/.ctx/godmode/handoff-checkpoint.json"

export def checkpoint-path [] {
    $CHECKPOINT_PATH
}

export def load-checkpoint [] {
    let checkpoint = if ($CHECKPOINT_PATH | path exists) {
        open $CHECKPOINT_PATH
    } else {
        { run_date: null, repos: {} }
    }
    reset-checkpoint-for-today $checkpoint
}

export def reset-checkpoint-for-today [checkpoint: record] {
    let today = (date now | format date "%Y-%m-%d")
    let checkpoint_date = ($checkpoint | get -o run_date | default "")
    if $checkpoint_date == $today {
        $checkpoint
    } else {
        { run_date: $today, repos: {} }
    }
}

export def save-checkpoint [data: record] {
    let dir = ($CHECKPOINT_PATH | path dirname)
    if not ($dir | path exists) {
        mkdir $dir
    }
    $data | save -f $CHECKPOINT_PATH
}

# Update a single repo's checkpoint entry, merging fields.
export def update-repo [checkpoint: record, repo: string, fields: record] {
    let existing = ($checkpoint.repos | get -o $repo | default {
        status: "PENDING"
        attempts: 0
        claims: []
        error: null
    })
    let merged = ($existing | merge $fields | merge { updated: (date now | format date "%Y-%m-%dT%H:%M:%S%z") })
    let new_repos = ($checkpoint.repos | upsert $repo $merged)
    $checkpoint | upsert repos $new_repos
}

export def repo-status [checkpoint: record, repo: string] {
    $checkpoint.repos | get -o $repo | get -o status | default "PENDING"
}

# Repos that still need work this run (not already terminal from the current run).
export def pending-repos [checkpoint: record, all_repos: list] {
    let current = (reset-checkpoint-for-today $checkpoint)
    if ($current.repos | is-empty) {
        return $all_repos
    }

    $all_repos | where { |r|
        let s = (repo-status $current $r.repo)
        $s not-in ["COMPLETE" "PARTIAL" "VERIFIED" "FLAGGED" "COMMITTED"]
    }
}
