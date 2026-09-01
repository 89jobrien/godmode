#!/usr/bin/env nu
# Enumerate repos under ~/dev with uncommitted or unpushed work.
# Output: table of {repo, path, dirty, unpushed, ahead, behind}

def main [base: string = "/Users/joe/dev"] {
    ls $base
    | where type == dir
    | each { |d|
        let p = $d.name
        let git_dir = ($p | path join ".git")
        if not ($git_dir | path exists) {
            return null
        }

        let status = (do { git -C $p status --porcelain } | complete)
        if $status.exit_code != 0 {
            return null
        }
        let dirty = ($status.stdout | str trim | str length) > 0

        let upstream = (do { git -C $p rev-parse --abbrev-ref --symbolic-full-name '@{u}' } | complete)
        let has_upstream = $upstream.exit_code == 0

        let ahead_behind = if $has_upstream {
            let r = (do { git -C $p rev-list --left-right --count 'HEAD...@{u}' } | complete)
            if $r.exit_code == 0 {
                let parts = ($r.stdout | str trim | split row "\t")
                { ahead: ($parts.0 | into int), behind: ($parts.1 | into int) }
            } else {
                { ahead: 0, behind: 0 }
            }
        } else {
            { ahead: 0, behind: 0 }
        }

        let unpushed = $ahead_behind.ahead > 0

        if $dirty or $unpushed {
            {
                repo: ($p | path basename)
                path: $p
                dirty: $dirty
                unpushed: $unpushed
                ahead: $ahead_behind.ahead
                behind: $ahead_behind.behind
            }
        } else {
            null
        }
    }
    | compact
}
