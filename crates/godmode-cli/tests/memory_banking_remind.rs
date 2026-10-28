//! Integration coverage for `godmode memory-banking remind`.
//!
//! `remind` prints its answer rather than returning it, so the in-crate
//! regression tests cannot observe it. This drives the real binary instead and
//! checks the one thing a pinned `GIT_DIR` used to break: which repository the
//! reminder is computed from.
//!
//! git exports an absolute `GIT_DIR` to hooks run from a linked worktree — the
//! shape godmode is developed in — so a `remind` that ignores it would report
//! the *hook's* session rather than the one the user is working in. The test
//! pins `GIT_DIR` in the child process only, so nothing global is mutated.

use std::path::Path;
use std::process::{Command, Output};

mod common;

/// A `git` command in `dir` with the repository pins stripped.
fn git_in(dir: &Path) -> Command {
    common::scrubbed("git", dir)
}

/// A command for `bin` in `dir` with the repository pins stripped.
///
/// A `git` hook run from a linked worktree exports an absolute `GIT_DIR` to its
/// children, and this crate is developed from a linked worktree. A scratch
/// repository created or driven while that pin is inherited would read and
/// write the *hook's* repository instead, so every child here strips the pins
/// before any deliberate pin is applied.
fn scrubbed(bin: &Path, dir: &Path) -> Command {
    common::scrubbed(bin, dir)
}

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

/// A throwaway repository with a local identity and no hooks.
///
/// Hooks and commit signing are disabled locally: the developer's global
/// configuration signs through 1Password and runs taskit gates, neither of
/// which belongs in a scratch repository.
fn scratch_repo() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let path = dir.path();
    let hooks = tempfile::TempDir::new().expect("empty hooks dir");
    for args in [
        vec!["init", "--quiet"],
        vec!["config", "user.name", "godmode test"],
        vec!["config", "user.email", "test@example.invalid"],
        vec!["config", "commit.gpgsign", "false"],
        vec!["config", "tag.gpgsign", "false"],
        vec![
            "config",
            "core.hooksPath",
            hooks.path().to_str().expect("hooks path is UTF-8"),
        ],
    ] {
        let status = git_in(path).args(&args).status().expect("run git");
        assert!(status.success(), "git {args:?} must succeed");
    }
    dir
}

/// Commit `message` with an explicit author and committer date.
fn commit_at(repo: &Path, message: &str, date: &str) {
    let status = git_in(repo)
        .args(["commit", "--quiet", "--allow-empty", "-m", message])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .status()
        .expect("run git commit");
    assert!(status.success(), "git commit must succeed");
}

/// Run the binary in `dir` with `GIT_DIR` pinned to `pin`.
///
/// The pins are stripped first, so the child sees exactly the `GIT_DIR` this
/// test sets — nothing a git hook exported upstream.
fn run_remind(dir: &Path, pin: &Path) -> Output {
    scrubbed(&godmode_bin(), dir)
        .args(["memory-banking", "remind", "--json"])
        .env("GIT_DIR", pin)
        .output()
        .expect("run godmode")
}

/// Regression: the reminder must describe the working repository, not the one
/// an inherited `GIT_DIR` points at.
#[test]
fn remind_reports_the_working_repo_not_the_pinned_one() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock is after the epoch")
        .as_secs();
    // git accepts the raw form `@<seconds> +0000` for its date variables.
    let recent = format!("@{} +0000", now);
    let stale = format!("@{} +0000", now - 6 * 60 * 60);

    // The working repository had a commit minutes ago, so it earns a reminder.
    let working = scratch_repo();
    commit_at(working.path(), "feat: recent work", &recent);

    // The pinned repository's only commit is hours old, so it earns nothing.
    let pinned = scratch_repo();
    commit_at(pinned.path(), "chore: stale work", &stale);

    let out = run_remind(working.path(), &pinned.path().join(".git"));
    assert!(
        out.status.success(),
        "remind must exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"reminder\": true"),
        "the working repo had a recent commit, so remind must report it: {stdout}"
    );
}

/// Regression (config leak): `scratch_repo` must never write its local
/// identity / signing / hooks settings into a repository inherited through
/// `GIT_DIR`.
///
/// Before the spawn sites were scrubbed, an unscrubbed `git config` under a
/// pinned `GIT_DIR` wrote `user.name = "godmode test"`, `commit.gpgsign = false`,
/// and a dead `core.hooksPath` into the real (hook's) repository — the
/// incident that disabled signing and hooks on this very repo. This test
/// pins `GIT_DIR` at a stand-in repository, runs `scratch_repo`, and asserts
/// the pinned repository's config is byte-identical afterwards.
#[test]
fn scratch_repo_does_not_leak_config_into_a_pinned_git_dir() {
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    let _guard = ENV_LOCK.lock().expect("env lock");

    // Stand-in for the hook's repository.
    let pinned = tempfile::TempDir::new().expect("pinned temp dir");
    let pinned_git = pinned.path().join(".git");
    // `git init` honours `GIT_DIR`, so under a pinned environment it would
    // initialise the *pinned* (real) repository instead of this temp dir.
    // Scrub first, exactly like the fixture under test does.
    let init = common::scrubbed("git", pinned.path())
        .args(["init", "--quiet"])
        .status()
        .expect("git init pinned");
    assert!(init.success(), "pinned git init must succeed");
    let before =
        std::fs::read_to_string(pinned_git.join("config")).expect("read pinned config before");

    // Save, then install, the authentic pin shape a linked-worktree hook exports.
    let saved: Vec<(&str, Option<std::ffi::OsString>)> = common::GIT_PINNED_ENV
        .iter()
        .map(|v| (*v, std::env::var_os(v)))
        .collect();
    unsafe {
        std::env::set_var("GIT_DIR", &pinned_git);
        std::env::set_var("GIT_INDEX_FILE", pinned_git.join("index"));
        std::env::set_var("GIT_PREFIX", "");
    }
    for v in common::GIT_PINNED_ENV {
        if *v != "GIT_DIR" && *v != "GIT_INDEX_FILE" && *v != "GIT_PREFIX" {
            unsafe { std::env::remove_var(v) };
        }
    }

    // The fixture that performs git init + config writes. It must target its
    // own repository, never the pinned one.
    let scratch = scratch_repo();
    let _ = scratch.path();

    // Restore the process environment.
    for (k, val) in saved {
        unsafe {
            match val {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }

    let after =
        std::fs::read_to_string(pinned_git.join("config")).expect("read pinned config after");

    assert_eq!(
        before, after,
        "scratch_repo leaked configuration into the pinned GIT_DIR repository"
    );
    assert!(
        !after.contains("godmode test"),
        "pinned repo must not receive the scratch identity"
    );
    assert!(
        !after.contains("test@example.invalid"),
        "pinned repo must not receive the scratch email"
    );
}
