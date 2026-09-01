//! ConformanceTest — trait for implementing godmode conformance tests.

use serde::Serialize;

use super::context::TestContext;

/// Category of conformance test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum TestCategory {
    /// Single function or behaviour.
    Unit,
    /// Component interactions.
    Integration,
    /// Boundary conditions, error handling.
    EdgeCase,
}

impl TestCategory {
    /// Returns the stable snake-case label used in reports and filters.
    pub fn as_str(self) -> &'static str {
        match self {
            TestCategory::Unit => "unit",
            TestCategory::Integration => "integration",
            TestCategory::EdgeCase => "edge_case",
        }
    }
}

/// Result of a conformance test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum TestResult {
    /// The conformance check completed successfully.
    Pass,
    /// The conformance check failed.
    Fail {
        /// Diagnostic explanation of the failure.
        reason: String,
    },
    /// The conformance check was not executed.
    Skipped {
        /// Explanation of why the check was skipped.
        reason: String,
    },
}

impl TestResult {
    /// Returns whether this result represents a passing test.
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    /// Returns whether this result represents a failed test.
    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail { .. })
    }

    /// Returns whether this result represents a skipped test.
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skipped { .. })
    }
}

/// Trait all conformance tests implement.
pub trait ConformanceTest: Send + Sync {
    /// Returns the human-readable test name.
    fn name(&self) -> &str;
    /// Returns the crate whose behavior the test covers.
    fn crate_name(&self) -> &str;
    /// Returns the test's conformance category.
    fn category(&self) -> TestCategory;
    /// Executes the test using the supplied context.
    fn run(&self, ctx: &mut TestContext) -> TestResult;

    /// Returns the crate-qualified stable identifier for the test.
    fn id(&self) -> String {
        format!("{}::{}", self.crate_name(), self.name())
    }
}
