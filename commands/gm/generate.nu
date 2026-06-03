#!/usr/bin/env nu

# Generate Claude Code .md command files from YAML definitions.
# YAML source: commands/gm/*.yaml
# Output: commands/gm-<name>.md (top-level commands/ dir)

let gm_dir = ($env.FILE_PWD)
let out_dir = ($gm_dir | path dirname)

let yamls = (glob $"($gm_dir)/*.yaml")
let count = ($yamls | length)

let tmpl_dir = ($gm_dir | path join "templates")

for file in $yamls {
    let raw = (open $file)
    let name = ($raw | get name)
    let prompt = ($raw | get prompt)
    let tools = ($raw | get allowedTools | default [])
    let turns = ($raw | get maxTurns | default 10)
    let template = ($raw | get template | default null)

    let stem = ($file | path basename | str replace '.yaml' '')
    let out_name = $"gm-($stem).md"
    let out_path = ($out_dir | path join $out_name)

    # Build frontmatter
    let tools_yaml = ($tools | each {|t| $"  - ($t)" } | str join "\n")
    let frontmatter = ([
        "---"
        $"name: ($name)"
        "allowed_tools:"
        $tools_yaml
        $"max_turns: ($turns)"
        "---"
    ] | str join "\n")

    # Inject template if specified
    let tmpl_content = if $template != null {
        let tmpl_path = ($tmpl_dir | path join $"($template).md")
        if ($tmpl_path | path exists) {
            $"\n(open --raw $tmpl_path)\n"
        } else {
            print $"  WARNING: template '($template)' not found for ($stem)"
            "\n"
        }
    } else {
        "\n"
    }

    let content = $"($frontmatter)($tmpl_content)\n($prompt)"
    $content | save --force $out_path
    print $"  generated ($out_name) [($template | default 'no template')]"
}

print $"($count) commands generated"
