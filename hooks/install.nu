#!/usr/bin/env nu
# install.nu — install godmode git hooks into the current repository's git dir
#
# Usage (from the repo root or any worktree):
#   nu hooks/install.nu
#
# The hook scripts are copied into `<git-common-dir>/hooks/godmode/` and the
# installed wrappers resolve them relative to their own location
# (`$(dirname "$0")/godmode/...`). Installed hooks therefore keep working
# after the checkout they were installed from is moved or deleted — e.g. a
# worktree being removed. Re-run this script to refresh the copies.

let git_common_result = (run-external "git" "rev-parse" "--git-common-dir" | complete)
if $git_common_result.exit_code != 0 {
    print "install.nu: must be run inside a git repository"
    exit 1
}

# In a bare repository the git dir is the repo dir itself, so installing
# would scatter wrapper files into tracked or shared directories.
let bare_result = (run-external "git" "rev-parse" "--is-bare-repository" | complete)
if ($bare_result.exit_code == 0) and (($bare_result.stdout | str trim) == "true") {
    print "install.nu: refusing to install hooks into a bare repository"
    exit 1
}

let hooks_dir = $"($git_common_result.stdout | str trim | path expand)/hooks"
let payload_dir = $"($hooks_dir)/godmode"

# Resolve the plugin root: prefer $CLAUDE_PLUGIN_ROOT, fall back to the
# directory that contains this script.
let plugin_root = (
    $env.CLAUDE_PLUGIN_ROOT?
    | default ($env.CURRENT_FILE? | path dirname | path dirname | default $env.PWD)
)

let src_dir = $"($plugin_root)/hooks"

if not ($src_dir | path exists) {
    print $"install.nu: source not found at ($src_dir)"
    exit 1
}

# Copy the hook scripts and their shared library into the git dir so the
# installed hooks are self-contained.
mkdir $payload_dir
mkdir $"($payload_dir)/lib"

let copied = ["pre-commit.nu" "pre-push.nu" "post-commit.nu"]
for script in $copied {
    let src = $"($src_dir)/($script)"
    if ($src | path exists) {
        cp --force $src $"($payload_dir)/($script)"
    }
}
cp --force $"($src_dir)/lib/godmode-hook-lib.nu" $"($payload_dir)/lib/godmode-hook-lib.nu"

# Write a thin wrapper per hook that execs the copied script relative to the
# wrapper's own location, so the install survives source-checkout removal.
for script in $copied {
    if not ($payload_dir | path join $script | path exists) { continue }
    let hook_name = ($script | str replace ".nu" "")
    let target = $"($hooks_dir)/($hook_name)"
    let wrapper = (
        '#!/bin/sh' + "\n" +
        'exec nu "$(dirname "$0")/godmode/' + $script + '" "$@"' + "\n"
    )
    $wrapper | save --force $target
    run-external "chmod" "+x" $target
    print $"($hook_name) hook installed: ($target)"
    print ('  -> $(dirname "$0")/godmode/' + $script)
}

# Run hook migrations
let migrate_result = do { godmode hook migrate } | complete
if $migrate_result.exit_code != 0 {
    print "install.nu: hook migrate failed — check hooks/migrations/"
}
