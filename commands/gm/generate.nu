#!/usr/bin/env nu

# Compatibility entry point for the former commands/gm generator path.
let legacy_dir = $env.FILE_PWD
let repo_root = ($legacy_dir | path dirname | path dirname)
^nu ($repo_root | path join "command-support" "gm" "generate.nu")
