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
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail { .. })
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skipped { .. })
    }
}

/// Trait all conformance tests implement.
pub trait ConformanceTest: Send + Sync {
    fn name(&self) -> &str;
    fn crate_name(&self) -> &str;
    fn category(&self) -> TestCategory;
    fn run(&self, ctx: &mut TestContext) -> TestResult;

    fn id(&self) -> String {
        format!("{}::{}", self.crate_name(), self.name())
    }
}
