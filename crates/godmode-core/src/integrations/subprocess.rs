//! Shared subprocess execution helper for integration shell-outs.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Git environment variables that pin a repository regardless of the
/// subprocess working directory. [`run_in`] strips them so its `cwd` contract
/// survives when godmode runs under a git hook: hooks inherit `GIT_DIR` and
/// friends from the parent `git` process, which would otherwise leak into
/// every spawned `git`/`hj` and make it operate on the hook's repository
/// instead of the requested directory.
pub(crate) const GIT_PINNED_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
];

/// Run `bin` with `args`, returning stdout on success.
///
/// On failure returns an error with `context_msg` prepended.
/// If the binary is not on PATH, the error message includes `bin not found on PATH`.
pub fn run(bin: &str, args: &[&str], context_msg: &str) -> Result<String> {
    let out = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("{bin} not found on PATH — {context_msg}"))?;
    if !out.status.success() {
        bail!(
            "{bin} {cmd} failed: {stderr}",
            cmd = args.first().copied().unwrap_or(""),
            stderr = String::from_utf8_lossy(&out.stderr),
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Build the [`Command`] for [`run_in`]: `bin`/`args` in `cwd` with
/// [`GIT_PINNED_ENV`] stripped from the child environment.
///
/// `pub(crate)` so every in-crate `git` spawn shares one definition of
/// "operate on the repository at this directory" instead of silently
/// re-inheriting an ambient `GIT_DIR` from a git hook.
pub(crate) fn command_in(bin: &str, args: &[&str], cwd: &Path) -> Command {
    let mut cmd = Command::new(bin);
    cmd.args(args).current_dir(cwd);
    for var in GIT_PINNED_ENV {
        cmd.env_remove(var);
    }
    cmd
}

/// Run `bin` with `args` in `cwd`, returning stdout on success.
///
/// Git's repository-pinning environment variables (see [`GIT_PINNED_ENV`]) are
/// stripped from the child so that `cwd` — not an ambient `GIT_DIR` inherited
/// from a git hook — determines which repository the command operates on.
pub fn run_in(bin: &str, args: &[&str], cwd: &Path, context_msg: &str) -> Result<String> {
    let out = command_in(bin, args, cwd)
        .output()
        .with_context(|| format!("{bin} not found on PATH — {context_msg}"))?;
    if !out.status.success() {
        bail!(
            "{bin} {cmd} failed: {stderr}",
            cmd = args.first().copied().unwrap_or(""),
            stderr = String::from_utf8_lossy(&out.stderr),
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_echo_returns_stdout() {
        let out = run("echo", &["hello"], "echo test").unwrap();
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn run_missing_binary_returns_error() {
        let err = run("__nonexistent_bin_xyz__", &[], "test").unwrap_err();
        assert!(
            err.to_string().contains("not found on PATH"),
            "error should mention PATH: {}",
            err
        );
    }

    #[test]
    fn run_failing_command_returns_error() {
        let err = run("false", &[], "test false").unwrap_err();
        assert!(err.to_string().contains("failed"));
    }

    #[test]
    fn run_in_respects_cwd() {
        let dir = tempfile::TempDir::new().unwrap();
        let out = run_in("pwd", &[], dir.path(), "pwd test").unwrap();
        let expected = std::fs::canonicalize(dir.path()).unwrap();
        let actual = std::fs::canonicalize(out.trim()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn run_in_strips_git_pinned_env() {
        // Regression: git exports GIT_DIR to its hooks, so a `git` spawned by
        // godmode from a hook would otherwise operate on the hook's repository
        // instead of `cwd`. Assert every pinned var is removed for the child.
        let dir = tempfile::TempDir::new().unwrap();
        let cmd = command_in("git", &["status"], dir.path());
        let removed: Vec<String> = cmd
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(key, _)| key.to_string_lossy().into_owned())
            .collect();
        for var in GIT_PINNED_ENV {
            assert!(
                removed.iter().any(|k| k == var),
                "{var} should be stripped from the child environment"
            );
        }
    }
}
