# Persisted State

Godmode has several repository-local state contracts at functional/session baseline `3589433`:

- task graph: `.ctx/godmode/tasks.yaml` (`crates/godmode-core/src/model.rs:149-156`)
- session step and summary traces: dated JSONL under `.ctx/godmode/sessions/` (`crates/godmode-core/src/session.rs:85-110`)
- wave state: `.ctx/godmode/wave-status.json` (`crates/godmode-core/src/wave.rs:40-80`)
- workflow state: `.ctx/godmode/workflow-<name>.json` (`crates/godmode-core/src/workflow.rs:121-169`)
- pipeline state: `.ctx/godmode/pipeline.yaml` (`crates/godmode-core/src/pipeline.rs:101-126,172-192`)

These Serde-backed files are compatibility boundaries. [[Session]] routes normal task-graph persistence through `TaskGraphStorePort`; wave, workflow, and pipeline modules own their respective state.
