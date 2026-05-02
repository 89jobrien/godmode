#!/usr/bin/env nu
# install.nu — install godmode pre-commit hook into the current repo's .git/hooks/
#
# Usage (from the repo root):
#   nu hooks/install.nu

let git_root_result = (run-external "git" "rev-parse" "--show-toplevel" | complete)
if $git_root_result.exit_code != 0 {
    print "install.nu: must be run inside a git repository"
    exit 1
}

let git_root = ($git_root_result.stdout | str trim)
let hooks_dir = $"($git_root)/.git/hooks"
let target = $"($hooks_dir)/pre-commit"

# Resolve the plugin root: prefer $CLAUDE_PLUGIN_ROOT, fall back to the
# directory that contains this script.
let plugin_root = (
    $env.CLAUDE_PLUGIN_ROOT?
    | default ($env.CURRENT_FILE? | path dirname | path dirname | default $git_root)
)

let src = $"($plugin_root)/hooks/pre-commit.nu"

if not ($src | path exists) {
    print $"install.nu: source not found at ($src)"
    exit 1
}

# Write a thin wrapper that execs the nu script
let wrapper = $"#!/bin/sh\nexec nu \"($src)\" \"$@\"\n"
$wrapper | save --force $target
run-external "chmod" "+x" $target

print $"pre-commit hook installed: ($target)"
print $"  -> ($src)"
