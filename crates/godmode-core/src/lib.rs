//! Core domain and integration library for the `godmode` CLI.
//!
//! This crate contains task-graph domain logic, session orchestration, and
//! external-tool integrations. `godmode-cli` is intentionally thin and delegates
//! nearly all behavior to modules exported here.

pub mod agent;
pub mod agent_index;
pub mod builder;
pub mod cache;
pub mod config;
pub mod context;
pub mod detect;
pub mod dispatch;
pub mod doctor;
pub mod graph;
pub mod hooks;
pub mod init;
pub mod insights;
pub mod integrations;
pub mod memory_banking;
pub mod model;
pub mod pipeline;
pub mod plan;
pub mod policy;
pub mod registry;
pub mod release;
pub mod report_index;
pub mod review;
pub mod sarif;
pub mod scaffold;
pub mod session;
pub mod session_trace;
pub mod skill;
pub mod templates;
pub mod test_check;
pub mod verify;
pub mod wave;
pub mod workflow;
pub mod worktree;

#[cfg(feature = "testing")]
pub mod testing;
