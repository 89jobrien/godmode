use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use crux_runtime::types::crux_value::Crux;
use crux_runtime::types::error::CruxErr;
use crux_runtime::types::id::CruxId;
use crux_runtime::types::step::Step;

use crate::graph;
use crate::model::TaskGraph;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub struct Session {
    inner: Crux<TaskGraph>,
    root: PathBuf,
}

impl Session {
    /// Load the task graph and open a new session envelope.
    pub fn start(agent: impl Into<String>, root: &Path) -> Result<Self> {
        let graph = graph::load(root)?;
        let inner = Crux {
            id: CruxId::new(),
            agent: agent.into(),
            value: Ok(graph),
            steps: vec![],
            children: vec![],
            started_at: Utc::now(),
            finished_at: None,
        };
        Ok(Self {
            inner,
            root: root.to_path_buf(),
        })
    }

    /// Append a step to the session trace.
    pub fn record(&mut self, step: Step) {
        self.inner.steps.push(step);
    }

    /// Finalise with the current graph state, write `.ctx/sessions/<id>.json`.
    pub fn finish(mut self) -> Result<PathBuf> {
        self.inner.finished_at = Some(Utc::now());
        self.write_session()
    }

    /// Finalise with a failure, write `.ctx/sessions/<id>.json`.
    pub fn fail(mut self, err: CruxErr) -> Result<PathBuf> {
        self.inner.finished_at = Some(Utc::now());
        self.inner.value = Err(err);
        self.write_session()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn write_session(self) -> Result<PathBuf> {
        let dir = self.root.join(".ctx").join("sessions");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", self.inner.id));
        let json = serde_json::to_string_pretty(&self.inner)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crux_runtime::types::step::{StepKind, StepStatus};
    use tempfile::TempDir;

    fn make_step(name: &str) -> Step {
        Step {
            name: name.to_string(),
            kind: StepKind::Plain,
            status: StepStatus::Ok,
            confidence: 1.0,
            started_at: Utc::now(),
            duration_ms: 0,
            input_hash: 0,
            content_hash: None,
            output: None,
            error: None,
            attempt: 1,
            events: vec![],
            metadata: Default::default(),
            findings: vec![],
        }
    }

    #[test]
    fn session_trace_finish_roundtrips_crux_task_graph() {
        let dir = TempDir::new().unwrap();
        let session = Session::start("test-agent", dir.path()).unwrap();
        let path = session.finish().unwrap();

        assert!(path.exists());
        let json = std::fs::read_to_string(&path).unwrap();
        let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
        assert_eq!(crux.agent, "test-agent");
        assert!(crux.value.is_ok());
        assert!(crux.finished_at.is_some());
    }

    #[test]
    fn session_trace_fail_writes_err_value() {
        let dir = TempDir::new().unwrap();
        let session = Session::start("test-agent", dir.path()).unwrap();
        let err = CruxErr::step_failed("some-step", "something went wrong");
        let path = session.fail(err).unwrap();

        assert!(path.exists());
        let json = std::fs::read_to_string(&path).unwrap();
        let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
        assert!(crux.value.is_err());
        assert!(crux.finished_at.is_some());
    }

    #[test]
    fn session_trace_record_appends_steps() {
        let dir = TempDir::new().unwrap();
        let mut session = Session::start("test-agent", dir.path()).unwrap();
        session.record(make_step("step-one"));
        session.record(make_step("step-two"));
        let path = session.finish().unwrap();

        let json = std::fs::read_to_string(&path).unwrap();
        let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
        assert_eq!(crux.steps.len(), 2);
        assert_eq!(crux.steps[0].name, "step-one");
        assert_eq!(crux.steps[1].name, "step-two");
    }
}
