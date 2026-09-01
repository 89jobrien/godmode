#!/usr/bin/env nu

# Generate a markdown index table of all installed agents from ~/.claude/agents/*.md.
# Parses YAML frontmatter (name, description, skills, color) and deduplicates by name,
# preferring canonical (non-prefixed) filenames over domain-prefixed variants.
#
# The Rust CLI is authoritative for repository indexes:
#   godmode agent index [--check]
#   godmode skill index [--check]
# This legacy script remains for global ~/.claude/agents installations.
#
# Usage:
#   nu /Users/joe/dev/godmode/scripts/gen-index.nu
#   nu /Users/joe/dev/godmode/scripts/gen-index.nu | save ~/.claude/agents/INDEX.md

let agents_dir = ($env.HOME | path join ".claude" "agents")
let index_file = ($agents_dir | path join "INDEX.md")

let agents = (
    glob ($agents_dir | path join "*.md")
    | where {|f| $f != $index_file }
    | each { |f|
        let content = open --raw $f
        let lines = $content | lines
        let in_front = $lines | skip 1 | take while { |l| $l != '---' }
        let fname = $f | path basename | str replace '.md' ''

        let name_line   = ($in_front | where { |l| $l | str starts-with 'name:' }        | first)
        let desc_line   = ($in_front | where { |l| $l | str starts-with 'description:' } | first)
        let skills_line = ($in_front | where { |l| $l | str starts-with 'skills:' }      | first)
        let color_line  = ($in_front | where { |l| $l | str starts-with 'color:' }       | first)

        let name   = if ($name_line   == null) { $fname } else { $name_line   | str replace 'name:'        '' | str trim | str trim -c '"' }
        let desc   = if ($desc_line   == null) { '' }     else { $desc_line   | str replace 'description:' '' | str trim | str trim -c '"' }
        let skills = if ($skills_line == null) { '' }     else { $skills_line | str replace 'skills:'      '' | str trim }
        let color  = if ($color_line  == null) { '' }     else { $color_line  | str replace 'color:'       '' | str trim }
        let is_canonical = not ($fname | str contains '__')

        {file: $fname, name: $name, description: $desc, skills: $skills, color: $color, is_canonical: $is_canonical}
    }
    | sort-by is_canonical --reverse
    | uniq-by name
    | sort-by name
)

print "# Agent Index"
print ""
print "| Name | File | Description | Skills | Color |"
print "| ---- | ---- | ----------- | ------ | ----- |"

for a in $agents {
    let desc = $a.description | str replace -a "|" "/" | str replace -a "\n" " "
    print $"| ($a.name) | ($a.file).md | ($desc) | ($a.skills) | ($a.color) |"
}
