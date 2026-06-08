#!/usr/bin/env nu
# pre-commit-gate.nu — PreToolUse/Bash hook: delegates to `godmode hook run pre-commit-gate`.

let input = open --raw /dev/stdin | from json
let cmd = ($input | get --optional tool_input.command | default "")

# Only intercept git commit commands
let segments = ($cmd | split row -r '&&|;|\|' | each { str trim })
let is_commit = ($segments | any { |seg|
    let a = ($seg | str starts-with "git commit")
    let b = (($seg | str starts-with "git -C") and ($seg | str contains "commit"))
    $a or $b
})

if not $is_commit {
    print '{"decision":"approve"}'
    exit 0
}

# Skip if --no-verify explicitly requested
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

# Delegate to Rust
let result = do { godmode hook run pre-commit-gate --json } | complete
if $result.exit_code == 0 {
    print '{"decision":"approve"}'
} else {
    let reason = ($result.stderr | str trim)
    print -e $reason
    print $'{"decision":"block","reason":"($reason)"}'
}
exit 0
