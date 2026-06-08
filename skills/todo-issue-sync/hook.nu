#!/usr/bin/env nu
# todo-issue-sync/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run todo-issue-sync
