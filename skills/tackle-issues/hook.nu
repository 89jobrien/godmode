#!/usr/bin/env nu
# tackle-issues/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run tackle-issues
