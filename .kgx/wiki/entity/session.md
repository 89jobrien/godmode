# Session

At functional/session baseline `3589433`, `Session::open` delegates loading to default `SessionPorts`, whose graph-store adapter calls `graph::load` and `graph::save` (`crates/godmode-core/src/session.rs:61-68,196-235`).

Typed mutation methods route task transitions through graph functions, validate configured run commands before start, obtain timestamps from `ClockPort`, append best-effort Crux steps, and auto-save (`crates/godmode-core/src/session.rs:250-369,414-425`). Explicit `save` and summary writes return their errors (`crates/godmode-core/src/session.rs:398-408`).

`run_task_action` opens one Session and routes add, start, complete, block, unblock, remove, clear, and template application through it (`crates/godmode-cli/src/commands/task.rs:35-165,270-298`).

The default trace adapter writes dated step and summary JSONL under `.ctx/godmode/sessions/` (`crates/godmode-core/src/session.rs:85-110`). See [[Persisted State]] and [[Integrations]].
