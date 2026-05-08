#!/usr/bin/env nu
# pre-commit-gate.nu — PreToolUse/Bash hook
# Intercepts `git commit` commands and runs cargo fmt + clippy before allowing them.
# Blocks the commit if checks fail. Always exits 0 for non-commit commands.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")

# Only act on git commit commands
if not ($cmd | str contains "git commit") {
    exit 0
}

# Skip if --no-verify explicitly requested (user override)
if ($cmd | str contains "--no-verify") {
    exit 0
}

# Check we are inside a Cargo workspace
let cargo_result = do { cargo locate-project --workspace --message-format plain } | complete
if $cargo_result.exit_code != 0 {
    exit 0
}

print "[godmode:pre-commit-gate] git commit detected — running cargo gates first..."

# fmt check
let fmt = do { cargo fmt --all --check } | complete
if $fmt.exit_code != 0 {
    print $fmt.stdout
    print $fmt.stderr
    print "[godmode:pre-commit-gate] BLOCKED: cargo fmt --check failed. Run `cargo fmt --all` to fix."
    exit 2
}
print "[godmode:pre-commit-gate] fmt: ok"

# clippy
let clippy = do { cargo clippy --workspace -- -D warnings } | complete
if $clippy.exit_code != 0 {
    print $clippy.stdout
    print $clippy.stderr
    print "[godmode:pre-commit-gate] BLOCKED: cargo clippy failed. Fix warnings before committing."
    exit 2
}
print "[godmode:pre-commit-gate] clippy: ok"

print "[godmode:pre-commit-gate] all gates passed — proceeding with commit."
exit 0
