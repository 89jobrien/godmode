#!/usr/bin/env nu
# refactoring/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run refactoring
