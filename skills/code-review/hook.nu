#!/usr/bin/env nu
# code-review/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run code-review
