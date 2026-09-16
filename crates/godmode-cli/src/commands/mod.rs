use std::path::Path;

use anyhow::Result;

use crate::Cmd;

mod agent;
mod ci;
mod context;
mod dispatch;
mod doctor;
mod eval;
mod graph;
mod handoff;
mod handon;
pub(crate) mod hook;
mod init;
mod insight;
mod issue;
mod memory_banking;
mod pin;
mod pipeline;
mod plan;
mod policy;
mod release;
mod review;
mod scaffold;
mod session;
mod skill;
mod status;
pub(crate) mod task;
mod test_check;
mod unpin;
mod verify;
mod visualize_graph;
mod wave;
mod workflow;
mod worktree;

pub fn dispatch(cmd: Cmd, root: &Path, json: bool, sarif: bool) -> Result<()> {
    match cmd {
        command @ Cmd::Handon { .. } => handon::handle(command, root, json, sarif),
        command @ Cmd::Handoff => handoff::handle(command, root, json, sarif),
        command @ Cmd::Session { .. } => session::handle(command, root, json, sarif),
        Cmd::Task { action } => task::run_task_action(root, json, action),
        command @ Cmd::Plan { .. } => plan::handle(command, root, json, sarif),
        command @ Cmd::Context => context::handle(command, root, json, sarif),
        command @ Cmd::Status { .. } => status::handle(command, root, json, sarif),
        Cmd::Hook { action } => hook::run_hook_action(root, json, action),
        command @ Cmd::Dispatch { .. } => dispatch::handle(command, root, json, sarif),
        command @ Cmd::Agent { .. } => agent::handle(command, root, json, sarif),
        command @ Cmd::Verify { .. } => verify::handle(command, root, json, sarif),
        command @ Cmd::Wave { .. } => wave::handle(command, root, json, sarif),
        command @ Cmd::Worktree { .. } => worktree::handle(command, root, json, sarif),
        command @ Cmd::Ci { .. } => ci::handle(command, root, json, sarif),
        command @ Cmd::Issue { .. } => issue::handle(command, root, json, sarif),
        command @ Cmd::Graph { .. } => graph::handle(command, root, json, sarif),
        command @ Cmd::Eval { .. } => eval::handle(command, root, json, sarif),
        command @ Cmd::Skill { .. } => skill::handle(command, root, json, sarif),
        command @ Cmd::Review { .. } => review::handle(command, root, json, sarif),
        command @ Cmd::Release { .. } => release::handle(command, root, json, sarif),
        command @ Cmd::Workflow { .. } => workflow::handle(command, root, json, sarif),
        command @ Cmd::VisualizeGraph { .. } => visualize_graph::handle(command, root, json, sarif),
        command @ Cmd::MemoryBanking { .. } => memory_banking::handle(command, root, json, sarif),
        command @ Cmd::Insight { .. } => insight::handle(command, root, json, sarif),
        command @ Cmd::Pipeline { .. } => pipeline::handle(command, root, json, sarif),
        command @ Cmd::Policy { .. } => policy::handle(command, root, json, sarif),
        command @ Cmd::Pin { .. } => pin::handle(command, root, json, sarif),
        command @ Cmd::Unpin => unpin::handle(command, root, json, sarif),
        command @ Cmd::Init => init::handle(command, root, json, sarif),
        command @ Cmd::Doctor => doctor::handle(command, root, json, sarif),
        command @ Cmd::Scaffold { .. } => scaffold::handle(command, root, json, sarif),
        command @ Cmd::TestCheck { .. } => test_check::handle(command, root, json, sarif),
    }
}
