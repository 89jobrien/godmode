//! TestRunner — parallel/sequential conformance test execution with reporting.

use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;

use rayon::prelude::*;
use serde::Serialize;

use super::context::TestContext;
use super::traits::{ConformanceTest, TestCategory, TestResult};

/// Per-test run result.
#[derive(Debug, Clone, Serialize)]
pub struct TestRunResult {
    pub id: String,
    pub name: String,
    pub crate_name: String,
    pub category: TestCategory,
    #[serde(flatten)]
    pub result: TestResult,
    pub duration_ms: u64,
}

/// Aggregated summary of a test run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub results: Vec<TestRunResult>,
}

impl TestSummary {
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    pub fn by_crate(&self) -> HashMap<&str, Vec<&TestRunResult>> {
        let mut grouped: HashMap<&str, Vec<&TestRunResult>> = HashMap::new();
        for r in &self.results {
            grouped.entry(r.crate_name.as_str()).or_default().push(r);
        }
        grouped
    }
}

/// Configuration for report output.
#[derive(Debug, Clone, Default)]
pub struct ReportConfig {
    pub verbose: bool,
    pub summary_only: bool,
    pub show_timing: bool,
}

/// Runner for conformance tests.
pub struct TestRunner {
    tests: Vec<Box<dyn ConformanceTest>>,
    crate_filter: Option<String>,
    category_filter: Option<TestCategory>,
    name_filter: Option<String>,
    parallel: bool,
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            crate_filter: None,
            category_filter: None,
            name_filter: None,
            parallel: true,
        }
    }

    pub fn add_test<T: ConformanceTest + 'static>(&mut self, test: T) {
        self.tests.push(Box::new(test));
    }

    pub fn add_tests<I, T>(&mut self, tests: I)
    where
        I: IntoIterator<Item = T>,
        T: ConformanceTest + 'static,
    {
        for test in tests {
            self.tests.push(Box::new(test));
        }
    }

    /// Add pre-boxed tests (from `all()` collectors that return `Vec<Box<dyn ConformanceTest>>`).
    pub fn add_boxed(&mut self, tests: Vec<Box<dyn ConformanceTest>>) {
        self.tests.extend(tests);
    }

    pub fn filter_crate(mut self, crate_name: &str) -> Self {
        self.crate_filter = Some(crate_name.to_string());
        self
    }

    pub fn filter_category(mut self, category: TestCategory) -> Self {
        self.category_filter = Some(category);
        self
    }

    pub fn filter_name(mut self, pattern: &str) -> Self {
        self.name_filter = Some(pattern.to_string());
        self
    }

    pub fn parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    pub fn filtered_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|t| self.passes_filters(t.as_ref()))
            .count()
    }

    fn passes_filters(&self, test: &dyn ConformanceTest) -> bool {
        if let Some(ref cf) = self.crate_filter {
            if test.crate_name() != cf {
                return false;
            }
        }
        if let Some(cat) = self.category_filter {
            if test.category() != cat {
                return false;
            }
        }
        if let Some(ref nf) = self.name_filter {
            if !test.name().contains(nf.as_str()) {
                return false;
            }
        }
        true
    }

    fn run_one(test: &dyn ConformanceTest) -> TestRunResult {
        let start = Instant::now();
        let mut ctx = TestContext::new()
            .with_test_name(test.name())
            .with_logger_test_name(&test.id());
        let result = test.run(&mut ctx);
        TestRunResult {
            id: test.id(),
            name: test.name().to_string(),
            crate_name: test.crate_name().to_string(),
            category: test.category(),
            result,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn aggregate(results: Vec<TestRunResult>, duration_ms: u64) -> TestSummary {
        let mut s = TestSummary {
            duration_ms,
            ..Default::default()
        };
        for r in &results {
            s.total += 1;
            match &r.result {
                TestResult::Pass => s.passed += 1,
                TestResult::Fail { .. } => s.failed += 1,
                TestResult::Skipped { .. } => s.skipped += 1,
            }
        }
        s.results = results;
        s
    }

    pub fn run(&self) -> TestSummary {
        let start = Instant::now();
        let filtered: Vec<_> = self
            .tests
            .iter()
            .filter(|t| self.passes_filters(t.as_ref()))
            .collect();

        let results: Vec<TestRunResult> = if self.parallel {
            filtered
                .par_iter()
                .map(|t| Self::run_one(t.as_ref()))
                .collect()
        } else {
            filtered.iter().map(|t| Self::run_one(t.as_ref())).collect()
        };

        Self::aggregate(results, start.elapsed().as_millis() as u64)
    }
}

/// Report generator.
pub struct ReportGenerator;

impl ReportGenerator {
    pub fn text<W: Write>(
        writer: &mut W,
        summary: &TestSummary,
        config: &ReportConfig,
    ) -> std::io::Result<()> {
        writeln!(writer, "═══════════════════════════════════════════════")?;
        writeln!(writer, "       GODMODE CONFORMANCE TEST RESULTS        ")?;
        writeln!(writer, "═══════════════════════════════════════════════")?;
        writeln!(writer)?;

        let by_crate = summary.by_crate();
        let mut crate_names: Vec<_> = by_crate.keys().copied().collect();
        crate_names.sort_unstable();

        for cn in crate_names {
            let results = &by_crate[cn];
            let pass = results.iter().filter(|r| r.result.is_pass()).count();
            let fail = results.iter().filter(|r| r.result.is_fail()).count();
            let skip = results.iter().filter(|r| r.result.is_skipped()).count();
            let icon = if fail > 0 { '✗' } else { '✓' };
            write!(
                writer,
                "{} {}: {} pass, {} fail, {} skip",
                icon, cn, pass, fail, skip
            )?;
            if config.show_timing {
                let ms: u64 = results.iter().map(|r| r.duration_ms).sum();
                write!(writer, " ({}ms)", ms)?;
            }
            writeln!(writer)?;

            if !config.summary_only && (config.verbose || fail > 0) {
                for r in results.iter() {
                    let (icon, msg) = match &r.result {
                        TestResult::Pass => ("  ✓", String::new()),
                        TestResult::Fail { reason } => ("  ✗", format!(" FAILED:\n{}", reason)),
                        TestResult::Skipped { reason } => {
                            ("  ○", format!(" (skipped: {})", reason))
                        }
                    };
                    if config.verbose || r.result.is_fail() {
                        write!(writer, "{} {}{}", icon, r.name, msg)?;
                        if config.show_timing {
                            write!(writer, " ({}ms)", r.duration_ms)?;
                        }
                        writeln!(writer)?;
                    }
                }
            }
        }

        writeln!(writer)?;
        writeln!(writer, "───────────────────────────────────────────────")?;
        write!(
            writer,
            "TOTAL: {} pass, {} fail, {} skip ({} tests)",
            summary.passed, summary.failed, summary.skipped, summary.total
        )?;
        if config.show_timing {
            write!(writer, " in {}ms", summary.duration_ms)?;
        }
        writeln!(writer)?;
        writeln!(writer)?;
        if summary.is_success() {
            writeln!(writer, "RESULT: PASSED")?;
        } else {
            writeln!(writer, "RESULT: FAILED")?;
        }
        Ok(())
    }

    pub fn json<W: Write>(writer: &mut W, summary: &TestSummary) -> std::io::Result<()> {
        let report = serde_json::json!({
            "report_version": "1.0",
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "summary": {
                "total": summary.total,
                "passed": summary.passed,
                "failed": summary.failed,
                "skipped": summary.skipped,
                "duration_ms": summary.duration_ms,
                "success": summary.is_success(),
            },
            "results": summary.results,
        });
        writeln!(writer, "{}", serde_json::to_string_pretty(&report).unwrap())
    }

    pub fn github_actions<W: Write>(writer: &mut W, summary: &TestSummary) -> std::io::Result<()> {
        for r in &summary.results {
            if let TestResult::Fail { reason } = &r.result {
                let first_line = reason.lines().next().unwrap_or(reason.as_str());
                writeln!(
                    writer,
                    "::error title=Conformance Failed::{}::{} - {}",
                    r.crate_name, r.name, first_line
                )?;
            }
        }
        if summary.is_success() {
            writeln!(
                writer,
                "::notice::Conformance passed: {}/{} in {}ms",
                summary.passed, summary.total, summary.duration_ms
            )?;
        } else {
            writeln!(
                writer,
                "::error::Conformance failed: {} passed, {} failed",
                summary.passed, summary.failed
            )?;
        }
        Ok(())
    }
}
