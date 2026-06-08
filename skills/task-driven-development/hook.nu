#!/usr/bin/env nu
# task-driven-development/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run task-driven-development
