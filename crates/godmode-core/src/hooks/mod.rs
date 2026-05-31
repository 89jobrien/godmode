//! Hook implementations ported from Nushell scripts.
//!
//! Each submodule implements one lifecycle hook. The CLI dispatches to these
//! via `godmode hook run <name>`.

pub mod auto_block;
pub mod hook_context;
pub mod pre_commit;
pub mod quality_gate;
pub mod stop_guard;
