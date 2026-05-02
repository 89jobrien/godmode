//! GitHub integrations: issue import and CI triage.

pub mod ci;
pub mod issues;

// Re-export the public surface so existing `use crate::integrations::gh::*` paths keep working.
pub use ci::{CiFailureClass, CiTriageResult, ci_triage, classify_log, fix_hint};
pub use issues::{issue_close, issues_to_tasks, parse_issue_list, pull_issues};
