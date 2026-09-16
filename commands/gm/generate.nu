#!/usr/bin/env nu

# TODO(#112): Derive workflow commands from canonical pipeline definitions.
# Generate repository-local Claude Code and OpenCode command projections.
# YAML source: commands/gm/*.yaml
# Outputs: commands/gm-<name>.md and .opencode/commands/gm-<name>.md

def render-frontmatter [command: record, target: string] {
    let fields = if $target == "claude" {
        {
            name: $command.name
            allowed-tools: $command.tools
            max-turns: $command.turns
        }
    } else if $target == "opencode" {
        {
            description: $command.description
            subtask: false
        }
    } else {
        error make {msg: $"unsupported command projection target: ($target)"}
    }

    ["---" ($fields | to yaml | str trim) "---"] | str join "\n"
}

def render-content [command: record, target: string] {
    let frontmatter = (render-frontmatter $command $target)
    let body = if $target == "opencode" {
        $command.body | str replace --all "/gm:" "/gm-"
    } else {
        $command.body
    }
    [$frontmatter $body] | str join ""
}

def stage-target [commands: list<record>, target: string, stage_dir: path] {
    mkdir $stage_dir
    for command in $commands {
        let out_path = ($stage_dir | path join $command.out_name)
        render-content $command $target | save $out_path
    }
}

def install-target [stage_dir: path, out_dir: path] {
    for stale in (glob ($out_dir | path join "gm-*.md") | sort) {
        rm $stale
    }
    for generated in (glob ($stage_dir | path join "gm-*.md") | sort) {
        mv $generated $out_dir
    }
}

let gm_dir = $env.FILE_PWD
let commands_dir = ($gm_dir | path dirname)
let repo_dir = ($commands_dir | path dirname)
let opencode_dir = ($repo_dir | path join ".opencode" "commands")
let template_dir = ($gm_dir | path join "templates")
let stage_root = ($repo_dir | path join ".command-projection-stage")
let yamls = (glob ($gm_dir | path join "*.yaml") | sort)

let commands = ($yamls | each {|file|
    let raw = (open $file)
    let stem = ($file | path basename | str replace ".yaml" "")
    let prompt = ($raw | get prompt)
    let description = ($prompt | lines | where {|line| ($line | str trim) != "" } | first | str trim)
    let template = ($raw | get template | default null)
    let body = if $template != null {
        let template_path = ($template_dir | path join $"($template).md")
        if not ($template_path | path exists) {
            error make {msg: $"template ($template) not found for ($stem)"}
        }
        ["\n" (open --raw $template_path) "\n\n" $prompt] | str join ""
    } else {
        ["\n\n" $prompt] | str join ""
    }

    {
        name: ($raw | get name)
        description: $description
        out_name: $"gm-($stem).md"
        tools: ($raw | get allowedTools | default [])
        turns: ($raw | get maxTurns | default 10)
        body: $body
    }
})

if ($stage_root | path exists) {
    rm --recursive --force $stage_root
}
let claude_stage = ($stage_root | path join "claude")
let opencode_stage = ($stage_root | path join "opencode")
stage-target $commands "claude" $claude_stage
stage-target $commands "opencode" $opencode_stage

# Validate both destinations before replacing either projection.
mkdir $commands_dir
mkdir $opencode_dir
install-target $claude_stage $commands_dir
install-target $opencode_stage $opencode_dir
rm --recursive --force $stage_root

print $"($commands | length) commands projected to both targets"
