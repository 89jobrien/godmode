#!/usr/bin/env nu
# cap/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run cap
