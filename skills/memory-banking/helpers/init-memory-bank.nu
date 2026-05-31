#!/usr/bin/env nu
# memory-banking/helpers/init-memory-bank.nu
# Initialize .ctx/memory-banking/ via the godmode CLI.
# Run manually: nu skills/memory-banking/helpers/init-memory-bank.nu

let result = do { godmode memory-banking init } | complete
if $result.exit_code == 0 {
    print $result.stdout
} else {
    print $"[memory-banking] init failed: ($result.stderr)"
}
