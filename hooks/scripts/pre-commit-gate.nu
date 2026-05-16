#!/usr/bin/env nu
# pre-commit-gate.nu — PreToolUse/Bash hook
# Intercepts `git commit` commands and runs cargo fmt + clippy before allowing them.
# Communicates decisions via JSON on stdout; diagnostics go to stderr.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")

# Split the command on pipeline operators (&&, ;, |) and check whether any
# segment is a git commit invocation. Naive str-contains would trigger on
# commands like `cargo fmt --all && git commit -m "..."`, blocking the fmt
# step before it could run.
let segments = ($cmd | split row -r '&&|;|\|' | each { str trim })
let is_commit = ($segments | any { |seg|
    ($seg | str starts-with "git commit") or
    (($seg | str starts-with "git -C") and ($seg | str contains "commit"))
})

if not $is_commit {
    print '{"decision":"approve"}'
    exit 0
}

# Skip if --no-verify explicitly requested (user override)
if ($cmd | str contains "--no-verify") {
    print '{"decision":"approve"}'
    exit 0
}

# Check we are inside a Cargo workspace
let cargo_result = do { cargo locate-project --workspace --message-format plain } | complete
if $cargo_result.exit_code != 0 {
    print '{"decision":"approve"}'
    exit 0
}

print -e "[godmode:pre-commit-gate] git commit detected — running cargo gates..."

# fmt check
let fmt = do { cargo fmt --all --check } | complete
if $fmt.exit_code != 0 {
    print -e $fmt.stdout
    print -e $fmt.stderr
    let reason = "cargo fmt --check failed. Run `cargo fmt --all` to fix."
    print -e $"[godmode:pre-commit-gate] BLOCKED: ($reason)"
    print $'{"decision":"block","reason":"($reason)"}'
    exit 0
}
print -e "[godmode:pre-commit-gate] fmt: ok"

# clippy
let clippy = do { cargo clippy --workspace -- -D warnings } | complete
if $clippy.exit_code != 0 {
    print -e $clippy.stdout
    print -e $clippy.stderr
    let reason = "cargo clippy failed. Fix warnings before committing."
    print -e $"[godmode:pre-commit-gate] BLOCKED: ($reason)"
    print $'{"decision":"block","reason":"($reason)"}'
    exit 0
}
print -e "[godmode:pre-commit-gate] clippy: ok"

print -e "[godmode:pre-commit-gate] all gates passed."
print '{"decision":"approve"}'
exit 0
