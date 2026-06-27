#!/usr/bin/env nu
# Check that every skill directory has an entry in skill-index.json.
# Exits 0 if the index is complete; exits 1 and lists missing entries if not.
# Run from the godmode repo root.

let index_path = ($env.PWD | path join "skills" "using-godmode" "references" "skill-index.json")
let skills_dir = ($env.PWD | path join "skills")

let indexed = (open $index_path | get skills | get name)
let on_disk = (
    ls $skills_dir
    | where type == dir
    | get name
    | each { path basename }
    | where { |n| ($n == "_lib") | not $in }
)

let missing = ($on_disk | where { |n| ($indexed | any { |i| $i == $n }) | not $in })

if ($missing | is-empty) {
    print "skill-index.json is up to date"
    exit 0
} else {
    print $"ERROR: ($missing | length) skill(s) missing from skill-index.json:"
    $missing | each { |n| print $"  - ($n)" }
    exit 1
}
