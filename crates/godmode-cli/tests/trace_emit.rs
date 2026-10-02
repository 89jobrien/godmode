//! Integration coverage for `godmode trace emit`.
//!
//! The emit subcommands are what the Nushell shim in `skills/_lib/trace.nu`
//! calls, so these tests exercise the real argv boundary rather than the Rust
//! functions behind it. That boundary is where argument handling broke once:
//! a helper invoked as `--json` could not be recorded, because clap read
//! `--json` as a flag rather than as the value of `--args`.

use std::path::Path;
use std::process::{Command, Output};

fn godmode_bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_godmode") {
        return std::path::PathBuf::from(path);
    }
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("target/debug/godmode")
}

fn run_godmode(dir: &Path, args: &[&str]) -> Output {
    Command::new(godmode_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run godmode")
}

/// A throwaway git repo, since `godmode` resolves the trace log from the
/// repository root. `name` lands in the initial commit so a failing scratch
/// repo is identifiable.
fn scratch(name: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .expect("git init");
    assert!(init.success(), "git init must succeed for the scratch repo");
    let commit = Command::new("git")
        .args(["commit", "-q", "--allow-empty", "-m", name])
        .current_dir(dir.path())
        .status()
        .expect("git commit");
    assert!(
        commit.success(),
        "git commit must succeed for the scratch repo"
    );
    dir
}

fn trace_body(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(".ctx/godmode/traces/trace.jsonl"))
        .expect("trace log must exist after an emit")
}

#[test]
fn skill_start_accepts_an_argument_that_looks_like_a_flag() {
    // Regression: `--args --json` failed with "a value is required for
    // '--args' but none was supplied", so the shim recorded nothing.
    let dir = scratch("hyphen-args");
    let out = run_godmode(
        dir.path(),
        &[
            "trace",
            "emit",
            "skill-start",
            "--skill",
            "demo",
            "--helper",
            "helpers/demo.nu",
            "--args",
            "--json",
            "--args",
            "-x",
        ],
    );
    assert!(
        out.status.success(),
        "emit must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(trace_body(dir.path()).contains("demo"));
}

#[test]
fn skill_start_prints_a_trace_id_the_close_call_accepts() {
    // The whole lifecycle depends on the printed id parsing back to its skill.
    let dir = scratch("round-trip");
    let start = run_godmode(
        dir.path(),
        &[
            "trace",
            "emit",
            "skill-start",
            "--skill",
            "demo",
            "--helper",
            "h.nu",
        ],
    );
    assert!(start.status.success());
    let tid = String::from_utf8_lossy(&start.stdout).trim().to_string();
    assert!(!tid.is_empty(), "skill-start must print a trace id");
    assert!(
        tid.starts_with("demo#"),
        "id must lead with the skill: {tid}"
    );

    let close = run_godmode(
        dir.path(),
        &["trace", "emit", "skill-complete", "--trace-id", &tid],
    );
    assert!(
        close.status.success(),
        "closing with a minted id must succeed: {}",
        String::from_utf8_lossy(&close.stderr)
    );
    assert!(trace_body(dir.path()).contains(&tid));
}

#[test]
fn skill_error_records_the_exit_code_and_stderr_tail() {
    let dir = scratch("error");
    let out = run_godmode(
        dir.path(),
        &[
            "trace",
            "emit",
            "skill-error",
            "--trace-id",
            "demo#12345.0#h",
            "--exit-code",
            "7",
            "--stderr-tail",
            "boom",
        ],
    );
    assert!(out.status.success());
    let body = trace_body(dir.path());
    assert!(body.contains("boom"), "stderr tail must be recorded");
    assert!(
        body.contains("\"exit_code\":7"),
        "exit code must be recorded"
    );
}

#[test]
fn agent_start_records_the_slot_and_crate() {
    let dir = scratch("slot");
    let out = run_godmode(
        dir.path(),
        &[
            "trace",
            "emit",
            "agent-start",
            "--agent-id",
            "explore",
            "--slot",
            "1",
            "--crate",
            "godmode-core",
        ],
    );
    assert!(out.status.success());
    let body = trace_body(dir.path());
    assert!(body.contains("explore"));
    assert!(body.contains("\"slot\":\"1\""), "slot must be recorded");
    assert!(body.contains("godmode-core"), "crate must be recorded");
}

#[test]
fn a_hyphen_leading_value_is_rejected_for_an_identifier_flag() {
    // Deliberate, and pinned so it cannot change by accident. Only `--args`
    // and `--commits` allow hyphen values, because they carry free-form caller
    // input. Identifier flags do not: allowing hyphens everywhere would let a
    // missing value silently swallow the next flag, so `--slot --crate x` would
    // set slot to "--crate" instead of erroring.
    let dir = scratch("hyphen-slot");
    let out = run_godmode(
        dir.path(),
        &[
            "trace",
            "emit",
            "agent-start",
            "--agent-id",
            "a",
            "--slot",
            "-1",
        ],
    );
    assert!(
        !out.status.success(),
        "a hyphen-leading slot must fail loudly rather than be swallowed"
    );
}

#[test]
fn an_unparseable_trace_id_still_closes_without_failing() {
    // A caller may hand back anything. Tracing must not be able to fail the
    // helper that made the call, so the record is written with a fallback skill.
    let dir = scratch("junk-id");
    let out = run_godmode(
        dir.path(),
        &["trace", "emit", "skill-complete", "--trace-id", "garbage"],
    );
    assert!(
        out.status.success(),
        "a junk id must not fail the caller: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(trace_body(dir.path()).contains("unknown"));
}

#[test]
fn emitted_events_are_then_queryable_in_the_same_process_sequence() {
    // The emit and query paths share a schema; this is the seam that a
    // divergence in either would break.
    let dir = scratch("seam");
    for args in [
        vec![
            "trace",
            "emit",
            "agent-start",
            "--agent-id",
            "explore",
            "--slot",
            "1",
        ],
        vec![
            "trace",
            "emit",
            "agent-start",
            "--agent-id",
            "explore",
            "--slot",
            "1",
        ],
    ] {
        assert!(run_godmode(dir.path(), &args).status.success());
    }

    let out = run_godmode(dir.path(), &["trace", "--json", "summary"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"explore\""),
        "the unresolved agent must surface in summary: {stdout}"
    );
}
