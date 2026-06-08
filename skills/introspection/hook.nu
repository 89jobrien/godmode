#!/usr/bin/env nu
# introspection/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run introspection
