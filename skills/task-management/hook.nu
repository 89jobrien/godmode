#!/usr/bin/env nu
# task-management/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run task-management
