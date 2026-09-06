#!/usr/bin/env nu

# Compatibility entry point. Rendering is owned by godmode-core.
let gm_dir = $env.FILE_PWD
let repo_root = ($gm_dir | path dirname | path dirname)
^cargo run --manifest-path ($repo_root | path join "Cargo.toml") -p godmode-cli -- command generate --target claude --source-dir $gm_dir --output-dir ($repo_root | path join "commands")
