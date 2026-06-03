use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::integrations::subprocess;

pub struct WorktreeInfo {
    pub branch: String,
    pub path: PathBuf,
    pub issue_number: Option<u64>,
}

pub fn gitignore_contains(root: &Path, entry: &str) -> bool {
    let path = root.join(".gitignore");
    match fs::read_to_string(&path) {
        Ok(content) => content.lines().any(|line| line.trim() == entry),
        Err(_) => false,
    }
}

pub fn ensure_gitignore(root: &Path, entry: &str) -> Result<()> {
    if gitignore_contains(root, entry) {
        return Ok(());
    }
    let path = root.join(".gitignore");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let new_content = format!("{}\n{}\n", existing, entry);
    fs::write(&path, new_content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn add(root: &Path, branch: &str, issue_number: Option<u64>) -> Result<WorktreeInfo> {
    let worktrees_dir = root.join(".worktrees");
    fs::create_dir_all(&worktrees_dir)
        .with_context(|| format!("failed to create {}", worktrees_dir.display()))?;

    ensure_gitignore(root, ".worktrees/")?;

    // Fetch origin main — degrade gracefully on failure
    let root_str = root.to_str().unwrap_or(".");
    if let Err(e) = subprocess::run(
        "git",
        &["-C", root_str, "fetch", "origin", "main"],
        "git fetch",
    ) {
        eprintln!("warn: git fetch origin main failed: {e}");
    }

    let worktree_path = format!(".worktrees/{branch}");
    subprocess::run(
        "git",
        &[
            "-C",
            root_str,
            "worktree",
            "add",
            &worktree_path,
            "-b",
            branch,
        ],
        "git worktree add",
    )
    .with_context(|| "failed to run git worktree add")?;

    Ok(WorktreeInfo {
        branch: branch.to_string(),
        path: root.join(".worktrees").join(branch),
        issue_number,
    })
}

pub fn remove(root: &Path, branch: &str) -> Result<()> {
    let root_str = root.to_str().unwrap_or(".");

    // Check for unmerged commits
    let log_arg = format!("main..{branch}");
    let log_output = subprocess::run(
        "git",
        &["-C", root_str, "log", "--oneline", &log_arg],
        "git log for unmerged check",
    )
    .with_context(|| "failed to run git log")?;

    if !log_output.trim().is_empty() {
        bail!("branch {branch} has unmerged commits — merge to main first");
    }

    let worktree_path = format!(".worktrees/{branch}");
    subprocess::run(
        "git",
        &["-C", root_str, "worktree", "remove", &worktree_path],
        "git worktree remove",
    )
    .with_context(|| "failed to run git worktree remove")?;

    subprocess::run(
        "git",
        &["-C", root_str, "branch", "-d", branch],
        "git branch -d",
    )
    .with_context(|| "failed to run git branch -d")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gitignore_contains_detects_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), ".worktrees/\n.ctx/\n").unwrap();
        assert!(gitignore_contains(dir.path(), ".worktrees/"));
        assert!(!gitignore_contains(dir.path(), ".env"));
    }

    #[test]
    fn gitignore_missing_file_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!gitignore_contains(dir.path(), ".worktrees/"));
    }

    #[test]
    fn ensure_gitignore_appends_missing_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), ".ctx/\n").unwrap();
        ensure_gitignore(dir.path(), ".worktrees/").unwrap();
        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains(".worktrees/"));
    }

    #[test]
    fn ensure_gitignore_no_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), ".worktrees/\n").unwrap();
        ensure_gitignore(dir.path(), ".worktrees/").unwrap();
        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(content.matches(".worktrees/").count(), 1);
    }

    #[test]
    fn ensure_gitignore_creates_file_if_absent() {
        let dir = tempfile::tempdir().unwrap();
        ensure_gitignore(dir.path(), ".worktrees/").unwrap();
        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains(".worktrees/"));
    }
}
