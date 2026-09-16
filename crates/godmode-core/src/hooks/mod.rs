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
pub mod registry;
pub use registry::{
    CoverageIssueKind, HookClient, HookRegistration, diagnose_coverage, generate_manifest,
};
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
pub mod trace_log;
pub mod using_godmode;
pub mod verification;
pub mod wave_integration;
pub mod writing_plans;

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn registry_projects_required_hooks() {
        let manifest = generate_manifest(HookClient::Claude);
        let commands: Vec<_> = crate::integrations::hook_runner::list_hooks_from_json(&manifest)
            .into_iter()
            .map(|(_, _, command)| command)
            .collect();

        for required in [
            "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-bash-nag.nu",
            "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-pipeline-step.nu",
            "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/memory-bank-update-remind.nu",
        ] {
            assert!(
                commands.iter().any(|command| command == required),
                "missing {required}"
            );
        }
    }

    #[test]
    fn tracked_manifest_matches_registry_projection() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tracked: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("hooks/hooks.json")).unwrap())
                .unwrap();
        assert_eq!(tracked, generate_manifest(HookClient::Claude));
    }

    #[test]
    fn coverage_reports_each_diagnostic_kind() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("hooks/scripts")).unwrap();
        std::fs::write(root.path().join("hooks/scripts/active.nu"), "").unwrap();
        let registry = vec![
            HookRegistration::active(
                "active",
                HookClient::Claude,
                "Stop",
                "*",
                "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/active.nu",
                10,
            ),
            HookRegistration::active(
                "missing",
                HookClient::Claude,
                "Stop",
                "*",
                "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/missing.nu",
                10,
            ),
            HookRegistration::superseded(
                "old",
                "active",
                "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/old.nu",
            ),
        ];
        let manifest = serde_json::json!({"hooks": {"Stop": [{"matcher": "*", "hooks": [
            {"type": "command", "command": "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/active.nu", "timeout": 10},
            {"type": "command", "command": "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/active.nu", "timeout": 10},
            {"type": "command", "command": "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/old.nu", "timeout": 10},
            {"type": "command", "command": "nu unknown.nu", "timeout": 10}
        ]}]}});

        let report = diagnose_coverage(root.path(), HookClient::Claude, &registry, &manifest);
        assert!(
            report
                .iter()
                .any(|issue| issue.kind == CoverageIssueKind::Unregistered)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.kind == CoverageIssueKind::Missing)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.kind == CoverageIssueKind::Duplicate)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.kind == CoverageIssueKind::Superseded)
        );
    }
}
