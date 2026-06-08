#!/usr/bin/env nu
# ci-fix/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run ci-fix
