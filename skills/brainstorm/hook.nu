#!/usr/bin/env nu
# brainstorm/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run brainstorm
