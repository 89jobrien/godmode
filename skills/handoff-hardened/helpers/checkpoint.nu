#!/usr/bin/env nu
# Checkpoint store for the hardened handoff state machine.
# Default: $HOME/dev/.ctx/godmode/handoff-checkpoint.json
# Shape: { run_date: str, repos: { <repo>: { status, attempts, claims, error, updated } } }

export def checkpoint-path [] {
    $env | get -o GODMODE_HANDOFF_CHECKPOINT | default ($env.HOME | path join "dev/.ctx/godmode/handoff-checkpoint.json")
}

export def load-checkpoint-from [path: path] {
    let checkpoint = if ($path | path exists) {
        try { open $path } catch { { run_date: null, repos: {} } }
    } else {
        { run_date: null, repos: {} }
    }
    if (($checkpoint | describe) !~ "record") {
        return (reset-checkpoint-for-today { run_date: null, repos: {} })
    }
    reset-checkpoint-for-today $checkpoint
}

export def load-checkpoint [] {
    load-checkpoint-from (checkpoint-path)
}

export def reset-checkpoint-for-today [checkpoint: record] {
    let today = (date now | format date "%Y-%m-%d")
    let checkpoint_date = ($checkpoint | get -o run_date | default "")
    let repos = ($checkpoint | get -o repos | default {})
    if $checkpoint_date == $today and (($repos | describe) =~ "record") {
        $checkpoint | upsert repos $repos
    } else {
        { run_date: $today, repos: {} }
    }
}

export def save-checkpoint [data: record] {
    let path = (checkpoint-path)
    let dir = ($path | path dirname)
    if not ($dir | path exists) { mkdir $dir }
    $data | save --force $path
}

export def update-repo [checkpoint: record, repo: string, fields: record] {
    let repos = ($checkpoint | get -o repos | default {})
    let existing = ($repos | get -o $repo | default {
        status: "PENDING"
        attempts: 0
        claims: []
        error: null
    })
    let merged = ($existing | merge $fields | merge { updated: (date now | format date "%Y-%m-%dT%H:%M:%S%z") })
    $checkpoint | upsert repos ($repos | upsert $repo $merged)
}

export def repo-status [checkpoint: record, repo: string] {
    $checkpoint | get -o repos | default {} | get -o $repo | get -o status | default "PENDING"
}

export def pending-repos [checkpoint: record, all_repos: list] {
    let current = (reset-checkpoint-for-today $checkpoint)
    if ($current.repos | is-empty) { return $all_repos }
    $all_repos | where { |repo|
        (repo-status $current $repo.repo) not-in ["COMPLETE" "PARTIAL" "VERIFIED" "FLAGGED" "COMMITTED"]
    }
}
