//! Conformance tests for [`ReportIndexPort`].
//!
//! Any adapter that implements `ReportIndexPort` must pass every case in
//! `report_index_suite()`. To add a new adapter, call the suite in a new `#[test]`.

use godmode_core::report_index::{JsonFileIndex, ReportIndexPort};
use tempfile::TempDir;

// ── Minimal inline conformance harness ───────────────────────────────────────
// Mirrors godmode_core::testing::conformance, which is feature-gated.

trait ConformanceCase {
    fn name(&self) -> &str;
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String>;
}

struct ConformanceSuite {
    cases: Vec<Box<dyn ConformanceCase>>,
}

impl ConformanceSuite {
    fn new() -> Self {
        Self { cases: Vec::new() }
    }
    fn case(mut self, c: Box<dyn ConformanceCase>) -> Self {
        self.cases.push(c);
        self
    }
    fn assert_all(&self, subject: &dyn ReportIndexPort) {
        let mut failures = Vec::new();
        for case in &self.cases {
            if let Err(reason) = case.run(subject) {
                failures.push(format!("  FAIL [{}]: {}", case.name(), reason));
            }
        }
        if !failures.is_empty() {
            panic!("Conformance failures:\n{}", failures.join("\n"));
        }
    }
}

// ── Contract cases ────────────────────────────────────────────────────────────

struct LoadReturnsOk;
impl ConformanceCase for LoadReturnsOk {
    fn name(&self) -> &str {
        "load_returns_ok"
    }
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String> {
        subject.load().map(|_| ()).map_err(|e| e.to_string())
    }
}

struct AddEntryAppearsInLoad;
impl ConformanceCase for AddEntryAppearsInLoad {
    fn name(&self) -> &str {
        "add_entry_appears_in_load"
    }
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String> {
        subject
            .add_entry("reflect", "report.md")
            .map_err(|e| e.to_string())?;
        let index = subject.load().map_err(|e| e.to_string())?;
        let cat = index
            .categories
            .get("reflect")
            .ok_or_else(|| "category 'reflect' missing after add_entry".to_string())?;
        if cat.files.contains(&"report.md".to_string()) {
            Ok(())
        } else {
            Err(format!("'report.md' not in files: {:?}", cat.files))
        }
    }
}

struct AddEntryIsIdempotent;
impl ConformanceCase for AddEntryIsIdempotent {
    fn name(&self) -> &str {
        "add_entry_is_idempotent"
    }
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String> {
        subject
            .add_entry("reflect", "dup.md")
            .map_err(|e| e.to_string())?;
        subject
            .add_entry("reflect", "dup.md")
            .map_err(|e| e.to_string())?;
        let index = subject.load().map_err(|e| e.to_string())?;
        let cat = index
            .categories
            .get("reflect")
            .ok_or_else(|| "category 'reflect' missing".to_string())?;
        let count = cat.files.iter().filter(|f| f.as_str() == "dup.md").count();
        if count == 1 {
            Ok(())
        } else {
            Err(format!(
                "expected exactly 1 occurrence of 'dup.md', got {count}"
            ))
        }
    }
}

struct AddItemAppearsInLoad;
impl ConformanceCase for AddItemAppearsInLoad {
    fn name(&self) -> &str {
        "add_item_appears_in_load"
    }
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String> {
        subject
            .add_item("insights", "sub/detail.md")
            .map_err(|e| e.to_string())?;
        let index = subject.load().map_err(|e| e.to_string())?;
        let cat = index
            .categories
            .get("insights")
            .ok_or_else(|| "category 'insights' missing after add_item".to_string())?;
        if cat.items.contains(&"sub/detail.md".to_string()) {
            Ok(())
        } else {
            Err(format!("'sub/detail.md' not in items: {:?}", cat.items))
        }
    }
}

struct FilesAreSorted;
impl ConformanceCase for FilesAreSorted {
    fn name(&self) -> &str {
        "files_are_sorted"
    }
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String> {
        subject
            .add_entry("reflect", "z-last.md")
            .map_err(|e| e.to_string())?;
        subject
            .add_entry("reflect", "a-first.md")
            .map_err(|e| e.to_string())?;
        let index = subject.load().map_err(|e| e.to_string())?;
        let cat = index
            .categories
            .get("reflect")
            .ok_or_else(|| "category 'reflect' missing".to_string())?;
        let mut sorted = cat.files.clone();
        sorted.sort();
        if cat.files == sorted {
            Ok(())
        } else {
            Err(format!("files not sorted: {:?}", cat.files))
        }
    }
}

struct RebuildReturnsOk;
impl ConformanceCase for RebuildReturnsOk {
    fn name(&self) -> &str {
        "rebuild_returns_ok"
    }
    fn run(&self, subject: &dyn ReportIndexPort) -> Result<(), String> {
        subject.rebuild().map(|_| ()).map_err(|e| e.to_string())
    }
}

// ── Suite ─────────────────────────────────────────────────────────────────────

fn report_index_suite() -> ConformanceSuite {
    ConformanceSuite::new()
        .case(Box::new(LoadReturnsOk))
        .case(Box::new(AddEntryAppearsInLoad))
        .case(Box::new(AddEntryIsIdempotent))
        .case(Box::new(AddItemAppearsInLoad))
        .case(Box::new(FilesAreSorted))
        .case(Box::new(RebuildReturnsOk))
}

// ── Adapter tests ─────────────────────────────────────────────────────────────

#[test]
fn json_file_index_satisfies_report_index_port() {
    let dir = TempDir::new().unwrap();
    let adapter = JsonFileIndex::new(dir.path());
    report_index_suite().assert_all(&adapter);
}
