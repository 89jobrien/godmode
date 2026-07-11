#!/usr/bin/env nu
# find-paths.nu — extract all literal file paths from markdown docs and check existence
#
# Usage:
#   nu skills/doc-sync/helpers/find-paths.nu [--dir <docs-dir>]
#
# Output: table of path, exists (bool), source file

def main [--dir: string = "."] {
    let md_files = (glob $"($dir)/**/*.md")

    let results = $md_files | each { |file|
        let content = open --raw $file
        # Match paths: start with /, ./, ~/, or word chars followed by /
        let paths = ($content | lines | each { |line|
            $line | parse --regex '`([~/.]?[a-zA-Z0-9_.\-/]+/[a-zA-Z0-9_.\-/]*)`'
            | get capture0? | default []
        } | flatten)

        $paths | each { |p|
            let expanded = ($p | str replace '~' $env.HOME)
            {
                path: $p,
                exists: ($expanded | path exists),
                source: $file,
            }
        }
    } | flatten

    $results | sort-by exists | each { |r|
        let status = if $r.exists { "✅" } else { "❌" }
        print $"($status) ($r.path) — ($r.source)"
    }

    let broken = ($results | where exists == false)
    if ($broken | length) > 0 {
        print $"\n($broken | length) broken path(s) found."
        exit 1
    } else {
        print "\nAll paths verified."
    }
}
