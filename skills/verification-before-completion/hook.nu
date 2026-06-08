#!/usr/bin/env nu
# verification-before-completion/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run verification-before-completion
