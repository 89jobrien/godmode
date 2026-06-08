#!/usr/bin/env nu
# moa/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run moa
