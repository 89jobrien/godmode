#!/usr/bin/env nu
# parallel-agents/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run parallel-agents
