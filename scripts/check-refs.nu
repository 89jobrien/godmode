#!/usr/bin/env nu

# Audit all SKILL.md files in the godmode skills/ directory for broken file references.
# Finds backtick, bold, and markdown link references to helpers/, references/,
# scripts/, assets/ paths and reports any that don't exist on disk.
#
# Replaces: godmode-check-refs.nu (one-off in /tmp)
# Used by: godmode:introspection skill
#
# Usage:
#   nu /Users/joe/dev/godmode/scripts/check-refs.nu
#   nu /Users/joe/dev/godmode/scripts/check-refs.nu --root /path/to/godmode
#   nu /Users/joe/dev/godmode/scripts/check-refs.nu --json
#   nu /Users/joe/dev/godmode/scripts/check-refs.nu --skill brainstorm
#   nu /Users/joe/dev/godmode/scripts/check-refs.nu --fix
#   nu /Users/joe/dev/godmode/scripts/check-refs.nu --ci   # exits 1 if issues found

def main [
    --root: string = ""    # Godmode root dir (default: $GODMODE_ROOT or ~/dev/godmode)
    --skill: string = ""   # Check only this skill by name (e.g. "brainstorm")
    --fix                  # Create empty stub files for broken references
    --json                 # Output JSON instead of human-readable text
    --ci                   # Exit non-zero if any broken references are found
] {
    let godmode_root = if $root != "" {
        $root
    } else if not ($env | get --optional GODMODE_ROOT | is-empty) {
        $env.GODMODE_ROOT
    } else {
        $"/($env.HOME)/dev/godmode"
    }

    let skills_dir = ($godmode_root | path join "skills")

    let skill_dirs = (
        ls $skills_dir
        | where type == dir
        | get name
        | if $skill != "" {
            where {|d| ($d | path basename) == $skill }
          } else { $in }
    )

    if ($skill != "") and ($skill_dirs | length) == 0 {
        print $"No skill directory found for: ($skill)"
        exit 1
    }

    let skill_files = (
        $skill_dirs
        | each {|dir| $dir | path join "SKILL.md" }
        | where {|p| $p | path exists }
    )

    mut issues = []

    for file in $skill_files {
        let dir      = ($file | path dirname)
        let rel_file = ($file | str replace $godmode_root "")
        let text     = (open --raw $file)

        # backtick refs: `helpers/foo.nu`
        let ticked = ($text | parse -r '`(?<path>(references|helpers|scripts|assets)/[^`]+)`')
        # bold refs: **helpers/foo.nu**
        let bold   = ($text | parse -r '\*\*(?<path>(references|helpers|scripts|assets)/[^*]+)\*\*')
        # markdown link refs: [label](helpers/foo.nu)
        let linked = ($text | parse -r '\[([^\]]+)\]\((?<path>(references|helpers|scripts|assets)/[^)]+)\)')

        for hit in ($ticked | append $bold | append $linked) {
            let clean = (
                $hit.path
                | str trim
                | str replace -r '\s.*$' ''    # strip arg placeholders e.g. `helpers/foo.nu <arg>`
                | str replace -r '[.,;:)]+$' ''
            )
            let target = ($dir | path join $clean)

            if not ($target | path exists) {
                $issues = ($issues | append {skill: $rel_file, missing: $clean, full_path: $target})
            }
        }
    }

    let result = ($issues | sort-by skill missing | uniq)

    if $fix and ($result | length) > 0 {
        for issue in $result {
            let parent = ($issue.full_path | path dirname)
            if not ($parent | path exists) {
                mkdir $parent
            }
            "# TODO: fill in this helper/reference\n" | save --force $issue.full_path
            print $"  created stub: ($issue.full_path)"
        }
        let n = ($result | length)
        print $"Created ($n) stub file(s) — fill in content before committing."
        return
    }

    if $json {
        $result | select skill missing | to json
        return
    }

    if ($result | length) == 0 {
        print "All skill references OK."
    } else {
        let count = ($result | length)
        print $"($count) broken references:"
        $result | each {|r| print $"  ($r.skill): ($r.missing)"}
        if $ci {
            exit 1
        }
    }
}
