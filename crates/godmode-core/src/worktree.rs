use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

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
    let fetch = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "fetch",
            "origin",
            "main",
        ])
        .output();
    if let Err(e) = fetch {
        eprintln!("warn: git fetch origin main failed: {e}");
    } else if let Ok(out) = fetch
        && !out.status.success()
    {
        eprintln!(
            "warn: git fetch origin main exited non-zero: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let worktree_path = format!(".worktrees/{branch}");
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "worktree",
            "add",
            &worktree_path,
            "-b",
            branch,
        ])
        .output()
        .with_context(|| "failed to run git worktree add")?;

    if !out.status.success() {
        bail!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(WorktreeInfo {
        branch: branch.to_string(),
        path: root.join(".worktrees").join(branch),
        issue_number,
    })
}

pub fn remove(root: &Path, branch: &str) -> Result<()> {
    // Check for unmerged commits
    let log_arg = format!("main..{branch}");
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "log",
            "--oneline",
            &log_arg,
        ])
        .output()
        .with_context(|| "failed to run git log")?;

    let log_output = String::from_utf8_lossy(&out.stdout);
    if !log_output.trim().is_empty() {
        bail!("branch {branch} has unmerged commits — merge to main first");
    }

    let worktree_path = format!(".worktrees/{branch}");
    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "worktree",
            "remove",
            &worktree_path,
        ])
        .output()
        .with_context(|| "failed to run git worktree remove")?;
    if !out.status.success() {
        bail!(
            "git worktree remove failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let out = Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."), "branch", "-d", branch])
        .output()
        .with_context(|| "failed to run git branch -d")?;
    if !out.status.success() {
        bail!(
            "git branch -d failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

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
