use godmode_core::integrations::rx;

mod fake_bin;
use fake_bin::FakeBin;

// ---------------------------------------------------------------------------
// Shell-out tests using fake binaries
// ---------------------------------------------------------------------------

#[test]
fn direct_cmd_runs() {
    let fake = FakeBin::new("my-cmd").stdout("ok\n").build();
    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", fake.path_with()) };
    let status = rx::run_cmd("my-cmd");
    unsafe { std::env::set_var("PATH", &old_path) };
    assert!(status.unwrap().success());
}

#[test]
fn rx_prefix_delegates_to_rx() {
    let fake = FakeBin::new("rx").echo_argv().build();
    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", fake.path_with()) };
    // run_cmd captures status; we need stdout — run via Command directly to inspect argv.
    // But run_cmd doesn't return stdout, so verify via resolve_cmd + manual Command.
    let (prog, args) = rx::resolve_cmd("rx:my-script");
    let out = std::process::Command::new(&prog)
        .args(&args)
        .env("PATH", fake.path_with())
        .output()
        .unwrap();
    unsafe { std::env::set_var("PATH", &old_path) };
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("run"),
        "expected 'run' in argv, got: {stdout}"
    );
    assert!(
        stdout.contains("my-script"),
        "expected 'my-script' in argv, got: {stdout}"
    );
}

#[test]
fn nonzero_exit_propagated() {
    let fake = FakeBin::new("failing-cmd").exit_code(2).build();
    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", fake.path_with()) };
    let status = rx::run_cmd("failing-cmd");
    unsafe { std::env::set_var("PATH", &old_path) };
    let status = status.unwrap();
    assert!(!status.success());
    assert_eq!(status.code(), Some(2));
}

// ---------------------------------------------------------------------------
// Pure-logic tests
// ---------------------------------------------------------------------------

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
