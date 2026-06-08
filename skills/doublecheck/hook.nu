#!/usr/bin/env nu
# doublecheck/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run doublecheck
