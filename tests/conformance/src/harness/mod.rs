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
