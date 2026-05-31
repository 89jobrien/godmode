#!/usr/bin/env nu
# memory-bank-update-remind.nu — Stop hook (thin wrapper)
# Delegates to `godmode memory-banking remind`. Always exits 0.

let input = open --raw /dev/stdin | from json

let result = do { godmode memory-banking remind } | complete
if $result.exit_code == 0 and ($result.stdout | str trim | str length) > 0 {
    print $result.stdout
}

exit 0
