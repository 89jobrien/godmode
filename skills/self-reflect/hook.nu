#!/usr/bin/env nu
# self-reflect/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run self-reflect
