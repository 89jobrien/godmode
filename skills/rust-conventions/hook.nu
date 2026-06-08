#!/usr/bin/env nu
# rust-conventions/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run rust-conventions
