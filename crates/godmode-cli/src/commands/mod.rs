//! Command handlers extracted from `main.rs`.
//!
//! Each module owns one subcommand family and returns a shared `anyhow::Result`.

pub(crate) mod hook;
pub(crate) mod task;

pub(crate) use hook::run_hook_action;
pub(crate) use task::run_task_action;
