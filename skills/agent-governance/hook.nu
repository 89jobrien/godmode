#!/usr/bin/env nu
# agent-governance/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run agent-governance
