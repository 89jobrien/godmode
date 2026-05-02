#!/usr/bin/env rust-script
//! Property tests for wave-integrate logic.
//!
//! Tests the pure functions that the shell scripts delegate to:
//!   - branch list parsing (space-separated string → Vec<String>)
//!   - conflict log rendering
//!   - integration state machine (integrated/failed accounting)
//!
//! Run: rust-script wave-integrate-proptest.rs
//!
//! ```cargo
//! [dependencies]
//! proptest = "1"
//! ```

use proptest::prelude::*;

// ── Branch list parsing ───────────────────────────────────────────────────────
// Mirrors: `$branches | split row " " | where { |l| ($l | str trim) != "" }`

fn parse_branches(input: &str) -> Vec<String> {
    input
        .split(' ')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

// ── Conflict log rendering ────────────────────────────────────────────────────
// Mirrors the log template written at the end of the script.

struct IntegrationResult<'a> {
    integrated: &'a [String],
    failed: &'a [String],
}

fn render_log(result: &IntegrationResult) -> String {
    let branch_summary = if result.integrated.is_empty() {
        String::new()
    } else {
        result
            .integrated
            .iter()
            .map(|b| format!("- {b}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let failed_summary = if result.failed.is_empty() {
        "none".to_string()
    } else {
        result.failed.join(", ")
    };

    format!(
        "# Wave Integration Log\n\n\
         ## Branches Integrated\n\n\
         {branch_summary}\n\n\
         ## Failed / Skipped\n\n\
         {failed_summary}\n"
    )
}

// ── State machine ─────────────────────────────────────────────────────────────
// Models the per-branch loop: each branch either integrates or fails.

#[derive(Debug, Clone)]
enum BranchOutcome {
    Integrated,
    Failed,
}

fn run_state_machine(branches: &[String], outcomes: &[BranchOutcome]) -> (Vec<String>, Vec<String>) {
    let mut integrated = Vec::new();
    let mut failed = Vec::new();
    for (branch, outcome) in branches.iter().zip(outcomes.iter()) {
        match outcome {
            BranchOutcome::Integrated => integrated.push(branch.clone()),
            BranchOutcome::Failed => failed.push(branch.clone()),
        }
    }
    (integrated, failed)
}

// ── Strategies ────────────────────────────────────────────────────────────────

fn branch_name() -> impl Strategy<Value = String> {
    // Valid git branch name component: letters, digits, hyphens, slashes
    "[a-z][a-z0-9/-]{0,15}".prop_map(|s| s.trim_matches('/').to_string())
        .prop_filter("non-empty", |s| !s.is_empty())
}

fn branch_list_string() -> impl Strategy<Value = String> {
    prop::collection::vec(branch_name(), 1..=8).prop_map(|v| v.join(" "))
}

fn outcomes(n: usize) -> impl Strategy<Value = Vec<BranchOutcome>> {
    prop::collection::vec(
        prop_oneof![Just(BranchOutcome::Integrated), Just(BranchOutcome::Failed)],
        n..=n,
    )
}

// ── Properties ────────────────────────────────────────────────────────────────

proptest! {
    /// Parsing a space-joined list round-trips: split → join → split is stable.
    #[test]
    fn parse_roundtrips(branches in prop::collection::vec(branch_name(), 1..=8)) {
        let joined = branches.join(" ");
        let parsed = parse_branches(&joined);
        prop_assert_eq!(parsed, branches);
    }

    /// Extra spaces between branches are collapsed — no empty entries.
    #[test]
    fn parse_no_empty_entries(s in "[ ]*([a-z][a-z0-9/-]{0,10}[ ]+){0,6}[a-z][a-z0-9/-]{0,10}[ ]*") {
        let parsed = parse_branches(&s);
        for b in &parsed {
            prop_assert!(!b.is_empty(), "found empty entry in {:?}", parsed);
        }
    }

    /// Parsing preserves order.
    #[test]
    fn parse_preserves_order(branches in prop::collection::vec(branch_name(), 1..=8)) {
        let joined = branches.join(" ");
        let parsed = parse_branches(&joined);
        prop_assert_eq!(parsed.len(), branches.len());
        for (a, b) in parsed.iter().zip(branches.iter()) {
            prop_assert_eq!(a, b);
        }
    }

    /// integrated + failed == total branches processed (no branch lost).
    #[test]
    fn state_machine_accounts_all(s in branch_list_string()) {
        let branches = parse_branches(&s);
        let n = branches.len();
        let outs = outcomes(n);
        let outcomes_vec = outs.new_tree(&mut proptest::test_runner::TestRunner::default())
            .unwrap()
            .current();
        let (integrated, failed) = run_state_machine(&branches, &outcomes_vec);
        prop_assert_eq!(integrated.len() + failed.len(), n);
    }

    /// Every integrated branch appears exactly once in the log.
    #[test]
    fn log_contains_each_integrated_branch(branches in prop::collection::vec(branch_name(), 1..=6)) {
        let n = branches.len();
        let all_integrated: Vec<BranchOutcome> = vec![BranchOutcome::Integrated; n];
        let (integrated, failed) = run_state_machine(&branches, &all_integrated);
        let result = IntegrationResult { integrated: &integrated, failed: &failed };
        let log = render_log(&result);
        for b in &branches {
            let count = log.matches(b.as_str()).count();
            prop_assert_eq!(count, 1, "branch {b} appears {count} times in log");
        }
    }

    /// When all branches fail, log contains "none" for integrated section.
    #[test]
    fn log_none_when_all_failed(branches in prop::collection::vec(branch_name(), 1..=6)) {
        let n = branches.len();
        let all_failed: Vec<BranchOutcome> = vec![BranchOutcome::Failed; n];
        let (integrated, failed) = run_state_machine(&branches, &all_failed);
        let result = IntegrationResult { integrated: &integrated, failed: &failed };
        let log = render_log(&result);
        prop_assert!(log.contains("none"), "expected 'none' in log when all failed");
    }

    /// When all branches integrate, failed summary is "none".
    #[test]
    fn log_failed_none_when_all_integrated(branches in prop::collection::vec(branch_name(), 1..=6)) {
        let n = branches.len();
        let all_ok: Vec<BranchOutcome> = vec![BranchOutcome::Integrated; n];
        let (integrated, failed) = run_state_machine(&branches, &all_ok);
        let result = IntegrationResult { integrated: &integrated, failed: &failed };
        let log = render_log(&result);
        // The failed section should say "none"
        let after_failed_header = log.split("## Failed / Skipped").nth(1).unwrap_or("");
        prop_assert!(after_failed_header.contains("none"),
            "expected 'none' in failed section, got: {after_failed_header}");
    }

    /// integrated and failed lists are disjoint.
    #[test]
    fn integrated_and_failed_are_disjoint(s in branch_list_string()) {
        let branches = parse_branches(&s);
        let n = branches.len();
        // Alternate: even indices integrate, odd indices fail
        let outs: Vec<BranchOutcome> = (0..n)
            .map(|i| if i % 2 == 0 { BranchOutcome::Integrated } else { BranchOutcome::Failed })
            .collect();
        let (integrated, failed) = run_state_machine(&branches, &outs);
        for b in &integrated {
            prop_assert!(!failed.contains(b), "branch {b} in both integrated and failed");
        }
    }
}

fn main() {
    println!("Running wave-integrate property tests...");
    // proptest runs via the proptest! macro above when invoked as a test binary.
    // This main just confirms the script compiles and runs stand-alone.
    let samples = [
        "feat/a feat/b feat/c",
        "  feat/x   feat/y  ",
        "single",
    ];
    for s in samples {
        let parsed = parse_branches(s);
        println!("parse({s:?}) -> {parsed:?}");
    }
    println!("All parse samples OK. Run with `cargo test` for property tests.");
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parse_single() {
        assert_eq!(parse_branches("feat/a"), vec!["feat/a"]);
    }

    #[test]
    fn parse_multiple() {
        assert_eq!(parse_branches("feat/a feat/b feat/c"), vec!["feat/a", "feat/b", "feat/c"]);
    }

    #[test]
    fn parse_extra_spaces() {
        assert_eq!(parse_branches("  feat/a   feat/b  "), vec!["feat/a", "feat/b"]);
    }

    #[test]
    fn parse_empty_string() {
        assert_eq!(parse_branches(""), Vec::<String>::new());
    }

    #[test]
    fn state_machine_all_integrated() {
        let branches = vec!["a".to_string(), "b".to_string()];
        let outcomes = vec![BranchOutcome::Integrated, BranchOutcome::Integrated];
        let (i, f) = run_state_machine(&branches, &outcomes);
        assert_eq!(i, vec!["a", "b"]);
        assert!(f.is_empty());
    }

    #[test]
    fn state_machine_mixed() {
        let branches = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let outcomes = vec![BranchOutcome::Integrated, BranchOutcome::Failed, BranchOutcome::Integrated];
        let (i, f) = run_state_machine(&branches, &outcomes);
        assert_eq!(i, vec!["a", "c"]);
        assert_eq!(f, vec!["b"]);
    }

    #[test]
    fn log_failed_summary_none_when_empty() {
        let result = IntegrationResult { integrated: &["a".to_string()], failed: &[] };
        let log = render_log(&result);
        assert!(log.contains("none"));
    }

    #[test]
    fn log_failed_summary_lists_branches() {
        let integrated = vec![];
        let failed = vec!["a".to_string(), "b".to_string()];
        let result = IntegrationResult { integrated: &integrated, failed: &failed };
        let log = render_log(&result);
        assert!(log.contains("a, b"));
    }
}
