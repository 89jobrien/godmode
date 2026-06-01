#!/usr/bin/env nu
# memory-bank-update-remind.nu — Stop hook (thin wrapper)
# Delegates to `godmode memory-banking remind`. Always exits 0.

# Stop hooks run in a restricted PATH; prepend user binary dirs
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.cargo/bin" | prepend $"($env.HOME)/.local/bin")

let result = do { godmode memory-banking remind } | complete
if $result.exit_code == 0 and ($result.stdout | str trim | str length) > 0 {
    print $result.stdout
}

exit 0
