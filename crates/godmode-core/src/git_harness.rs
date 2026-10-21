//! Test-only git fixtures for the repository-pin regression suite.
//!
//! `git` exports an absolute `GIT_DIR` to hooks that run from a linked
//! worktree — the shape godmode is developed in (`.worktrees/<name>` plus
//! `git push`). A child `git` that inherits that pin reads the hook's
//! repository regardless of `-C` or `current_dir`, so these fixtures hand a
//! helper a **fixture** repository while a **decoy** repository is pinned.
//!
//! Fixtures disable hooks and commit signing locally: the developer's global
//! configuration signs through 1Password and runs taskit gates, neither of
//! which belongs in a throwaway repository.
#![allow(dead_code)]

use std::path::Path;
use std::process::Output;
use std::sync::{Mutex, MutexGuard, PoisonError};

use tempfile::TempDir;

use crate::integrations::subprocess::{GIT_PINNED_ENV, command_in};

/// Serialises tests that mutate process-wide `GIT_*` variables.
///
/// nextest runs every test in its own process, so the lock is uncontended
/// there; it keeps plain `cargo test` (threads in one process) deterministic.
static GIT_ENV_LOCK: Mutex<()> = Mutex::new(());

fn git_ok_in(cwd: &Path, args: &[&str]) -> String {
    let out = command_in("git", args, cwd)
        .output()
        .unwrap_or_else(|error| panic!("failed to run git {args:?}: {error}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A throwaway repository with a local identity and no hooks.
pub(crate) struct GitRepo {
    dir: TempDir,
    hooks: TempDir,
}

impl GitRepo {
    /// Initialise a repository whose commits and tags never touch the network,
    /// the global commit-signing key, or the global hooks.
    pub(crate) fn init() -> Self {
        let dir = TempDir::new().expect("tempdir for a git fixture");
        let hooks = TempDir::new().expect("tempdir for empty git hooks");
        let repo = Self { dir, hooks };
        repo.git_ok(&["init", "--quiet"]);
        repo.git_ok(&["config", "user.name", "godmode test"]);
        repo.git_ok(&["config", "user.email", "test@example.invalid"]);
        repo.git_ok(&["config", "commit.gpgsign", "false"]);
        repo.git_ok(&["config", "tag.gpgsign", "false"]);
        // The global `core.hooksPath` runs taskit gates; point it at an empty
        // directory held outside the fixture working tree.
        let hooks_path = repo
            .hooks
            .path()
            .to_str()
            .expect("tempdir path is UTF-8")
            .to_string();
        repo.git_ok(&["config", "core.hooksPath", &hooks_path]);
        repo
    }

    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Commit `message` and return the new `HEAD` sha.
    ///
    /// `date` pins the author and committer time (ISO 8601 with an offset), so
    /// tests can order commits without sleeping.
    pub(crate) fn commit(&self, message: &str, date: Option<&str>) -> String {
        let mut cmd = command_in(
            "git",
            &["commit", "--quiet", "--allow-empty", "-m", message],
            self.path(),
        );
        if let Some(date) = date {
            cmd.env("GIT_AUTHOR_DATE", date)
                .env("GIT_COMMITTER_DATE", date);
        }
        let out = cmd.output().expect("run git commit");
        assert!(
            out.status.success(),
            "git commit failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        self.git_ok(&["rev-parse", "HEAD"])
    }

    pub(crate) fn tag(&self, name: &str) {
        self.git_ok(&["tag", name]);
    }

    pub(crate) fn remote(&self, url: &str) {
        self.git_ok(&["remote", "add", "origin", url]);
    }

    /// Write `body` to `relative`, creating parent directories.
    pub(crate) fn write(&self, relative: &str, body: &str) {
        let path = self.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture directory");
        }
        std::fs::write(&path, body).expect("write fixture file");
    }

    pub(crate) fn git(&self, args: &[&str]) -> Output {
        command_in("git", args, self.path())
            .output()
            .unwrap_or_else(|error| panic!("failed to run git {args:?}: {error}"))
    }

    pub(crate) fn git_ok(&self, args: &[&str]) -> String {
        git_ok_in(self.path(), args)
    }
}

/// A bare repository used as a push target; never touches the network.
pub(crate) struct BareRepo {
    dir: TempDir,
}

impl BareRepo {
    pub(crate) fn init() -> Self {
        let dir = TempDir::new().expect("tempdir for a bare git fixture");
        let repo = Self { dir };
        git_ok_in(repo.path(), &["init", "--quiet", "--bare"]);
        repo
    }

    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }

    pub(crate) fn git_ok(&self, args: &[&str]) -> String {
        git_ok_in(self.path(), args)
    }
}

/// Pins `GIT_DIR` for the test scope and restores the process environment on
/// drop, clearing the rest of git's repository-pin set first so a developer
/// shell cannot skew the fixture.
pub(crate) struct PinnedGitDir {
    _lock: MutexGuard<'static, ()>,
    prior: Vec<(&'static str, Option<String>)>,
}

impl PinnedGitDir {
    /// Pin `repo`'s git directory — the shape git hands to a linked-worktree
    /// hook, and enough to redirect every repository-reading command (`log`,
    /// `describe`, `tag`, `diff`, `worktree list`).
    pub(crate) fn pointing_at(repo: &GitRepo) -> Self {
        Self::pin(&repo.path().join(".git"), None)
    }

    /// Pin `GIT_DIR` and `GIT_WORK_TREE` together. The work tree variable is
    /// what redirects `git rev-parse --show-toplevel` away from the process
    /// working directory.
    pub(crate) fn with_work_tree(repo: &GitRepo) -> Self {
        Self::pin(&repo.path().join(".git"), Some(repo.path()))
    }

    fn pin(git_dir: &Path, work_tree: Option<&Path>) -> Self {
        let lock = GIT_ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let mut prior = Vec::with_capacity(GIT_PINNED_ENV.len());
        for var in GIT_PINNED_ENV {
            prior.push((*var, std::env::var(var).ok()));
            // SAFETY: the guard in `_lock` serialises every test that mutates
            // these keys, and is only released after `drop` restores them.
            unsafe { std::env::remove_var(var) };
        }
        // SAFETY: as above.
        unsafe { std::env::set_var("GIT_DIR", git_dir) };
        if let Some(work_tree) = work_tree {
            // SAFETY: as above.
            unsafe { std::env::set_var("GIT_WORK_TREE", work_tree) };
        }
        Self { _lock: lock, prior }
    }
}

impl Drop for PinnedGitDir {
    fn drop(&mut self) {
        // SAFETY: `_lock` has not been dropped yet, so no other test in this
        // process can be mutating these keys.
        for (key, prior) in self.prior.drain(..) {
            match prior {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}
