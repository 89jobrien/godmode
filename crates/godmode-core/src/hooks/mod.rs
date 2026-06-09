//! Hook implementations ported from Nushell scripts.
//!
//! Each submodule implements one lifecycle hook. The CLI dispatches to these
//! via `godmode hook run <name>`.

pub mod agent_governance;
pub mod auto_block;
pub mod brainstorm;
pub mod cap;
pub mod ci_fix;
pub mod code_review;
pub mod context_map;
pub mod design;
pub mod doublecheck;
pub mod hook_context;
pub mod introspection;
pub mod memory_banking;
pub mod merge;
pub mod mini_context_graph;
pub mod moa;
pub mod observability;
pub mod parallel_agents;
pub mod pre_commit;
pub mod quality_gate;
pub mod receiving_review;
pub mod refactoring;
pub mod rust_conventions;
pub mod self_reflect;
pub mod stop_guard;
pub mod systematic_debugging;
pub mod tackle_issues;
pub mod task_driven_dev;
pub mod task_management;
pub mod testing_philosophy;
pub mod todo_issue_sync;
pub mod using_godmode;
pub mod verification;
pub mod wave_integration;
pub mod writing_plans;
