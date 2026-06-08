#!/usr/bin/env nu
# testing-philosophy/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run testing-philosophy
