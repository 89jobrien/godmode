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
