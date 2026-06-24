#!/usr/bin/env nu

def repo-root [] {
    let result = (do { git rev-parse --show-toplevel } | complete)
    if $result.exit_code != 0 {
        error make { msg: $"not in a git repo: ($result.stderr | str trim)" }
    }
    $result.stdout | str trim
}

def default-target [] {
    if "AICHAT_CONFIG_DIR" in $env {
        $env.AICHAT_CONFIG_DIR
    } else {
        $env.HOME | path join ".config" "aichat"
    }
}

def ensure-config [target: path] {
    let config = ($target | path join "config.yaml")
    if not ($config | path exists) {
        "save: false\nfunction_calling: true\n" | save --force $config
    }
}

def copy-file [source: path, dest: path, --force] {
    mkdir ($dest | path dirname)
    if ($dest | path exists) {
        let current = (open --raw $dest)
        let incoming = (open --raw $source)
        if $current == $incoming {
            return
        }
        if not $force {
            error make {
                msg: $"refusing to overwrite existing file without --force: ($dest)"
            }
        }
    }
    cp $source $dest
}

def merge-agents-list [source: path, dest: path] {
    mkdir ($dest | path dirname)
    let existing = if ($dest | path exists) {
        open $dest | lines
    } else {
        []
    }
    let incoming = open $source | lines
    let merged = ($existing | append $incoming | where { |line| ($line | str trim) != "" } | uniq)
    $merged | str join "\n" | save --force $dest
    "\n" | save --append $dest
}

def validate-install [target: path] {
    let result = (
        do {
            with-env { AICHAT_CONFIG_DIR: ($target | path expand) } {
                aichat --agent godmode-assistant --info
            }
        } | complete
    )
    if $result.exit_code != 0 {
        error make { msg: $"aichat agent validation failed: ($result.stderr | str trim)" }
    }
    $result.stdout
}

def warn-default-config-dir [target: path] {
    if "AICHAT_CONFIG_DIR" in $env {
        print $"AICHAT_CONFIG_DIR is set to ($env.AICHAT_CONFIG_DIR)"
        return
    }
    let result = (do { aichat --info } | complete)
    if $result.exit_code != 0 {
        print $"warning: unable to inspect default aichat config: ($result.stderr | str trim)"
        return
    }
    let matches = ($result.stdout | lines | where { |line| $line =~ "^config_file" })
    let config_line = if ($matches | is-empty) { "" } else { $matches | first }
    let expected = ($target | path expand | path join "config.yaml")
    if ($config_line | is-empty) {
        print "warning: unable to locate config_file in aichat --info output"
    } else if not ($config_line | str contains $expected) {
        print $"warning: default aichat config differs from install target: ($config_line)"
        print $"set AICHAT_CONFIG_DIR=($target | path expand) when using this installed pack"
    }
}

def main [
    --target: path
    --dry-run
    --force
] {
    let root = repo-root
    let source = ($root | path join "aichat")
    let requested_target = if $target != null {
        $target
    } else {
        default-target
    }
    let install_target = if $dry_run {
        mktemp -d
    } else {
        $requested_target
    }
    let target = ($install_target | path expand)

    mkdir $target
    ensure-config $target

    let source_agents = ($source | path join "functions" "agents.txt")
    let target_agents = ($target | path join "functions" "agents.txt")
    merge-agents-list $source_agents $target_agents

    let source_index = ($source | path join "functions" "agents" "godmode-assistant" "index.yaml")
    let target_index = ($target | path join "functions" "agents" "godmode-assistant" "index.yaml")
    copy-file $source_index $target_index --force=$force

    let source_config = ($source | path join "agents" "godmode-assistant" "config.yaml")
    let target_config = ($target | path join "agents" "godmode-assistant" "config.yaml")
    copy-file $source_config $target_config --force=$force

    for role in [gm-planner gm-reviewer gm-debugger gm-verifier] {
        let source_role = ($source | path join "roles" $"($role).md")
        let target_role = ($target | path join "roles" $"($role).md")
        copy-file $source_role $target_role --force=$force
    }

    let info = validate-install $target
    if $dry_run {
        print $"dry-run validated godmode AIChat system in ($target)"
        print $"requested install target remains unchanged: ($requested_target | path expand)"
    } else {
        print $"installed godmode AIChat system to ($target)"
    }
    warn-default-config-dir $target
    print ""
    print ($info | lines | first 12 | str join "\n")
}
