#!/usr/bin/env nu
# wave-integration/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run wave-integration
