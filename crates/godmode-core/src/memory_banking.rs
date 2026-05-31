//! Memory banking — persistent source-backed project context at `.ctx/memory-banking/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{Context, Result};

/// All memory-bank template file names in display order.
const TEMPLATE_FILES: &[&str] = &[
    "project-brief.md",
    "product-context.md",
    "tech-context.md",
    "system-patterns.md",
    "active-context.md",
    "progress.md",
];

/// Resolved memory-banking directory for a git root.
pub fn memory_banking_dir(git_root: &Path) -> PathBuf {
    git_root.join(".ctx").join("memory-banking")
}

/// Returns true if `.ctx/memory-banking/` exists and has at least one .md file.
pub fn exists(git_root: &Path) -> bool {
    let dir = memory_banking_dir(git_root);
    dir.is_dir()
        && fs::read_dir(&dir).is_ok_and(|mut entries| {
            entries.any(|e| e.is_ok_and(|e| e.path().extension().is_some_and(|ext| ext == "md")))
        })
}

/// List .md files in the memory-banking directory.
pub fn list_files(git_root: &Path) -> Result<Vec<PathBuf>> {
    let dir = memory_banking_dir(git_root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort();
    Ok(files)
}

/// Get the unix timestamp of the most recent commit (seconds since epoch).
/// Returns 0 if git is unavailable or repo has no commits.
fn latest_commit_timestamp(git_root: &Path) -> u64 {
    Command::new("git")
        .args(["log", "-1", "--format=%ct"])
        .current_dir(git_root)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

/// Check which files are stale (modified before the latest commit).
pub fn stale_files(git_root: &Path) -> Result<Vec<String>> {
    let commit_ts = latest_commit_timestamp(git_root);
    if commit_ts == 0 {
        return Ok(vec![]);
    }

    let commit_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(commit_ts);
    let files = list_files(git_root)?;
    let mut stale = Vec::new();

    for file in files {
        let modified = fs::metadata(&file)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if modified < commit_time
            && let Some(name) = file.file_name()
        {
            stale.push(name.to_string_lossy().to_string());
        }
    }

    Ok(stale)
}

/// Print all memory-bank file contents for context injection.
/// Returns the output as a string (for --json mode) or prints directly.
pub fn inject(git_root: &Path, json: bool) -> Result<()> {
    let files = list_files(git_root)?;
    if files.is_empty() {
        return Ok(());
    }

    let stale = stale_files(git_root)?;

    if json {
        let mut entries = Vec::new();
        for file in &files {
            let name = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content =
                fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
            let is_stale = stale.contains(&name);
            entries.push(serde_json::json!({
                "file": name,
                "content": content,
                "stale": is_stale,
            }));
        }
        let output = serde_json::json!({
            "memory_banking": entries,
            "stale_files": stale,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("[memory-banking] Injecting project context from .ctx/memory-banking/");
        for file in &files {
            let basename = file.file_name().unwrap_or_default().to_string_lossy();
            let content =
                fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
            println!("--- {} ---", basename);
            println!("{}", content);
        }
        if !stale.is_empty() {
            println!(
                "[memory-banking] STALE: {} — older than latest commit. Consider updating.",
                stale.join(", ")
            );
        }
    }

    Ok(())
}

/// Check for recent commits and print a reminder to update memory-bank files.
pub fn remind(git_root: &Path, json: bool) -> Result<()> {
    let output = Command::new("git")
        .args(["log", "--since=2 hours ago", "--oneline"])
        .current_dir(git_root)
        .output()?;

    let has_commits =
        output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty();

    if has_commits {
        if json {
            let msg = serde_json::json!({
                "reminder": true,
                "message": "Session had commits. Update .ctx/memory-banking/active-context.md and progress.md before ending.",
            });
            println!("{}", serde_json::to_string_pretty(&msg)?);
        } else {
            println!(
                "[memory-banking] Session had commits. Update .ctx/memory-banking/active-context.md and progress.md before ending."
            );
        }
    }

    Ok(())
}

/// Create `.ctx/memory-banking/` with empty template files.
pub fn init(git_root: &Path) -> Result<()> {
    let dir = memory_banking_dir(git_root);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    for &name in TEMPLATE_FILES {
        let path = dir.join(name);
        if !path.exists() {
            let heading = name.trim_end_matches(".md").replace('-', " ");
            let content = format!(
                "# {}\n\n<!-- TODO: populate from source code -->\n",
                capitalize(&heading)
            );
            fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
        }
    }

    println!(
        "[memory-banking] Initialized .ctx/memory-banking/ with {} template files.",
        TEMPLATE_FILES.len()
    );
    Ok(())
}

/// Show status of memory-banking directory.
pub fn status(git_root: &Path, json: bool) -> Result<()> {
    let dir = memory_banking_dir(git_root);
    if !dir.is_dir() {
        if json {
            println!(r#"{{"exists": false}}"#);
        } else {
            println!("[memory-banking] Not initialized. Run `godmode memory-banking init`.");
        }
        return Ok(());
    }

    let files = list_files(git_root)?;
    let stale = stale_files(git_root)?;

    if json {
        let file_names: Vec<String> = files
            .iter()
            .filter_map(|f| f.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        let output = serde_json::json!({
            "exists": true,
            "files": file_names,
            "stale_files": stale,
            "total": files.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "[memory-banking] {} files in .ctx/memory-banking/",
            files.len()
        );
        for file in &files {
            let name = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let marker = if stale.contains(&name) {
                " (STALE)"
            } else {
                ""
            };
            println!("  {}{}", name, marker);
        }
    }

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_fake_root() -> TempDir {
        let tmp = TempDir::new().expect("create tempdir");
        tmp
    }

    fn setup_with_memory_bank(files: &[(&str, &str)]) -> TempDir {
        let tmp = setup_fake_root();
        let mb = tmp.path().join(".ctx").join("memory-banking");
        fs::create_dir_all(&mb).expect("create memory-banking dir");
        for (name, content) in files {
            fs::write(mb.join(name), content).expect("write file");
        }
        tmp
    }

    #[test]
    fn memory_banking_dir_resolves_correctly() {
        let p = Path::new("/some/project");
        assert_eq!(
            memory_banking_dir(p),
            PathBuf::from("/some/project/.ctx/memory-banking")
        );
    }

    #[test]
    fn exists_false_when_no_dir() {
        let tmp = setup_fake_root();
        assert!(!exists(tmp.path()));
    }

    #[test]
    fn exists_false_when_dir_empty() {
        let tmp = setup_fake_root();
        let mb = tmp.path().join(".ctx").join("memory-banking");
        fs::create_dir_all(&mb).unwrap();
        assert!(!exists(tmp.path()));
    }

    #[test]
    fn exists_false_when_only_non_md_files() {
        let tmp = setup_fake_root();
        let mb = tmp.path().join(".ctx").join("memory-banking");
        fs::create_dir_all(&mb).unwrap();
        fs::write(mb.join(".DS_Store"), "junk").unwrap();
        assert!(!exists(tmp.path()));
    }

    #[test]
    fn exists_true_when_md_present() {
        let tmp = setup_with_memory_bank(&[("progress.md", "# Progress")]);
        assert!(exists(tmp.path()));
    }

    #[test]
    fn list_files_empty_when_no_dir() {
        let tmp = setup_fake_root();
        let files = list_files(tmp.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn list_files_returns_only_md_sorted() {
        let tmp = setup_with_memory_bank(&[
            ("z-file.md", "z"),
            ("a-file.md", "a"),
            (".DS_Store", "junk"),
            ("notes.txt", "not md"),
        ]);
        let files = list_files(tmp.path()).unwrap();
        let names: Vec<&str> = files
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect();
        assert_eq!(names, vec!["a-file.md", "z-file.md"]);
    }

    #[test]
    fn init_creates_all_template_files() {
        let tmp = setup_fake_root();
        init(tmp.path()).unwrap();

        let mb = memory_banking_dir(tmp.path());
        assert!(mb.is_dir());

        for &name in TEMPLATE_FILES {
            let path = mb.join(name);
            assert!(path.exists(), "missing {name}");
            let content = fs::read_to_string(&path).unwrap();
            assert!(
                content.starts_with("# "),
                "file {name} should start with heading"
            );
        }
    }

    #[test]
    fn init_does_not_overwrite_existing() {
        let tmp = setup_with_memory_bank(&[("progress.md", "# Custom content")]);
        init(tmp.path()).unwrap();

        let content =
            fs::read_to_string(memory_banking_dir(tmp.path()).join("progress.md")).unwrap();
        assert_eq!(content, "# Custom content");
    }

    #[test]
    fn stale_files_empty_when_no_git() {
        let tmp = setup_with_memory_bank(&[("progress.md", "# p")]);
        // No .git dir, so latest_commit_timestamp returns 0
        let stale = stale_files(tmp.path()).unwrap();
        assert!(stale.is_empty());
    }

    #[test]
    fn capitalize_works() {
        assert_eq!(capitalize("hello world"), "Hello world");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("A"), "A");
    }
}
