//! TestContext — unified interface for conformance test execution.

use std::fmt::Debug;

use super::comparison::OutputComparator;
use super::logging::TestLogger;
use super::traits::TestResult;

pub struct TestContext {
    logger: TestLogger,
    comparator: OutputComparator,
    has_failures: bool,
    test_name: String,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    /// Creates an empty test context with no recorded failures.
    pub fn new() -> Self {
        Self {
            logger: TestLogger::new(),
            comparator: OutputComparator::new(),
            has_failures: false,
            test_name: String::new(),
        }
    }

    /// Sets the test name on both the context and its logger.
    pub fn with_test_name(mut self, name: &str) -> Self {
        self.test_name = name.to_string();
        self.logger.set_test_name(name);
        self
    }

    /// Sets the name displayed by the test logger.
    pub fn with_logger_test_name(mut self, name: &str) -> Self {
        self.logger.set_test_name(name);
        self
    }

    /// Logs a named input using its debug representation.
    pub fn log_input<T: Debug>(&mut self, name: &str, value: &T) {
        self.logger.log_input(name, value);
    }

    /// Logs a named expected value using its debug representation.
    pub fn log_expected<T: Debug>(&mut self, name: &str, value: &T) {
        self.logger.log_expected(name, value);
    }

    /// Logs a named actual value using its debug representation.
    pub fn log_actual<T: Debug>(&mut self, name: &str, value: &T) {
        self.logger.log_actual(name, value);
    }

    /// Compares debug-printable values and records a failure when they differ.
    pub fn assert_eq<T: PartialEq + Debug>(&mut self, expected: &T, actual: &T) -> bool {
        let result = self.comparator.compare_debug(expected, actual);
        if result.is_fail() {
            self.has_failures = true;
            if let super::comparison::CompareResult::Different(ref diff) = result {
                self.logger.error(&format!("FAIL: {}", diff.describe()));
            }
            false
        } else {
            true
        }
    }

    /// Compares strings and records a diff when they differ.
    pub fn assert_str_eq(&mut self, expected: &str, actual: &str) -> bool {
        let result = self.comparator.compare_str(expected, actual);
        if result.is_fail() {
            self.has_failures = true;
            if let super::comparison::CompareResult::Different(ref diff) = result {
                self.logger.error(&format!("FAIL: {}", diff.describe()));
            }
            false
        } else {
            true
        }
    }

    /// Compares floating-point values and records a failure outside epsilon.
    pub fn assert_f64_eq(&mut self, expected: f64, actual: f64, epsilon: f64) -> bool {
        let result = self.comparator.compare_f64(expected, actual, epsilon);
        if result.is_fail() {
            self.has_failures = true;
            if let super::comparison::CompareResult::Different(ref diff) = result {
                self.logger.error(&format!("FAIL: {}", diff.describe()));
            }
            false
        } else {
            true
        }
    }

    /// Records an explicit test failure with the supplied reason.
    pub fn fail(&mut self, reason: &str) {
        self.has_failures = true;
        self.logger.error(&format!("FAIL: {}", reason));
    }

    /// Creates a skipped test result with the supplied reason.
    pub fn skip(reason: &str) -> TestResult {
        TestResult::Skipped {
            reason: reason.to_string(),
        }
    }

    /// Logs a section heading, then invokes the callback with this context.
    pub fn section<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.logger.section(name, |_| {});
        f(self);
    }

    /// Returns failure output when any assertion failed; otherwise returns pass.
    pub fn result(&self) -> TestResult {
        if self.has_failures {
            TestResult::Fail {
                reason: self.logger.render(),
            }
        } else {
            TestResult::Pass
        }
    }
}
