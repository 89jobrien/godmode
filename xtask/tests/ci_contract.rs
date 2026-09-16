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
    elsewhere: PathBuf,
    home: PathBuf,
    target: PathBuf,
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
            elsewhere: root.join("elsewhere"),
            home: root.join("home"),
            target: root.join("custom-target"),
            root,
        };
        fs::create_dir_all(&fixture.bin).unwrap();
        fs::create_dir_all(&fixture.elsewhere).unwrap();
        fs::create_dir_all(&fixture.home).unwrap();
        fs::create_dir_all(&fixture.target).unwrap();
        fixture.fake_tool("cargo");
        fixture.fake_tool("just");
        fixture
    }
    fn fake_tool(&self, name: &str) {
        let script = format!(
            r#"#!/bin/sh
printf {name} >> "$FAKE_LOG"
for argument in "$@"; do printf "\t%s" "$argument" >> "$FAKE_LOG"; done
printf "\tPWD=%s\n" "$PWD" >> "$FAKE_LOG"
if [ "${{FAKE_CREATE_DIST:-}}" = 1 ] && [ "$*" = "build --release -p godmode-cli" ]; then
  /bin/mkdir -p "$CARGO_TARGET_DIR/release"
  printf binary > "$CARGO_TARGET_DIR/release/godmode"
fi
if [ "${{FAKE_FAIL_MATCH:-}}" = "{name} $*" ]; then exit 23; fi
"#
        );
        let path = self.bin.join(name);
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
    fn command(&self, task: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .arg(task)
            .current_dir(&self.elsewhere)
            .env("PATH", &self.bin)
            .env("FAKE_LOG", &self.log)
            .env("HOME", &self.home)
            .env("CARGO_TARGET_DIR", &self.target);
        command
    }
    fn run(&self, task: &str) -> Output {
        self.command(task).output().unwrap()
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

#[test]
fn ci_runs_current_workspace_quality_contract_from_project_root() {
    let fixture = Fixture::new();
    let output = fixture.run("ci");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    for command in [
        "cargo\tfmt\t--all\t--check",
        "cargo\tcheck\t--workspace\t--all-features",
        "cargo\tclippy\t--workspace\t--all-targets\t--all-features\t--\t-D\twarnings",
        "cargo\tnextest\trun\t--workspace\t--all-features",
        "cargo\tdeny\tcheck",
        "just\tconformance",
    ] {
        assert!(
            log.contains(command),
            "missing command {command:?} in {log:?}"
        );
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    assert!(
        log.lines()
            .all(|line| line.contains(&format!("PWD={}", root.display())))
    );
}

#[test]
fn ci_stops_after_the_first_failed_gate() {
    let fixture = Fixture::new();
    let output = fixture
        .command("ci")
        .env("FAKE_FAIL_MATCH", "cargo check --workspace --all-features")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fixture.log().lines().count(), 2);
    assert!(!fixture.log().contains("clippy"));
}

#[test]
fn install_uses_cargo_target_dir_for_the_built_binary() {
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
