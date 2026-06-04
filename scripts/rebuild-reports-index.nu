#!/usr/bin/env nu

# Rebuild .ctx/godmode/reports/godmode-reports.index.json from disk.
# Scans all known category subdirectories and writes a fresh index.

let root = (git rev-parse --show-toplevel | str trim)
let reports = $"($root)/.ctx/godmode/reports"

if not ($reports | path exists) {
    print $"(ansi red)reports dir not found: ($reports)(ansi reset)"
    exit 1
}

let categories = [
    {
        name: "reflect"
        path: "reflect/"
        description: "Session self-reflection reports"
    }
    {
        name: "introspection"
        path: "introspection/"
        description: "Skill/agent/plugin consistency audits"
    }
    {
        name: "insights"
        path: "insights/"
        description: "Session insight captures and individual insight items"
    }
]

mut cats = {}

for cat in $categories {
    let dir = $"($reports)/($cat.name)"
    if not ($dir | path exists) {
        continue
    }

    let files = (ls $dir
        | where type == file and name ends-with ".md"
        | get name
        | each { |f| $f | path basename }
        | sort)

    mut items = []
    let items_dir = $"($dir)/items"
    if ($items_dir | path exists) {
        $items = (ls $items_dir
            | where type == file
            | get name
            | each { |f| $"items/($f | path basename)" }
            | sort)
    }

    mut entry = {
        path: $cat.path
        description: $cat.description
        files: $files
    }
    if ($items | length) > 0 {
        $entry = ($entry | merge { items: $items })
    }

    $cats = ($cats | merge { ($cat.name): $entry })
}

let index = {
    version: 1
    generated: (date now | format date "%Y-%m-%d")
    categories: $cats
}

let out = $"($reports)/godmode-reports.index.json"
$index | to json --indent 2 | save --force $out
print $"rebuilt ($out) — ($cats | columns | length) categories"
