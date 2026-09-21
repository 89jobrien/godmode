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
    /// Returns the stable serialized name of this test category.
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
    Pass,
    Fail { reason: String },
    Skipped { reason: String },
}

impl TestResult {
    /// Returns `true` for a passing result.
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    /// Returns `true` for a failing result.
    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail { .. })
    }

    /// Returns `true` for a skipped result.
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skipped { .. })
    }
}

/// Trait all conformance tests implement.
pub trait ConformanceTest: Send + Sync {
    /// Returns the test's name within its crate.
    fn name(&self) -> &str;
    /// Returns the crate or subsystem covered by the test.
    fn crate_name(&self) -> &str;
    /// Returns the test's execution category.
    fn category(&self) -> TestCategory;
    /// Executes the test using the supplied context.
    fn run(&self, ctx: &mut TestContext) -> TestResult;

    /// Builds the test identifier as `<crate>::<name>`.
    fn id(&self) -> String {
        format!("{}::{}", self.crate_name(), self.name())
    }
}
