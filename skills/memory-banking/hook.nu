#!/usr/bin/env nu
# memory-banking/hook.nu — SessionStart hook (delegates to CLI).
# Prints memory-bank contents if .ctx/memory-banking/ exists.

let input = open --raw /dev/stdin | from json

let result = do { godmode memory-banking inject } | complete
if $result.exit_code == 0 and ($result.stdout | str trim | str length) > 0 {
    print $result.stdout
}

exit 0
