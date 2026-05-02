#!/usr/bin/env nu
# audit.nu — inventory skills, check reference integrity, and flag missing index entries.
# Usage: nu skills/self-review/helpers/audit.nu

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [] {
    let root = (repo-root)
    let skills_dir = $"($root)/skills"
    let tid = (trace-start "introspection" "audit.nu")

    let skill_dirs = (ls $skills_dir | where type == "dir" | get name
        | where { |d| ($d | path basename) != "_lib" })
    let index_content = (open $"($skills_dir)/using-godmode/references/skill-index.md")

    mut issues = []

    for skill_dir in $skill_dirs {
        let skill_name = ($skill_dir | path basename)
        let skill_md = $"($skill_dir)/SKILL.md"

        if not ($skill_md | path exists) {
            $issues = ($issues | append $"[($skill_name)] missing SKILL.md")
            continue
        }

        let content = (open $skill_md)

        if not ($index_content | str contains $skill_name) {
            $issues = ($issues | append $"[($skill_name)] not in skill-index.md")
        }

        for ref_line in ($content | lines | where { |l| $l | str contains "helpers/" }) {
            let fname = ($ref_line | parse --regex '`helpers/(?P<f>[^`]+)`' | get f? | first?)
            if not ($fname | is-empty) and not ($"($skill_dir)/helpers/($fname)" | path exists) {
                $issues = ($issues | append $"[($skill_name)] broken helper ref: helpers/($fname)")
            }
        }

        for ref_line in ($content | lines | where { |l| $l | str contains "references/" }) {
            let fname = ($ref_line | parse --regex '`references/(?P<f>[^`]+)`' | get f? | first?)
            if not ($fname | is-empty) and not ($"($skill_dir)/references/($fname)" | path exists) {
                $issues = ($issues | append $"[($skill_name)] broken reference: references/($fname)")
            }
        }
    }

    if ($issues | is-empty) {
        trace-end $tid
        print $"($skill_dirs | length) skills checked. No issues."
    } else {
        trace-error $tid 1 ($issues | str join "\n")
        for issue in $issues { print $issue }
        exit 1
    }
}
