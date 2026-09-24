#!/usr/bin/env nu
# Inventory bundled skills/agents/commands and validate reference/index integrity.

def declared-name [path: path] {
    let line = (open --raw $path | lines | where {|line| $line | str trim | str starts-with "name:"} | first)
    $line | split row ":" | skip 1 | str join ":" | str trim | str replace --all '"' ""
}

def main [] {
    let root = (git rev-parse --show-toplevel | str trim)
    let skills_dir = ($root | path join "skills")
    let using_godmode = ($skills_dir | path join "using-godmode" "SKILL.md")
    let machine_index_path = ($skills_dir | path join "using-godmode" "references" "skill-index.json")
    let using_content = (open --raw $using_godmode)
    let machine_names = (open $machine_index_path | get skills.name)
    let skill_files = (glob ($skills_dir | path join "**" "SKILL.md") | sort)
    mut issues = []

    for skill_md in $skill_files {
        let skill_dir = ($skill_md | path dirname)
        let dirname = ($skill_dir | path basename)
        if ($dirname | str ends-with "-workspace") { continue }
        let declared = (declared-name $skill_md)
        let short_name = ($declared | str replace "godmode:" "")
        if $short_name != $dirname {
            $issues = ($issues | append $"[($dirname)] frontmatter name mismatch: ($declared)")
        }
        if not ($using_content | str contains $"`godmode:($short_name)`") {
            $issues = ($issues | append $"[($short_name)] missing from using-godmode skill table")
        }
        if $short_name not-in $machine_names {
            $issues = ($issues | append $"[($short_name)] missing from skill-index.json")
        }

        let content = (open --raw $skill_md)
        for ref_line in ($content | lines | where {|line| $line =~ 'helpers/'}) {
            for parsed in ($ref_line | parse --regex '`helpers/(?P<f>[^\s`]+)`') {
                let raw = $parsed.f
                let brace = ($raw | parse --regex '^(?P<base>[^{]+)\{(?P<alts>[^}]+)\}(?P<rest>.*)$')
                let candidates = if ($brace | is-not-empty) {
                    let value = ($brace | first)
                    $value.alts | split row "," | each {|alt| $"($value.base)($alt)($value.rest)"}
                } else { [$raw] }
                for candidate in $candidates {
                    if not ($skill_dir | path join "helpers" $candidate | path exists) {
                        $issues = ($issues | append $"[($short_name)] broken helper: helpers/($candidate)")
                    }
                }
            }
        }
        for ref_line in ($content | lines | where {|line| $line =~ 'references/'}) {
            for parsed in ($ref_line | parse --regex '`references/(?P<f>[^\s`]+)`') {
                let target = $parsed.f
                if ($target | str contains "<") or ($target | str contains "*") { continue }
                if not ($skill_dir | path join "references" $target | path exists) {
                    $issues = ($issues | append $"[($short_name)] broken reference: references/($target)")
                }
            }
        }
    }

    let agent_rows = (glob ($root | path join "agents" "*.md")
        | where {|path| ($path | path basename) != "INDEX.md"}
        | each {|path| {name: (declared-name $path), path: $path}})
    for duplicate in ($agent_rows | group-by name | transpose name entries
        | where {|row| ($row.entries | length) > 1}) {
        let files = ($duplicate.entries | get path | each {|path| $path | path basename} | str join ", ")
        $issues = ($issues | append $"[agents] duplicate name ($duplicate.name): ($files)")
    }
    let agent_index = (open --raw ($root | path join "agents" "INDEX.md"))
    for row in $agent_rows {
        if not ($agent_index | str contains $"| ($row.name)") {
            $issues = ($issues | append $"[agents] ($row.name) missing from agents/INDEX.md")
        }
    }

    let source_commands = (glob ($root | path join "commands" "gm" "*.yaml")
        | each {|path| $path | path parse | get stem} | sort)
    let claude_commands = (glob ($root | path join "commands" "gm-*.md")
        | each {|path| $path | path parse | get stem | str replace "gm-" ""} | sort)
    let opencode_commands = (glob ($root | path join ".opencode" "commands" "gm-*.md")
        | each {|path| $path | path parse | get stem | str replace "gm-" ""} | sort)
    if $source_commands != $claude_commands {
        $issues = ($issues | append "[commands] Claude projections differ from YAML sources")
    }
    if $source_commands != $opencode_commands {
        $issues = ($issues | append "[commands] OpenCode projections differ from YAML sources")
    }

    let date = (date now | format date "%Y-%m-%d")
    let timestamp = (date now | format date "%Y-%m-%d %H:%M")
    let reports_root = ($root | path join ".ctx" "godmode" "reports")
    let report_dir = ($reports_root | path join "introspection")
    mkdir $report_dir
    let report_name = $"introspection-($date).md"
    let report_path = ($report_dir | path join $report_name)
    let report = if ($issues | is-empty) {
        $"# Introspect Report — ($timestamp)\n\n## No issues found\n\n- ($skill_files | length) bundled skills checked.\n- Agent names are unique and indexed.\n- Command projections match YAML sources.\n- Helper and reference paths resolve.\n"
    } else {
        let lines = ($issues | each {|issue| $"- ($issue)"} | str join "\n")
        $"# Introspect Report — ($timestamp)\n\n## Blocking\n\n($lines)\n"
    }
    $report | save --force $report_path

    let report_index_path = ($reports_root | path join "godmode-reports.index.json")
    let report_index = if ($report_index_path | path exists) {
        open $report_index_path
    } else {
        {categories: {introspection: {files: []}}}
    }
    let existing = ($report_index.categories.introspection.files? | default [])
    let files = if $report_name in $existing { $existing } else { $existing | append $report_name }
    $report_index | upsert categories.introspection.files $files | to json --indent 2
        | save --force $report_index_path

    if ($issues | is-empty) {
        print $"($skill_files | length) skills checked. No issues. Report: ($report_path)"
    } else {
        for issue in $issues { print $issue }
        print $"\nReport written: ($report_path)"
        exit 1
    }
}
