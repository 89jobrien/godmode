use godmode_core::integrations::hj;

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
