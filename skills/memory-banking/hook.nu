#!/usr/bin/env nu
# memory-banking/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run memory-banking
