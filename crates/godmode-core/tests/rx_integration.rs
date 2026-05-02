use godmode_core::integrations::rx;

#[test]
fn resolve_cmd_direct_single_word() {
    let (prog, args) = rx::resolve_cmd("cargo");
    assert_eq!(prog, "cargo");
    assert!(args.is_empty());
}

#[test]
fn resolve_cmd_direct_with_args() {
    let (prog, args) = rx::resolve_cmd("cargo nextest run");
    assert_eq!(prog, "cargo");
    assert_eq!(args, vec!["nextest", "run"]);
}

#[test]
fn resolve_cmd_rx_prefix_delegates() {
    let (prog, args) = rx::resolve_cmd("rx:my-script");
    assert_eq!(prog, "rx");
    assert_eq!(args, vec!["run", "my-script"]);
}

#[test]
fn resolve_cmd_rx_prefix_trims_whitespace() {
    let (prog, args) = rx::resolve_cmd("rx:  my-script  ");
    assert_eq!(prog, "rx");
    assert_eq!(args, vec!["run", "my-script"]);
}

// Smoke test: real shell-out, no fake bins needed.
#[test]
fn run_cmd_true_exits_zero() {
    let status = rx::run_cmd("true").unwrap();
    assert!(status.success());
}

#[test]
fn run_cmd_false_exits_nonzero() {
    let status = rx::run_cmd("false").unwrap();
    assert!(!status.success());
}
