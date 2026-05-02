use godmode_core::integrations::hj;

mod fake_bin;
use fake_bin::FakeBin;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal temp dir with a Cargo.toml so `detect::package_name` works.
fn make_fake_root(name: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
    )
    .unwrap();
    dir
}

// ---------------------------------------------------------------------------
// Shell-out tests
// ---------------------------------------------------------------------------

#[test]
fn handon_returns_stdout() {
    let root = make_fake_root("test-project");
    let fake = FakeBin::new("hj").stdout("handon output\n").build();
    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", fake.path_with()) };
    let result = hj::handon(root.path());
    unsafe { std::env::set_var("PATH", &old_path) };
    assert_eq!(result.unwrap(), "handon output\n");
}

#[test]
fn handoff_passes_args() {
    let root = make_fake_root("test-project");
    let fake = FakeBin::new("hj").echo_argv().build();
    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", fake.path_with()) };
    let result = hj::handoff(
        root.path(),
        "passing",
        "19 passed",
        "feat: add tests",
        &["abc123"],
    );
    unsafe { std::env::set_var("PATH", &old_path) };
    let stdout = result.unwrap();
    // fake hj echoed argv as JSON — check required flags are present
    assert!(stdout.contains("--build"), "missing --build in: {stdout}");
    assert!(stdout.contains("--tests"), "missing --tests in: {stdout}");
    assert!(
        stdout.contains("--log-summary"),
        "missing --log-summary in: {stdout}"
    );
    assert!(stdout.contains("--commit"), "missing --commit in: {stdout}");
    assert!(stdout.contains("abc123"), "missing commit sha in: {stdout}");
}

#[test]
fn handon_errors_when_hj_missing() {
    let root = make_fake_root("test-project");
    // Use an empty temp dir — no hj binary present.
    let empty = tempfile::TempDir::new().unwrap();
    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", empty.path().to_str().unwrap()) };
    let result = hj::handon(root.path());
    unsafe { std::env::set_var("PATH", &old_path) };
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("hj not found") || msg.contains("hj"),
        "expected helpful error, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Pure-logic tests
// ---------------------------------------------------------------------------

#[test]
fn build_handoff_args_contains_required_flags() {
    let args = hj::build_handoff_args(
        "my-project",
        "passing",
        "19 passed",
        "feat: tool integration",
        &["abc1234", "def5678"],
    );
    assert_eq!(args[0], "handoff");
    assert!(args.contains(&"--project".into()));
    assert!(args.contains(&"my-project".into()));
    assert!(args.contains(&"--build".into()));
    assert!(args.contains(&"passing".into()));
    assert!(args.contains(&"--tests".into()));
    assert!(args.contains(&"19 passed".into()));
    assert!(args.contains(&"--log-summary".into()));
    assert!(args.contains(&"feat: tool integration".into()));
    assert!(args.contains(&"--commit".into()));
    assert!(args.contains(&"abc1234".into()));
    assert!(args.contains(&"def5678".into()));
}

#[test]
fn build_handoff_args_no_commits() {
    let args = hj::build_handoff_args("proj", "ok", "ok", "summary", &[]);
    assert!(!args.contains(&"--commit".into()));
}

#[test]
fn build_handoff_args_multiple_commits_each_flagged() {
    let args = hj::build_handoff_args("p", "ok", "ok", "s", &["a", "b", "c"]);
    let commit_flags = args.iter().filter(|a| a.as_str() == "--commit").count();
    assert_eq!(commit_flags, 3);
}
