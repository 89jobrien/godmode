//! Shared conformance harness building blocks.
//!
//! Re-exports provide a single import surface for test modules.

pub mod comparison;
pub mod context;
pub mod fixtures;
pub mod logging;
pub mod runner;
pub mod traits;

pub use context::TestContext;
pub use runner::{ReportConfig, ReportGenerator, TestRunner, TestSummary};
pub use traits::{ConformanceTest, TestCategory, TestResult};

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    struct Dummy;
    impl ConformanceTest for Dummy {
        fn name(&self) -> &str {
            "dummy"
        }
        fn crate_name(&self) -> &str {
            "core"
        }
        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }
        fn run(&self, _: &mut TestContext) -> TestResult {
            TestResult::Pass
        }
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_harness_forwarding_apis_remain_callable() {
        let dir = tempfile::tempdir().unwrap();
        let loader = fixtures::FixtureLoader::with_dir(dir.path());
        assert!(loader.list().is_empty());
        assert_eq!(
            fixtures::fixture_dir(dir.path()),
            dir.path().join("tests/conformance/fixtures/expected")
        );
        let mut runner = TestRunner::new();
        runner.add_test(Dummy);
        runner.add_tests([Dummy]);
        assert_eq!(runner.test_count(), 2);
        assert_eq!(runner.filter_crate("core").filtered_count(), 2);
        let mut context = TestContext::new();
        assert!(context.assert_f64_eq(1.0, 1.0, f64::EPSILON));
        let mut logger = logging::TestLogger::new();
        logger.set_test_name("dummy");
        logger.clear_test_name();
    }
}
