#!/usr/bin/env nu
# mini-context-graph/hook.nu — delegates to Rust implementation.
open --raw /dev/stdin | godmode hook run mini-context-graph
