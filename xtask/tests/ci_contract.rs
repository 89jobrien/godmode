//! Contract tests for the repository-wide CI task.

#[test]
fn ci_gate_contains_all_required_steps() {
    let source = include_str!("../src/main.rs");
    for required in [
        "--all-targets",
        "--all-features",
        "tests/conformance/plugin-structure.nu",
        "skill index --check",
        "agent index --check",
        "command generate --target claude --check",
        "release validate",
        "cargo deny check",
    ] {
        assert!(source.contains(required), "CI gate missing: {required}");
    }
}
