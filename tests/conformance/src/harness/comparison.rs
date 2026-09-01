//! OutputComparator — diff generation for conformance assertions.

use similar::{ChangeTag, TextDiff};
use std::fmt::Debug;

/// Result of a comparison.
#[derive(Debug, Clone, PartialEq)]
pub enum CompareResult {
    /// The compared values are exactly equal.
    Equal,
    /// The numeric values differ by no more than the permitted tolerance.
    ApproximatelyEqual {
        /// Absolute difference between the values.
        delta: f64,
        /// Maximum permitted absolute difference.
        epsilon: f64,
    },
    /// The values differ, with details describing the mismatch.
    Different(Diff),
}

impl CompareResult {
    /// Returns whether the comparison represents a mismatch.
    pub fn is_fail(&self) -> bool {
        matches!(self, CompareResult::Different(_))
    }
}

/// Detailed diff between expected and actual.
#[derive(Debug, Clone, PartialEq)]
pub struct Diff {
    /// Expected value rendered for diagnostics.
    pub expected: String,
    /// Actual value rendered for diagnostics.
    pub actual: String,
    /// Line-oriented unified diff between the rendered values.
    pub unified_diff: String,
}

impl Diff {
    /// Renders a human-readable description of the mismatch.
    pub fn describe(&self) -> String {
        if self.unified_diff.is_empty() {
            format!("expected {:?}, got {:?}", self.expected, self.actual)
        } else {
            format!("diff:\n{}", self.unified_diff)
        }
    }
}

/// Comparator for test outputs.
pub struct OutputComparator;

impl OutputComparator {
    /// Creates a stateless output comparator.
    pub fn new() -> Self {
        Self
    }

    /// Compares values using equality and their debug representations.
    pub fn compare_debug<T: PartialEq + Debug>(&self, expected: &T, actual: &T) -> CompareResult {
        if expected == actual {
            CompareResult::Equal
        } else {
            let exp_s = format!("{:?}", expected);
            let act_s = format!("{:?}", actual);
            CompareResult::Different(Diff {
                unified_diff: unified_diff(&exp_s, &act_s),
                expected: exp_s,
                actual: act_s,
            })
        }
    }

    /// Compares strings and produces a line-oriented diff on mismatch.
    pub fn compare_str(&self, expected: &str, actual: &str) -> CompareResult {
        if expected == actual {
            CompareResult::Equal
        } else {
            CompareResult::Different(Diff {
                unified_diff: unified_diff(expected, actual),
                expected: expected.to_string(),
                actual: actual.to_string(),
            })
        }
    }

    /// Compares floating-point values using an absolute epsilon tolerance.
    pub fn compare_f64(&self, expected: f64, actual: f64, epsilon: f64) -> CompareResult {
        let delta = (expected - actual).abs();
        if delta <= epsilon {
            CompareResult::ApproximatelyEqual { delta, epsilon }
        } else {
            CompareResult::Different(Diff {
                expected: expected.to_string(),
                actual: actual.to_string(),
                unified_diff: format!("delta {} exceeds epsilon {}", delta, epsilon),
            })
        }
    }
}

impl Default for OutputComparator {
    fn default() -> Self {
        Self::new()
    }
}

fn unified_diff(expected: &str, actual: &str) -> String {
    let diff = TextDiff::from_lines(expected, actual);
    let mut out = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        out.push_str(&format!("{}{}", sign, change));
    }
    out
}
