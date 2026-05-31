#!/usr/bin/env nu
# memory-bank-inject.nu — SessionStart hook (thin wrapper)
# Delegates to `godmode memory-banking inject`. Always exits 0.

let input = open --raw /dev/stdin | from json

let result = do { godmode memory-banking inject } | complete
if $result.exit_code == 0 and ($result.stdout | str trim | str length) > 0 {
    print $result.stdout
}

exit 0
