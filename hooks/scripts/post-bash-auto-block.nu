#!/usr/bin/env nu
# post-bash-auto-block.nu — PostToolUse/Bash hook: delegates to `godmode hook run auto-block`.

let input = open --raw /dev/stdin

# Delegate to Rust — pass stdin JSON through
let result = do { $input | godmode hook run auto-block } | complete
if not ($result.stderr | str trim | is-empty) {
    print -e $result.stderr
}
exit 0
