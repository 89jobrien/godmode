#!/usr/bin/env nu
# context-map/helpers/search-commands.nu
# Quick reference commands for building a context map.
# Not executed automatically — used as a reference by the agent.

# Search for a type/function across Rust files
# rg "TypeName|fn_name" --type rust -l

# Find trait implementations
# rg "impl TraitName" --type rust -l

# Find module declarations and imports
# rg "^mod context|use .*context" --type rust -l

# Who imports this module?
# rg "use godmode_core::context" --type rust -l

# What does this file depend on?
# rg "^use " crates/godmode-core/src/context.rs

# Find tests for a module
# rg "context" --type rust crates/ -l | rg "tests?|spec"

# Find reference patterns (similar implementations)
# rg "#\[derive.*Serialize" --type rust -l | head -10
