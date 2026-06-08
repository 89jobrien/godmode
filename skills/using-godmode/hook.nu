#!/usr/bin/env nu
# using-godmode/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run using-godmode
