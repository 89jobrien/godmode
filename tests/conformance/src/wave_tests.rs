//! Conformance tests for godmode_core::wave — parallel agent slot state.

use godmode_core::wave;

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

/// Verifies that wave initialization creates pending slots for each agent.
pub struct WaveInitCreatesSlots;
impl ConformanceTest for WaveInitCreatesSlots {
    fn name(&self) -> &str {
        "wave_init_creates_slots"
    }
    fn crate_name(&self) -> &str {
        "wave"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let state = wave::init(dir.path(), 1, &["agent-a", "agent-b"]).unwrap();
        ctx.assert_eq(&1u32, &state.wave);
        ctx.assert_eq(&2usize, &state.agents.len());
        ctx.assert_eq(&wave::SlotStatus::Pending, &state.agents["agent-a"].status);
        ctx.result()
    }
}

/// Verifies that persisted wave state can be loaded without data loss.
pub struct WaveRoundtrip;
impl ConformanceTest for WaveRoundtrip {
    fn name(&self) -> &str {
        "wave_roundtrip"
    }
    fn crate_name(&self) -> &str {
        "wave"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        wave::init(dir.path(), 2, &["slot-1"]).unwrap();
        let loaded = wave::load(dir.path()).unwrap();
        ctx.assert_eq(&2u32, &loaded.wave);
        ctx.assert_eq(&1usize, &loaded.agents.len());
        ctx.result()
    }
}

/// Verifies that marking a slot done records its status and commits.
pub struct WaveMarkDone;
impl ConformanceTest for WaveMarkDone {
    fn name(&self) -> &str {
        "wave_mark_done"
    }
    fn crate_name(&self) -> &str {
        "wave"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        wave::init(dir.path(), 1, &["a", "b"]).unwrap();
        wave::mark_done(dir.path(), "a", vec!["abc123".into()]).unwrap();
        let state = wave::load(dir.path()).unwrap();
        ctx.assert_eq(&wave::SlotStatus::Done, &state.agents["a"].status);
        ctx.assert_eq(&vec!["abc123".to_string()], &state.agents["a"].commits);
        ctx.assert_eq(&wave::SlotStatus::Pending, &state.agents["b"].status);
        ctx.result()
    }
}

/// Verifies that an agent slot can be marked blocked.
pub struct WaveMarkBlocked;
impl ConformanceTest for WaveMarkBlocked {
    fn name(&self) -> &str {
        "wave_mark_blocked"
    }
    fn crate_name(&self) -> &str {
        "wave"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        wave::init(dir.path(), 1, &["a"]).unwrap();
        wave::mark_blocked(dir.path(), "a").unwrap();
        let state = wave::load(dir.path()).unwrap();
        ctx.assert_eq(&wave::SlotStatus::Blocked, &state.agents["a"].status);
        ctx.result()
    }
}

/// Verifies that all-done detection requires every slot to be complete.
pub struct WaveCheckAllDone;
impl ConformanceTest for WaveCheckAllDone {
    fn name(&self) -> &str {
        "wave_check_all_done"
    }
    fn crate_name(&self) -> &str {
        "wave"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        wave::init(dir.path(), 1, &["a", "b"]).unwrap();
        let state = wave::load(dir.path()).unwrap();
        // not done yet — one pending
        if wave::all_done(&state) {
            ctx.fail("all_done should be false when slots are pending");
        }
        wave::mark_done(dir.path(), "a", vec![]).unwrap();
        wave::mark_done(dir.path(), "b", vec![]).unwrap();
        let state = wave::load(dir.path()).unwrap();
        if !wave::all_done(&state) {
            ctx.fail("all_done should be true when all slots are done");
        }
        ctx.result()
    }
}

/// Verifies wave completion checks when a slot is blocked and none are pending.
pub struct WaveCheckHasBlocked;
impl ConformanceTest for WaveCheckHasBlocked {
    fn name(&self) -> &str {
        "wave_check_has_blocked"
    }
    fn crate_name(&self) -> &str {
        "wave"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        wave::init(dir.path(), 1, &["a", "b"]).unwrap();
        wave::mark_blocked(dir.path(), "a").unwrap();
        // check() returns true when all are done-or-blocked (i.e. no pending)
        // mark b done too
        wave::mark_done(dir.path(), "b", vec![]).unwrap();
        let state = wave::load(dir.path()).unwrap();
        if !wave::check(&state) {
            ctx.fail("check() should return true when no slots are pending");
        }
        if wave::all_done(&state) {
            ctx.fail("all_done should be false when a slot is blocked");
        }
        ctx.result()
    }
}

/// Returns all wave state conformance tests.
pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(WaveInitCreatesSlots),
        Box::new(WaveRoundtrip),
        Box::new(WaveMarkDone),
        Box::new(WaveMarkBlocked),
        Box::new(WaveCheckAllDone),
        Box::new(WaveCheckHasBlocked),
    ]
}
