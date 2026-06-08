#!/usr/bin/env nu
# systematic-debugging/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run systematic-debugging
