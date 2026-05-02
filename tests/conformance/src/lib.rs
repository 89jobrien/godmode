//! godmode-conformance — behavioural parity test harness for godmode-core.
//!
//! Mirrors the charmed_rust conformance pattern: typed `ConformanceTest` trait,
//! parallel `TestRunner`, JSON / text / GitHub Actions report generation.

pub mod dispatch_tests;
pub mod graph_tests;
pub mod harness;
pub mod plan_tests;

/// Build a `TestRunner` pre-loaded with all conformance tests.
pub fn all_tests() -> harness::TestRunner {
    let mut runner = harness::TestRunner::new();
    runner.add_boxed(graph_tests::all());
    runner.add_boxed(plan_tests::all());
    runner.add_boxed(dispatch_tests::all());
    runner
}
