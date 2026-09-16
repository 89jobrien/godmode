#!/usr/bin/env nu

# Generate repository-local Claude Code and OpenCode command projections.
# YAML source: commands/gm/*.yaml
# Outputs: commands/gm-<name>.md and .opencode/commands/gm-<name>.md

def compose-prompt [raw: record, repo_dir: path] {
    let pipeline_name = ($raw.pipeline? | default null)
    if $pipeline_name == null {
        return $raw.prompt
    }

    let pipeline_path = ($repo_dir | path join "pipelines" $"($pipeline_name).yaml")
    if not ($pipeline_path | path exists) {
        error make {msg: $"pipeline ($pipeline_name) not found for command ($raw.name)"}
    }
    let pipeline = (open $pipeline_path)
    if $pipeline.name != $pipeline_name {
        error make {msg: $"pipeline file ($pipeline_name) declares name ($pipeline.name)"}
    }

    let instructions = ($raw.stepInstructions? | default {})
    let skills = ($pipeline.steps | get skill)
    let stale = ($instructions | columns | where {|skill| $skill not-in $skills })
    if ($stale | is-not-empty) {
        let stale_list = ($stale | str join ", ")
        error make {msg: $"command ($raw.name) has instructions for non-pipeline steps: ($stale_list)"}
    }

    let workflow = ($pipeline.steps | enumerate | each {|entry|
        let step = $entry.item
        let detail = ($instructions | get -o $step.skill | default "")
        let qualifiers = ([
            (if ($step.optional? | default false) { "optional" })
            (if ($step.loop? | default null) == "per-task" { "repeat per task" })
        ] | compact)
        let qualifier = if ($qualifiers | is-empty) {
            ""
        } else {
            let qualifier_list = ($qualifiers | str join ", ")
            $" [($qualifier_list)]"
        }
        let suffix = if ($detail | str trim | is-empty) {
            "."
        } else {
            $" — ($detail | str trim)"
        }
        $"($entry.index + 1). Run godmode:($step.skill)($qualifier)($suffix)"
    } | str join "\n")

    let composed = ([($raw.prompt | str trim) $"## Workflow — pipeline: ($pipeline_name)" $workflow] | str join "\n\n")
    $"($composed)\n"
}

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

def render-target [commands: list<record>, target: string, out_dir: path] {
    for stale in (glob ($out_dir | path join "gm-*.md") | sort) {
        rm $stale
    }
    for command in $commands {
        let out_path = ($out_dir | path join $command.out_name)
        render-content $command $target | save $out_path
    }
}

def cleanup-scratch [paths: list<path>] {
    for path in $paths {
        if ($path | path exists) {
            rm --recursive --force $path
        }
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
    let prompt = (compose-prompt $raw $repo_dir)
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

let backup_root = ($repo_dir | path join ".command-projection-backup")
let claude_stage = ($stage_root | path join "claude")
let opencode_stage = ($stage_root | path join "opencode")
let claude_backup = ($backup_root | path join "claude")
let opencode_backup = ($backup_root | path join "opencode")
let opencode_had_target = ($opencode_dir | path exists)

cleanup-scratch [$stage_root $backup_root]

try {
    mkdir $stage_root
    cp --recursive $commands_dir $claude_stage
    if $opencode_had_target {
        cp --recursive $opencode_dir $opencode_stage
    } else {
        mkdir $opencode_stage
    }
    render-target $commands "claude" $claude_stage
    render-target $commands "opencode" $opencode_stage

    # Validate both destination parents before replacing either complete directory.
    mkdir ($opencode_dir | path dirname)
    mkdir $backup_root

    mv $commands_dir $claude_backup
    mv $claude_stage $commands_dir

    if $opencode_had_target {
        mv $opencode_dir $opencode_backup
    }
    mv $opencode_stage $opencode_dir

    cleanup-scratch [$stage_root $backup_root]
} catch {
    if ($opencode_backup | path exists) {
        if ($opencode_dir | path exists) {
            rm --recursive --force $opencode_dir
        }
        mv $opencode_backup $opencode_dir
    } else if not $opencode_had_target and ($opencode_dir | path exists) {
        rm --recursive --force $opencode_dir
    }
    if ($claude_backup | path exists) {
        if ($commands_dir | path exists) {
            rm --recursive --force $commands_dir
        }
        mv $claude_backup $commands_dir
    }
    cleanup-scratch [$stage_root $backup_root]
    error make {msg: "command projection install failed; restored previous projections"}
}

print $"($commands | length) commands projected to both targets"
