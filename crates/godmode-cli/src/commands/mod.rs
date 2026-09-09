//! Command handlers extracted from `main.rs`.
//!
//! Each module owns one subcommand family and returns a shared `anyhow::Result`.

pub(crate) mod agent;
pub(crate) mod command_renderer;
pub(crate) mod hook;
pub(crate) mod task;
pub(crate) mod trace;

pub(crate) use agent::run_agent_action;
pub(crate) use command_renderer::run_command_action;
pub(crate) use hook::run_hook_action;
pub(crate) use task::run_task_action;
pub(crate) use trace::run_trace_action;
