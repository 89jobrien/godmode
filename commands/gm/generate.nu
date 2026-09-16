#!/usr/bin/env nu

# TODO(#112): Derive workflow commands from canonical pipeline definitions.
# Generate repository-local Claude Code and OpenCode command projections.
# YAML source: commands/gm/*.yaml
# Outputs: commands/gm-<name>.md and .opencode/commands/gm-<name>.md

def render-frontmatter [command: record, target: string] {
    if $target == "claude" {
        let tools_yaml = ($command.tools | each {|tool| $"  - ($tool)" } | str join "\n")
        [
            "---"
            $"name: ($command.name)"
            "allowed-tools:"
            $tools_yaml
            $"max-turns: ($command.turns)"
            "---"
        ] | str join "\n"
    } else if $target == "opencode" {
        [
            "---"
            $"description: ($command.name)"
            "subtask: false"
            "---"
        ] | str join "\n"
    } else {
        error make {msg: $"unsupported command projection target: ($target)"}
    }
}

def render-content [command: record, target: string] {
    let frontmatter = (render-frontmatter $command $target)
    let body = if $target == "opencode" {
        $command.body | str replace --all "/gm:" "/gm-"
    } else {
        $command.body
    }
    $"($frontmatter)($body)"
}

def generate-target [commands: list<record>, target: string, out_dir: path] {
    mkdir $out_dir

    for stale in (glob ($out_dir | path join "gm-*.md")) {
        rm $stale
    }

    for command in $commands {
        let out_path = ($out_dir | path join $command.out_name)
        render-content $command $target | save --force $out_path
    }

    print $"  generated ($commands | length) ($target) commands"
}

let gm_dir = $env.FILE_PWD
let commands_dir = ($gm_dir | path dirname)
let repo_dir = ($commands_dir | path dirname)
let template_dir = ($gm_dir | path join "templates")
let yamls = (glob ($gm_dir | path join "*.yaml") | sort)

let commands = ($yamls | each {|file|
    let raw = (open $file)
    let stem = ($file | path basename | str replace ".yaml" "")
    let template = ($raw | get template | default null)
    let template_content = if $template != null {
        let template_path = ($template_dir | path join $"($template).md")
        if not ($template_path | path exists) {
            error make {msg: $"template ($template) not found for ($stem)"}
        }
        $"\n(open --raw $template_path)\n"
    } else {
        "\n"
    }

    {
        name: ($raw | get name)
        out_name: $"gm-($stem).md"
        tools: ($raw | get allowedTools | default [])
        turns: ($raw | get maxTurns | default 10)
        body: $"($template_content)\n($raw | get prompt)"
    }
})

generate-target $commands "claude" $commands_dir
generate-target $commands "opencode" ($repo_dir | path join ".opencode" "commands")
print $"($commands | length) commands projected to both targets"
