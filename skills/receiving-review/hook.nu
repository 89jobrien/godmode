#!/usr/bin/env nu
# receiving-review/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run receiving-review
