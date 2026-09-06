#!/usr/bin/env nu
# audit.nu — inventory skills, check reference integrity, and flag missing index entries.
# Usage: nu skills/introspection/helpers/audit.nu

def main [] {
    let root = (git rev-parse --show-toplevel | str trim)
    let skills_dir = $"($root)/skills"

    # Derive inventory from every SKILL.md so nested and *-workspace skills are included.
    let skill_dirs = (glob $"($skills_dir)/**/SKILL.md" | each { |p| $p | path dirname })

    # Check skill-index coverage via using-godmode/SKILL.md
    let index_path = $"($skills_dir)/using-godmode/SKILL.md"
    let index_content = if ($index_path | path exists) {
        open $index_path
    } else {
        ""
    }

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
            $issues = ($issues | append $"[($skill_name)] not referenced in using-godmode/SKILL.md")
        }

        for ref_line in ($content | lines | where { |l| $l | str contains "helpers/" }) {
            let parsed = ($ref_line | parse --regex '`helpers/(?P<f>[^\s`]+)`')
            if ($parsed | length) > 0 {
                let raw = ($parsed | first | get f)
                # Expand shell brace-alternation refs like "name.{sh,fish,nu}"
                # into their real candidate filenames before checking existence.
                let brace = ($raw | parse --regex '^(?P<base>[^{]+)\{(?P<alts>[^}]+)\}(?P<rest>.*)$')
                let candidates = if ($brace | length) > 0 {
                    let b = ($brace | first)
                    ($b.alts | split row "," | each { |alt| $"($b.base)($alt)($b.rest)" })
                } else {
                    [$raw]
                }
                for fname in $candidates {
                    if not ($"($skill_dir)/helpers/($fname)" | path exists) {
                        $issues = ($issues | append $"[($skill_name)] broken helper ref: helpers/($fname)")
                    }
                }
            }
        }

        for ref_line in ($content | lines | where { |l| $l | str contains "references/" }) {
            let parsed = ($ref_line | parse --regex '`references/(?P<f>[^\s`]+)`')
            if ($parsed | length) > 0 {
                let fname = ($parsed | first | get f)
                # Skip template placeholders like <topic>.md or *.md
                if ($fname | str contains "<") or ($fname | str contains "*") or ($ref_line | str contains "example path") {
                    continue
                }
                if not ($"($skill_dir)/references/($fname)" | path exists) {
                    $issues = ($issues | append $"[($skill_name)] broken reference: references/($fname)")
                }
            }
        }
    }

    let date = (date now | format date "%Y-%m-%d")
    let timestamp = (date now | format date "%Y-%m-%d %H:%M")
    let ctx_dir = $"($root)/.ctx/godmode/reports/introspection"
    let report_path = $"($ctx_dir)/introspection-($date).md"

    if not ($ctx_dir | path exists) {
        mkdir $ctx_dir
    }

    let report = if ($issues | is-empty) {
        $"# Introspect Report — ($timestamp)\n\n## No issues found\n- ($skill_dirs | length) skills checked. All references valid, index complete.\n"
    } else {
        let lines = ($issues | each { |i| $"- ($i)" } | str join "\n")
        $"# Introspect Report — ($timestamp)\n\n## Blocking\n($lines)\n"
    }

    $report | save --force $report_path

    if ($issues | is-empty) {
        print $"($skill_dirs | length) skills checked. No issues. Report: ($report_path)"
    } else {
        for issue in $issues { print $issue }
        print $"\nReport written: ($report_path)"
        exit 1
    }
}
