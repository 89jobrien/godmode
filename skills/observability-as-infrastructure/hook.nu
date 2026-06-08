#!/usr/bin/env nu
# observability-as-infrastructure/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run observability-as-infrastructure
