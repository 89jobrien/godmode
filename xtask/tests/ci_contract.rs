//! Process-level contracts for the repository maintenance task.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    bin: PathBuf,
    log: PathBuf,
    home: PathBuf,
    target: PathBuf,
    elsewhere: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "godmode-xtask-contract-{}-{id}",
            std::process::id()
        ));
        let fixture = Self {
            bin: root.join("bin"),
            log: root.join("commands.log"),
            home: root.join("home"),
            target: root.join("target"),
            elsewhere: root.join("elsewhere"),
            root,
        };
        for directory in [
            &fixture.bin,
            &fixture.home,
            &fixture.target,
            &fixture.elsewhere,
        ] {
            fs::create_dir_all(directory).unwrap();
        }
        fixture.fake_tool("cargo");
        fixture.fake_tool("nu");
        fixture
    }

    fn fake_tool(&self, name: &str) {
        let script = format!(
            r#"#!/bin/sh
printf {name} >> "$FAKE_LOG"
for argument in "$@"; do
    printf "\t%s" "$argument" >> "$FAKE_LOG"
done
printf "\tPWD=%s\n" "$PWD" >> "$FAKE_LOG"
if [ "${{FAKE_CREATE_DIST:-}}" = 1 ] && [ "$*" = "build --release -p godmode-cli" ]; then
    /bin/mkdir -p "$CARGO_TARGET_DIR/release"
    printf binary > "$CARGO_TARGET_DIR/release/godmode"
fi
if [ "${{FAKE_FAIL_MATCH:-}}" = "{name} $*" ]; then
    exit "${{FAKE_FAIL_STATUS:-17}}"
fi
exit 0
"#
        );
        let path = self.bin.join(name);
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn command(&self, xtask: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .arg(xtask)
            .current_dir(&self.elsewhere)
            .env("PATH", &self.bin)
            .env("HOME", &self.home)
            .env("CARGO_TARGET_DIR", &self.target)
            .env("FAKE_LOG", &self.log)
            .env_remove("CARGO_MANIFEST_DIR");
        command
    }

    fn run(&self, xtask: &str) -> Output {
        self.command(xtask).output().unwrap()
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn expected(program: &str, args: &[&str]) -> String {
    format!(
        "{program}\t{}\tPWD={}\n",
        args.join("\t"),
        workspace_root().display()
    )
}

#[test]
fn ci_runs_every_command_in_order_with_split_arguments_from_project_root() {
    let fixture = Fixture::new();
    let output = fixture.run("ci");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected_log = [
        expected("cargo", &["fmt", "--all", "--check"]),
        expected(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        ),
        expected("cargo", &["nextest", "run", "--workspace"]),
        expected(
            "cargo",
            &[
                "run",
                "-p",
                "godmode-conformance",
                "--bin",
                "run-conformance",
                "--",
                "--verbose",
            ],
        ),
        expected("nu", &["tests/conformance/plugin-structure.nu"]),
        expected(
            "cargo",
            &[
                "run",
                "-q",
                "-p",
                "godmode-cli",
                "--",
                "skill",
                "index",
                "--check",
            ],
        ),
        expected(
            "cargo",
            &[
                "run",
                "-q",
                "-p",
                "godmode-cli",
                "--",
                "agent",
                "index",
                "--check",
            ],
        ),
        expected(
            "cargo",
            &[
                "run",
                "-q",
                "-p",
                "godmode-cli",
                "--",
                "command",
                "generate",
                "--target",
                "claude",
                "--check",
            ],
        ),
        expected(
            "cargo",
            &[
                "run",
                "-q",
                "-p",
                "godmode-cli",
                "--",
                "release",
                "validate",
            ],
        ),
        expected("cargo", &["deny", "check"]),
    ]
    .concat();
    assert_eq!(fixture.log(), expected_log);
}

#[test]
fn pre_commit_reaches_its_commands_in_order() {
    let fixture = Fixture::new();
    let output = fixture.run("pre-commit");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture.log(),
        [
            expected("cargo", &["fmt", "--all", "--check"]),
            expected("cargo", &["clippy", "--workspace", "--", "-D", "warnings"]),
            expected(
                "cargo",
                &[
                    "run",
                    "-p",
                    "godmode-conformance",
                    "--bin",
                    "run-conformance",
                    "--",
                    "--verbose",
                ],
            ),
        ]
        .concat()
    );
}

#[test]
fn child_failure_status_is_propagated_and_later_commands_are_unreachable() {
    let fixture = Fixture::new();
    let output = fixture
        .command("ci")
        .env("FAKE_FAIL_MATCH", "cargo nextest run --workspace")
        .env("FAKE_FAIL_STATUS", "23")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("exit status: 23"));
    assert_eq!(fixture.log().lines().count(), 3);
    assert!(!fixture.log().contains("godmode-conformance"));
}

#[test]
fn dist_uses_project_root_when_invoked_elsewhere() {
    let fixture = Fixture::new();
    let output = fixture.run("dist");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture.log(),
        expected("cargo", &["build", "--release", "-p", "godmode-cli"])
    );
}

#[test]
fn install_builds_then_copies_binary_under_home() {
    let fixture = Fixture::new();
    let output = fixture
        .command("install")
        .env("FAKE_CREATE_DIST", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(fixture.home.join(".cargo/bin/godmode")).unwrap(),
        b"binary"
    );
}

#[test]
fn install_reports_missing_home_after_successful_build() {
    let fixture = Fixture::new();
    let output = fixture
        .command("install")
        .env("FAKE_CREATE_DIST", "1")
        .env_remove("HOME")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("HOME not set"));
    assert_eq!(fixture.log().lines().count(), 1);
}

#[test]
fn install_propagates_copy_failures() {
    let fixture = Fixture::new();
    let destination = fixture.home.join(".cargo/bin/godmode");
    fs::create_dir_all(&destination).unwrap();
    let output = fixture
        .command("install")
        .env("FAKE_CREATE_DIST", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to copy"), "{stderr}");
    assert!(
        stderr.contains(&destination.display().to_string()),
        "{stderr}"
    );
}

#[test]
fn unknown_command_returns_failure_without_spawning_children() {
    let fixture = Fixture::new();
    let output = fixture.run("not-a-command");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown xtask: not-a-command"));
    assert_eq!(fixture.log(), "");
}
