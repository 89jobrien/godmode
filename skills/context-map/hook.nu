#!/usr/bin/env nu
# context-map/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run context-map
