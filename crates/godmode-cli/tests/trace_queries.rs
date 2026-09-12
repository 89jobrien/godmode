//! CLI integration tests for built-in observability trace queries.

use std::process::Command;

fn godmode_bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_godmode")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/godmode")
        })
}

fn query(root: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let result = Command::new(godmode_bin())
        .arg("--json")
        .arg("trace")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn trace_queries_are_built_in_and_ignore_legacy_rows() {
    let temp = tempfile::tempdir().unwrap();
    let trace_dir = temp.path().join(".ctx/godmode/traces");
    std::fs::create_dir_all(&trace_dir).unwrap();
    std::fs::write(
        trace_dir.join("trace.jsonl"),
        concat!(
            "{\"kind\":\"started\",\"task_id\":\"legacy\"}\n",
            "{not-json}\n",
            "{\"event\":42}\n",
            "{\"event\":\"session.start\",\"session_id\":\"s1\",\"ts\":\"2026-09-07T10:00:00Z\"}\n",
            "{\"event\":\"skill.complete\",\"session_id\":\"s1\",\"skill\":\"verify\",\"duration_ms\":10,\"ts\":\"2026-09-07T10:00:01Z\"}\n",
            "{\"event\":\"skill.complete\",\"session_id\":\"s1\",\"skill\":\"verify\",\"duration_ms\":20,\"ts\":\"2026-09-07T10:00:02Z\"}\n",
            "{\"event\":\"agent.start\",\"session_id\":\"s1\",\"agent_id\":\"a1\",\"slot\":1,\"ts\":\"2026-09-07T10:00:03Z\"}\n",
            "{\"event\":\"agent.complete\",\"session_id\":\"s1\",\"agent_id\":\"a1\",\"slot\":1,\"ts\":\"2026-09-07T10:00:04Z\"}\n",
            "{\"event\":\"agent.start\",\"session_id\":\"s1\",\"agent_id\":\"a2\",\"slot\":2,\"ts\":\"2026-09-07T10:00:04Z\"}\n",
            "{\"event\":\"agent.blocked\",\"session_id\":\"s1\",\"agent_id\":\"a2\",\"slot\":2,\"ts\":\"2026-09-07T10:00:04Z\"}\n",
            "{\"event\":\"agent.start\",\"session_id\":\"s1\",\"agent_id\":\"a2\",\"slot\":2,\"ts\":\"2026-09-07T10:00:04Z\"}\n",
            "{\"event\":\"agent.denied\",\"session_id\":\"s1\",\"agent_id\":\"policy-only\",\"ts\":\"2026-09-07T10:00:04Z\"}\n",
            "{\"event\":\"decision\",\"session_id\":\"s1\",\"skill\":\"verify\",\"helper\":\"gate\",\"kind\":\"result\",\"value\":\"pass\",\"ts\":\"2026-09-07T10:00:05Z\"}\n",
            "{\"event\":\"skill.error\",\"session_id\":\"s1\",\"skill\":\"audit\",\"exit_code\":1,\"ts\":\"2026-09-07T10:00:06Z\"}\n",
            "{\"event\":\"session.end\",\"session_id\":\"s1\",\"ts\":\"2026-09-07T10:00:07Z\"}\n",
        ),
    )
    .unwrap();
    std::fs::write(
        temp.path().join(".ctx/godmode/session.json"),
        "{\"session_id\":\"s1\"}",
    )
    .unwrap();

    let stats = query(temp.path(), &["stats", "--current"]);
    assert_eq!(stats["legacy_rows"], 1);
    assert_eq!(stats["malformed_rows"], 2);
    assert_eq!(stats["skills"][0]["runs"], 2);
    assert_eq!(stats["skills"][0]["avg_ms"], 15);
    assert_eq!(stats["agents"][0]["status"], "complete");
    assert_eq!(stats["agents"][1]["status"], "running");
    assert_eq!(stats["decisions"].as_array().unwrap().len(), 1);

    let failures = query(temp.path(), &["failures", "--session", "s1"]);
    assert_eq!(failures.as_array().unwrap().len(), 3);
    assert_eq!(failures[2]["event"], "skill.error");

    let tail = query(temp.path(), &["tail", "--n", "2", "--session", "s1"]);
    assert_eq!(tail.as_array().unwrap().len(), 2);
    assert_eq!(tail[1]["event"], "session.end");

    let summary = query(temp.path(), &["summary", "--sessions", "1"]);
    assert_eq!(summary[0]["session_id"], "s1");
    assert_eq!(summary[0]["errors"], 1);
    assert_eq!(summary[0]["agents_complete"], 1);
    assert_eq!(summary[0]["agents_blocked"], 0);
    assert_eq!(summary[0]["agents_denied"], 1);
    assert_eq!(summary[0]["agents_running"], 1);
}

#[test]
fn trace_summary_can_select_the_previous_session() {
    let temp = tempfile::tempdir().unwrap();
    let trace_dir = temp.path().join(".ctx/godmode/traces");
    std::fs::create_dir_all(&trace_dir).unwrap();
    std::fs::write(
        trace_dir.join("trace.jsonl"),
        concat!(
            "{\"event\":\"session.start\",\"session_id\":\"previous\",\"ts\":\"2026-09-07T09:00:00Z\"}\n",
            "{\"event\":\"session.end\",\"session_id\":\"previous\",\"ts\":\"2026-09-07T09:30:00Z\"}\n",
            "{\"event\":\"session.start\",\"session_id\":\"current\",\"ts\":\"2026-09-07T10:00:00Z\"}\n",
        ),
    )
    .unwrap();
    std::fs::write(
        temp.path().join(".ctx/godmode/session.json"),
        "{\"session_id\":\"current\"}",
    )
    .unwrap();

    let summary = query(temp.path(), &["summary", "--sessions", "1", "--previous"]);
    assert_eq!(summary[0]["session_id"], "previous");
}

#[test]
fn trace_queries_handle_missing_files_and_reject_invalid_limits() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(
        query(temp.path(), &["tail"]),
        serde_json::Value::Array(vec![])
    );
    assert_eq!(
        query(temp.path(), &["failures"]),
        serde_json::Value::Array(vec![])
    );
    assert_eq!(
        query(temp.path(), &["summary"]),
        serde_json::Value::Array(vec![])
    );

    let result = Command::new(godmode_bin())
        .args(["trace", "summary", "--sessions", "0"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!result.status.success());

    let missing_current = Command::new(godmode_bin())
        .args(["trace", "stats", "--current"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!missing_current.status.success());
}

#[test]
fn current_trace_query_reports_malformed_session_state_context() {
    let temp = tempfile::tempdir().unwrap();
    let session_path = temp.path().join(".ctx/godmode/session.json");
    std::fs::create_dir_all(session_path.parent().unwrap()).unwrap();
    std::fs::write(&session_path, "{not-json}").unwrap();

    let result = Command::new(godmode_bin())
        .args(["trace", "stats", "--current"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("parsing"), "{stderr}");
    assert!(stderr.contains(".ctx/godmode/"), "{stderr}");
    assert!(stderr.contains("session.json"), "{stderr}");
}

#[test]
fn current_trace_query_reports_unreadable_session_state_context() {
    let temp = tempfile::tempdir().unwrap();
    let session_path = temp.path().join(".ctx/godmode/session.json");
    std::fs::create_dir_all(&session_path).unwrap();

    let result = Command::new(godmode_bin())
        .args(["trace", "stats", "--current"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("reading"), "{stderr}");
    assert!(stderr.contains(".ctx/godmode/"), "{stderr}");
    assert!(stderr.contains("session.json"), "{stderr}");
}

#[test]
fn trace_tail_zero_matches_core_empty_limit_invariant() {
    let temp = tempfile::tempdir().unwrap();
    let trace = temp.path().join(".ctx/godmode/traces");
    std::fs::create_dir_all(&trace).unwrap();
    std::fs::write(
        trace.join("trace.jsonl"),
        "{\"event\":\"session.start\",\"session_id\":\"s1\",\"ts\":\"now\"}\n",
    )
    .unwrap();
    assert_eq!(
        query(temp.path(), &["tail", "--n", "0"]),
        serde_json::json!([])
    );
}

#[test]
fn trace_queries_add_operation_context_to_io_errors() {
    let temp = tempfile::tempdir().unwrap();
    let trace = temp.path().join(".ctx/godmode/traces/trace.jsonl");
    std::fs::create_dir_all(&trace).unwrap();
    let output = Command::new(godmode_bin())
        .args(["trace", "tail"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("querying trace tail"));
    assert!(stderr.contains("trace.jsonl"));
}

#[test]
fn trace_human_output_labels_events_failures_stats_and_summaries() {
    let temp = tempfile::tempdir().unwrap();
    let trace = temp.path().join(".ctx/godmode/traces");
    std::fs::create_dir_all(&trace).unwrap();
    std::fs::write(trace.join("trace.jsonl"), concat!("{\"event\":\"session.start\",\"session_id\":\"s1\",\"ts\":\"start\"}\n","{\"event\":\"skill.error\",\"session_id\":\"s1\",\"skill\":\"verify\",\"exit_code\":1,\"ts\":\"end\"}\n","{\"event\":\"session.end\",\"session_id\":\"s1\",\"ts\":\"end\"}\n")).unwrap();
    for (args, expected) in [
        (&["trace", "tail"][..], "session.start"),
        (&["trace", "failures"][..], "skill.error"),
        (&["trace", "stats"][..], "Failures: 1"),
        (&["trace", "summary"][..], "s1"),
    ] {
        let output = Command::new(godmode_bin())
            .args(args)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "missing {expected}"
        );
    }
}
