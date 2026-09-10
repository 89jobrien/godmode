#!/usr/bin/env nu
# Enumerate repos under $HOME/dev with uncommitted, unpushed, or unreadable state.

export def classify-repo [repo: string, path: path, status: record, upstream: record, ahead_behind: any] {
    if $status.exit_code != 0 {
        let detail = ($status.stderr | default "" | str trim)
        let message = if ($detail | is-empty) { "unknown error" } else { $detail }
        return {
            repo: $repo path: $path dirty: false unpushed: false ahead: 0 behind: 0
            error: $"git status failed: ($message)"
        }
    }
    let dirty = not ($status.stdout | str trim | is-empty)
    mut ahead = 0
    mut behind = 0
    mut error = ""
    if $upstream.exit_code == 0 {
        if $ahead_behind == null or $ahead_behind.exit_code != 0 {
            let detail = if $ahead_behind == null { "not run" } else { $ahead_behind.stderr | default "" | str trim }
            $error = $"git rev-list failed: ($detail)"
        } else {
            let parts = ($ahead_behind.stdout | str trim | split row "\t")
            if ($parts | length) == 2 {
                let counts = (try { { ahead: ($parts.0 | into int), behind: ($parts.1 | into int) } } catch { null })
                if $counts == null {
                    $error = "git rev-list returned invalid counts"
                } else {
                    $ahead = $counts.ahead
                    $behind = $counts.behind
                }
            } else {
                $error = "git rev-list returned invalid counts"
            }
        }
    }
    let result = {
        repo: $repo path: $path dirty: $dirty unpushed: ($ahead > 0)
        ahead: $ahead behind: $behind error: (if ($error | is-empty) { null } else { $error })
    }
    if $dirty or $ahead > 0 or not ($error | is-empty) { $result } else { null }
}

def discover-repos [base: path] {
    ls $base
    | where type == dir
    | each { |dir|
        let path = $dir.name
        if not (($path | path join ".git") | path exists) { return null }
        let status = (do { git -C $path status --porcelain } | complete)
        let upstream = (do { git -C $path rev-parse --abbrev-ref --symbolic-full-name "@{u}" } | complete)
        let ahead_behind = if $upstream.exit_code == 0 {
            do { git -C $path rev-list --left-right --count "HEAD...@{u}" } | complete
        } else { null }
        classify-repo ($path | path basename) $path $status $upstream $ahead_behind
    }
    | compact
}

def main [base: string = ""] {
    let root = if ($base | is-empty) { $env.HOME | path join "dev" } else { $base }
    discover-repos $root
}
