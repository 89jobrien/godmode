# Trace events

Read this before changing session tracing or trace output.

`Session::start_task` / `Session::complete_task` append `crux_core::Step` JSONL to
`.ctx/godmode/sessions/YYYY-MM-DD.jsonl`. `Session::handoff` writes a `SessionSummary` record to
`.ctx/godmode/sessions/YYYY-MM-DD-summary.jsonl`. All trace writes are non-fatal (`let _ = ...`).
