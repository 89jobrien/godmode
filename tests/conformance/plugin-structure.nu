#!/usr/bin/env nu
# tests/conformance/plugin-structure.nu
# Run from repo root: nu tests/conformance/plugin-structure.nu
# Exits 0 on all checks passed, 1 on any failure.

# Extract the frontmatter name value from SKILL.md content.
# Returns empty string if not found.
def extract_fm_name [content: string] {
    # Find the name: line within the opening frontmatter block (before second ---)
    let lines = ($content | lines)
    mut in_fm = false
    mut past_first = false
    mut result = ""
    for line in $lines {
        if $line == "---" {
            if not $past_first {
                $in_fm = true
                $past_first = true
            } else {
                # closing ---
                break
            }
        } else if $in_fm {
            if ($line | str starts-with "name:") {
                $result = ($line | str replace --regex '^name:\s*"?' "" | str replace --regex '"?\s*$' "" | str trim)
                break
            }
        }
    }
    $result
}

def main [] {
    let repo_root = (git rev-parse --show-toplevel | str trim)
    let skills_dir = ($repo_root | path join "skills")
    let plugin_json = ($repo_root | path join ".claude-plugin" "plugin.json")
    let skill_index = ($repo_root | path join "skills" "using-godmode" "references" "skill-index.md")
    let using_godmode_skill = ($repo_root | path join "skills" "using-godmode" "SKILL.md")

    mut failures: list<string> = []
    mut checks = 0

    # Collect skill dirs (exclude _lib which has no SKILL.md by design)
    let skill_dirs = (ls $skills_dir | where type == "dir" | get name | where { |p| ($p | path basename) != "_lib" })

    # -----------------------------------------------------------------------
    # Check 1: Every skill dir has a SKILL.md
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        $checks = $checks + 1
        if not ($skill_md | path exists) {
            $failures = ($failures | append $"[($skill_name)] missing SKILL.md")
        }
    }

    # -----------------------------------------------------------------------
    # Check 2: Frontmatter `name` field is present and non-empty
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        $checks = $checks + 1
        let content = (open --raw $skill_md)
        let fm_name = (extract_fm_name $content)
        if ($fm_name | is-empty) {
            $failures = ($failures | append $"[($skill_name)] missing or empty frontmatter name:")
        }
    }

    # -----------------------------------------------------------------------
    # Check 3: skill name matches skill-index.md AND using-godmode/SKILL.md
    # using-godmode is excluded: it hosts the index and is not listed in it
    # -----------------------------------------------------------------------
    let index_content = (open --raw $skill_index)
    let using_content = (open --raw $using_godmode_skill)

    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        # using-godmode hosts the index; it is not an entry within it
        if $skill_name == "using-godmode" { continue }
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        $checks = $checks + 1
        let content = (open --raw $skill_md)
        let skill_full_name = (extract_fm_name $content)
        if ($skill_full_name | is-empty) { continue }

        if not ($index_content | str contains $skill_full_name) {
            $failures = ($failures | append $"[($skill_name)] name '($skill_full_name)' not found in skill-index.md")
        }
        if not ($using_content | str contains $skill_full_name) {
            $failures = ($failures | append $"[($skill_name)] name '($skill_full_name)' not found in using-godmode/SKILL.md Available Skills table")
        }
    }

    # -----------------------------------------------------------------------
    # Check 4: No orphan index entries — every entry in skill-index.md has a dir
    # -----------------------------------------------------------------------
    let index_names = (
        $index_content
        | lines
        | where { |l| $l =~ '`godmode:[^`]+`' }
        | each { |l|
            $l | parse --regex '`(godmode:[^`]+)`' | get capture0? | default []
        }
        | flatten
        | uniq
    )
    let dir_names = ($skill_dirs | each { |p| $p | path basename })

    for entry in $index_names {
        $checks = $checks + 1
        let short = ($entry | str replace "godmode:" "")
        if not ($dir_names | any { |d| $d == $short }) {
            $failures = ($failures | append $"[index] orphan entry '($entry)' — no matching skills/($short)/ dir")
        }
    }

    # -----------------------------------------------------------------------
    # Check 5: references/ links resolve
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let content = (open --raw $skill_md)
        let refs = (
            $content
            | lines
            | each { |l| $l | parse --regex '`(references/[^`]+)`' | get capture0? | default [] }
            | flatten
        )
        for ref in $refs {
            $checks = $checks + 1
            let resolved = ($skill_path | path join $ref)
            if not ($resolved | path exists) {
                $failures = ($failures | append $"[($skill_name)] broken references link: ($ref)")
            }
        }
    }

    # -----------------------------------------------------------------------
    # Check 6: helpers/ links resolve
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let content = (open --raw $skill_md)
        let refs = (
            $content
            | lines
            | each { |l| $l | parse --regex '`(helpers/[^`]+)`' | get capture0? | default [] }
            | flatten
        )
        for ref in $refs {
            $checks = $checks + 1
            let resolved = ($skill_path | path join $ref)
            if not ($resolved | path exists) {
                $failures = ($failures | append $"[($skill_name)] broken helpers link: ($ref)")
            }
        }
    }

    # -----------------------------------------------------------------------
    # Check 7: plugin.json allowed fields only
    # -----------------------------------------------------------------------
    $checks = $checks + 1
    if ($plugin_json | path exists) {
        let pj = (open $plugin_json)
        let allowed = ["name" "version" "author" "description"]
        let keys = ($pj | columns)
        for k in $keys {
            if not ($allowed | any { |a| $a == $k }) {
                $failures = ($failures | append $"[plugin.json] disallowed field: ($k)")
            }
        }
        # Check author sub-keys (only "name" allowed)
        let author_val = ($pj | get author? | default null)
        if $author_val != null {
            let author_keys = ($author_val | columns)
            let allowed_author = ["name"]
            for ak in $author_keys {
                if not ($allowed_author | any { |a| $a == $ak }) {
                    $failures = ($failures | append $"[plugin.json] disallowed author field: ($ak)")
                }
            }
        }
    } else {
        $failures = ($failures | append "[plugin.json] file not found at .claude-plugin/plugin.json")
    }

    # -----------------------------------------------------------------------
    # Check 8 (issue #14): CLI subcommand conformance
    # -----------------------------------------------------------------------
    let canonical_subcommands = [
        "handon"
        "handoff"
        "status"
        "task list"
        "task next"
        "task add"
        "task start"
        "task done"
        "task block"
        "task unblock"
        "task unblock-all"
        "task run"
        "task remove"
        "task clear"
        "task pull"
        "task push-done"
        "plan ingest"
        "dispatch"
        "agent"
    ]

    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let lines = (open --raw $skill_md | lines | enumerate)
        for row in $lines {
            let line = $row.item
            let lineno = ($row.index + 1)
            if not ($line =~ '^\s*godmode\s+\S') { continue }
            let trimmed = ($line | str trim)
            let parts = ($trimmed | split row ' ' | skip 1 | where { |p| ($p | str trim) != "" })
            if ($parts | length) == 0 { continue }
            let first = ($parts | first)
            # Skip flag-like, placeholder, variable tokens
            if ($first | str starts-with '-') or ($first | str starts-with '<') or ($first | str starts-with '$') or ($first | str starts-with '#') { continue }
            # Try two-token subcommand
            let two = if ($parts | length) >= 2 {
                let second = ($parts | get 1)
                if not ($second | str starts-with '-') and not ($second | str starts-with '<') and not ($second | str starts-with '$') and not ($second | str starts-with '[') {
                    $"($first) ($second)"
                } else { "" }
            } else { "" }

            $checks = $checks + 1
            let matched_two = if ($two | str length) > 0 {
                $canonical_subcommands | any { |s| $s == $two }
            } else { false }
            let matched_one = $canonical_subcommands | any { |s| $s == $first }
            if not $matched_two and not $matched_one {
                let shown = if ($two | str length) > 0 { $two } else { $first }
                $failures = ($failures | append $"[($skill_name):($lineno)] unknown subcommand: godmode ($shown)")
            }
        }
    }

    # -----------------------------------------------------------------------
    # Check 9 (issue #16): Merge strategy — git merge must use --no-ff
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let content = (open --raw $skill_md)
        $checks = $checks + 1
        let mentions_merge = ($content =~ 'git merge')
        let mentions_no_ff = ($content =~ '--no-ff')
        if $mentions_merge and not $mentions_no_ff {
            $failures = ($failures | append $"[($skill_name)] consistency violation: merge strategy — mentions 'git merge' but not '--no-ff'")
        }
    }

    # -----------------------------------------------------------------------
    # Check 10 (issue #16): Concurrency cap must be 5
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let content = (open --raw $skill_md)
        let limit_lines = ($content | lines | enumerate | where { |row| $row.item =~ '\d+\s+concurrent' })
        for row in $limit_lines {
            let nums = ($row.item | parse --regex '(\d+)\s+concurrent' | get capture0? | default [])
            for n in $nums {
                $checks = $checks + 1
                if $n != "5" {
                    $failures = ($failures | append $"[($skill_name)] consistency violation: concurrency cap — found ($n), expected 5")
                }
            }
        }
    }

    # -----------------------------------------------------------------------
    # Check 11 (issue #16): BLOCKED.md trigger must be 3 attempts
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let content = (open --raw $skill_md)
        let blocked_lines = (
            $content | lines | enumerate
            | where { |row| ($row.item =~ 'BLOCKED') and ($row.item =~ '\d+\s+(attempt|tries|retry|retries|failed)') }
        )
        for row in $blocked_lines {
            let nums = ($row.item | parse --regex '(\d+)\s+(attempt|tries|retry|retries|failed)' | get capture0? | default [])
            for n in $nums {
                $checks = $checks + 1
                if $n != "3" {
                    $failures = ($failures | append $"[($skill_name)] consistency violation: BLOCKED.md threshold — found ($n), expected 3")
                }
            }
        }
    }

    # -----------------------------------------------------------------------
    # Check 12 (issue #16): Branch guard — git commit requires git branch --show-current
    # -----------------------------------------------------------------------
    for skill_path in $skill_dirs {
        let skill_name = ($skill_path | path basename)
        let skill_md = ($skill_path | path join "SKILL.md")
        if not ($skill_md | path exists) { continue }
        let content = (open --raw $skill_md)
        $checks = $checks + 1
        let has_commit = ($content =~ 'git commit')
        let has_branch_guard = ($content =~ 'git branch --show-current')
        if $has_commit and not $has_branch_guard {
            $failures = ($failures | append $"[($skill_name)] consistency violation: branch guard — has 'git commit' but no 'git branch --show-current' check")
        }
    }

    # -----------------------------------------------------------------------
    # Results
    # -----------------------------------------------------------------------
    if ($failures | is-empty) {
        print $"($checks) checks passed."
        exit 0
    } else {
        for f in $failures {
            print $f
        }
        let n = ($failures | length)
        print $"\n($n) checks failed out of ($checks) total."
        exit 1
    }
}
