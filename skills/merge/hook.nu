#!/usr/bin/env nu
# merge/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run merge
