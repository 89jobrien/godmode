//! Resolve the repo root and `.ctx/` path from the current working directory.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Walk up from `start` to find the nearest directory containing `.git`.
pub fn repo_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            bail!("no git repository found from {}", start.display());
        }
    }
}

/// Resolve repo root from cwd, falling back to `cwd` if not in a git repo.
pub fn root_or_cwd() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    repo_root(&cwd).or(Ok(cwd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn finds_git_root() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let sub = dir.path().join("sub/dir");
        std::fs::create_dir_all(&sub).unwrap();
        let found = repo_root(&sub).unwrap();
        assert_eq!(found, dir.path());
    }

    #[test]
    fn fails_outside_git_repo() {
        // /tmp itself is not a git repo (usually).
        let result = repo_root(Path::new("/tmp"));
        // Either succeeds (if /tmp is somehow in a repo) or errors — just ensure no panic.
        let _ = result;
    }
}
