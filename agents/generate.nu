#!/usr/bin/env nu

# Generate Claude Code agent .md files from cfg YAML + prompt text.
# Config source: agents/cfg/<stem>.cfg.yaml
# Prompts: agents/prompts/<stem>.prompt.txt
# Templates: shared with commands — commands/gm/templates/<domain>.md
# Output: agents/<stem>.md

let agents_dir = ($env.FILE_PWD)
let cfg_dir = ($agents_dir | path join "cfg")
let prompts_dir = ($agents_dir | path join "prompts")
let tmpl_dir = ($agents_dir | path dirname | path join "commands" "gm" "templates")

let yamls = (glob $"($cfg_dir)/*.cfg.yaml")
let count = ($yamls | length)

for file in $yamls {
    let raw = (open $file)
    let stem = ($file | path basename | str replace '.cfg.yaml' '')
    let name = ($raw | get name)
    let description = ($raw | get description | str trim)
    let model = ($raw | get model | default "inherit")
    let color = ($raw | get color | default "purple")
    let tools = ($raw | get tools | default [])
    let skills = ($raw | get skills | default [])
    let template = ($raw | get template | default null)

    let out_path = ($agents_dir | path join $"($stem).md")
    let prompt_path = ($prompts_dir | path join $"($stem).prompt.txt")

    # Build frontmatter
    let tools_yaml = ($tools | each {|t| $"  - \"($t)\"" } | str join "\n")
    let skills_line = if ($skills | length) > 0 {
        ($skills | str join ", ")
    } else {
        ""
    }

    mut fm_lines = [
        "---"
        $"name: \"($name)\""
        $"description: >"
    ]

    # Wrap description at ~95 chars with 2-space indent
    let desc_lines = ($description | split row "\n" | each {|l| $"  ($l)" })
    $fm_lines = ($fm_lines | append $desc_lines)

    $fm_lines = ($fm_lines | append [
        $"model: ($model)"
        $"color: ($color)"
        "tools:"
        $tools_yaml
    ])

    if $skills_line != "" {
        $fm_lines = ($fm_lines | append [$"skills: ($skills_line)"])
    }

    $fm_lines = ($fm_lines | append ["---"])
    let frontmatter = ($fm_lines | str join "\n")

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

    # Load prompt body
    let prompt_body = if ($prompt_path | path exists) {
        (open --raw $prompt_path | str trim)
    } else {
        print $"  WARNING: no prompt file for ($stem)"
        ""
    }

    let content = $"($frontmatter)($tmpl_content)\n($prompt_body)\n"
    $content | save --force $out_path
    print $"  generated ($stem).md [($template | default 'no template')]"
}

print $"($count) agents generated"
