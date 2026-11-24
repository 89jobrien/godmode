//! Memory banking — persistent source-backed project context at `.ctx/memory-bank/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::integrations::subprocess::command_in;

/// Generated summary file. It is derived from the other bank files, so it is
/// never listed as a source (and never treated as stale).
pub const SUMMARY_FILE: &str = "context-summary.md";

/// Source file stems in summary order; unlisted files follow alphabetically.
/// Live state comes first, static background last: when the budget runs out,
/// whole files are dropped from the end.
const SUMMARY_ORDER: &[&str] = &[
    "active-context",
    "progress",
    "mistakes",
    "project-brief",
    "tech-context",
    "system-patterns",
    "product-context",
];

/// Per-section line caps, tried from most to least generous until the summary
/// fits its byte budget. Tighter caps cut sections into fragments, so they are
/// never tried; trailing files are dropped instead.
const SECTION_LINE_CAPS: &[usize] = &[12, 8, 5, 3];

/// All memory-bank template file names in display order.
const TEMPLATE_FILES: &[&str] = &[
    "project-brief.md",
    "product-context.md",
    "tech-context.md",
    "system-patterns.md",
    "active-context.md",
    "progress.md",
];

/// Resolved memory-bank directory for a git root.
///
/// Prefers the canonical `.ctx/memory-bank/`. Falls back to the intermediate
/// `.ctx/godmode/memory-bank/` and then the original `.ctx/memory-banking/`
/// so existing checkouts keep resolving until they are migrated.
pub fn memory_banking_dir(git_root: &Path) -> PathBuf {
    let canonical = git_root.join(".ctx").join("memory-bank");
    if canonical.exists() {
        return canonical;
    }
    let intermediate = git_root.join(".ctx").join("godmode").join("memory-bank");
    if intermediate.exists() {
        return intermediate;
    }
    let legacy = git_root.join(".ctx").join("memory-banking");
    if legacy.exists() {
        return legacy;
    }
    canonical
}

/// Returns true if the memory-bank directory exists and has at least one .md file.
pub fn exists(git_root: &Path) -> bool {
    let dir = memory_banking_dir(git_root);
    dir.is_dir()
        && fs::read_dir(&dir).is_ok_and(|mut entries| {
            entries.any(|e| e.is_ok_and(|e| e.path().extension().is_some_and(|ext| ext == "md")))
        })
}

/// List source .md files in the memory-banking directory (the generated
/// summary is excluded).
pub fn list_files(git_root: &Path) -> Result<Vec<PathBuf>> {
    let dir = memory_banking_dir(git_root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .filter(|p| p.file_name().is_some_and(|name| name != SUMMARY_FILE))
        .collect();
    files.sort();
    Ok(files)
}

/// Get the unix timestamp of the most recent commit (seconds since epoch).
/// Returns 0 if git is unavailable or repo has no commits.
fn latest_commit_timestamp(git_root: &Path) -> u64 {
    command_in("git", &["log", "-1", "--format=%ct"], git_root)
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

/// Sources in summary order: `SUMMARY_ORDER` first, then the rest alphabetically.
fn summary_ordered(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    let rank = |path: &PathBuf| {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let stem = name.split('.').next().unwrap_or_default().to_string();
        SUMMARY_ORDER
            .iter()
            .position(|s| *s == stem)
            .unwrap_or(SUMMARY_ORDER.len())
    };
    files.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
    files
}

/// Split markdown into a preamble (before the first `##`) and its `##`
/// sections, ignoring headings inside code fences.
fn split_sections(content: &str) -> Vec<(Option<&str>, Vec<&str>)> {
    let mut sections: Vec<(Option<&str>, Vec<&str>)> = vec![(None, Vec::new())];
    let mut in_fence = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && let Some(heading) = line.strip_prefix("## ") {
            sections.push((Some(heading.trim()), Vec::new()));
        } else if let Some(last) = sections.last_mut() {
            last.1.push(line);
        }
    }
    sections
}

/// First `cap` non-blank lines of a section body, extended to finish a wrapped
/// bullet or an open code fence. Returns the kept lines and how many non-blank
/// lines were cut.
fn take_section_lines<'a>(body: &[&'a str], cap: usize) -> (Vec<&'a str>, usize) {
    let mut kept: Vec<&str> = Vec::new();
    let mut counted = 0;
    let mut in_fence = false;
    let mut idx = 0;
    while idx < body.len() {
        let line = body[idx];
        let blank = line.trim().is_empty();
        if blank && kept.is_empty() {
            idx += 1;
            continue;
        }
        if counted >= cap && !in_fence && (blank || !line.starts_with(char::is_whitespace)) {
            break;
        }
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if !blank {
            counted += 1;
        }
        kept.push(line);
        idx += 1;
    }
    while kept.last().is_some_and(|l| l.trim().is_empty()) {
        kept.pop();
    }
    let cut = body[idx..].iter().filter(|l| !l.trim().is_empty()).count();
    (kept, cut)
}

fn is_skipped(heading: &str, skip: &[String]) -> bool {
    let heading = heading.to_lowercase();
    skip.iter().any(|s| heading.starts_with(&s.to_lowercase()))
}

/// One source file's block of the summary: its title, any `Last verified:`
/// line, the head of each non-skipped `##` section, and a pointer to the file.
fn render_source(name: &str, content: &str, cap: usize, skip: &[String]) -> String {
    let sections = split_sections(content);
    let title = sections[0]
        .1
        .iter()
        .find_map(|l| l.strip_prefix("# "))
        .map_or_else(|| name.to_string(), |t| t.trim().to_string());

    let mut out = format!("## {title}\n");
    for line in &sections[0].1 {
        if line.trim_start().starts_with("Last verified") {
            out.push_str(line.trim());
            out.push('\n');
        }
    }
    for (heading, body) in sections.iter().skip(1) {
        let Some(heading) = heading else { continue };
        if is_skipped(heading, skip) {
            continue;
        }
        let (kept, _) = take_section_lines(body, cap);
        if kept
            .iter()
            .all(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
        {
            continue;
        }
        out.push_str(&format!("\n### {heading}\n"));
        for line in kept {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(&format!("\nfull: {name} ({} bytes)\n", content.len()));
    out
}

/// Render the summary for `(file name, content)` sources, already in order.
///
/// Pure and deterministic: the same inputs always give byte-identical output.
/// Section line caps shrink until the whole summary fits `budget`; if the
/// tightest cap does not fit, trailing sources are dropped whole and named in an
/// `omitted:` line. Only a first source that alone overflows is cut at a line
/// boundary.
pub fn render_summary(sources: &[(String, String)], budget: usize, skip: &[String]) -> String {
    const HEADER: &str = "# Context summary\n";
    const TRUNCATED: &str = "\n[truncated: see full files]\n";

    let render = |cap: usize, count: usize| {
        let mut out = String::from(HEADER);
        for (name, content) in &sources[..count] {
            out.push('\n');
            out.push_str(&render_source(name, content, cap, skip));
        }
        if count < sources.len() {
            let omitted: Vec<&str> = sources[count..].iter().map(|(n, _)| n.as_str()).collect();
            out.push_str(&format!("\nomitted: {}\n", omitted.join(", ")));
        }
        out
    };

    for &cap in SECTION_LINE_CAPS {
        let out = render(cap, sources.len());
        if out.len() <= budget {
            return out;
        }
    }

    let tightest_cap = SECTION_LINE_CAPS.last().copied().unwrap_or(1);
    for count in (1..sources.len()).rev() {
        let out = render(tightest_cap, count);
        if out.len() <= budget {
            return out;
        }
    }

    let tightest = render(tightest_cap, sources.len().min(1));
    let room = budget.saturating_sub(TRUNCATED.len());
    let mut end = room.min(tightest.len());
    while !tightest.is_char_boundary(end) {
        end -= 1;
    }
    let cut = tightest[..end].rfind('\n').map_or(0, |i| i + 1);
    format!("{}{TRUNCATED}", &tightest[..cut])
}

/// Build the summary from the bank's source files. `None` when the bank is empty.
pub fn build_summary(git_root: &Path) -> Result<Option<String>> {
    let files = summary_ordered(list_files(git_root)?);
    if files.is_empty() {
        return Ok(None);
    }
    let sources = files
        .iter()
        .map(|file| {
            let name = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content =
                fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
            Ok((name, content))
        })
        .collect::<Result<Vec<_>>>()?;
    let cfg = Config::load(git_root).memory_banking;
    Ok(Some(render_summary(
        &sources,
        cfg.summary_budget,
        &cfg.skip_sections,
    )))
}

/// Write the summary file, skipping the write when the content is unchanged.
/// Returns whether the file was written.
pub fn write_summary(git_root: &Path, summary: &str) -> Result<bool> {
    let path = memory_banking_dir(git_root).join(SUMMARY_FILE);
    if fs::read_to_string(&path).is_ok_and(|existing| existing == summary) {
        return Ok(false);
    }
    fs::write(&path, summary).with_context(|| format!("writing {}", path.display()))?;
    Ok(true)
}

/// Regenerate `context-summary.md` from the other bank files.
pub fn summarize(git_root: &Path, json: bool) -> Result<()> {
    let Some(summary) = build_summary(git_root)? else {
        if json {
            println!(r#"{{"written": false, "reason": "empty"}}"#);
        } else {
            println!("[memory-banking] Nothing to summarize. Run `godmode memory-banking init`.");
        }
        return Ok(());
    };
    let written = write_summary(git_root, &summary)?;
    if json {
        println!(
            "{}",
            serde_json::json!({"written": written, "file": SUMMARY_FILE, "bytes": summary.len()})
        );
    } else {
        println!(
            "[memory-banking] {} .ctx/memory-bank/{SUMMARY_FILE} ({} bytes)",
            if written { "Wrote" } else { "Unchanged" },
            summary.len()
        );
    }
    Ok(())
}

/// Print the memory bank for context injection: the regenerated summary by
/// default, or every source file in full with `full`.
pub fn inject(git_root: &Path, json: bool, full: bool) -> Result<()> {
    let files = list_files(git_root)?;
    if files.is_empty() {
        return Ok(());
    }

    if full {
        return inject_full(&files, &stale_files(git_root)?, json);
    }

    let Some(summary) = build_summary(git_root)? else {
        return Ok(());
    };
    if let Err(e) = write_summary(git_root, &summary) {
        eprintln!("[memory-banking] could not write {SUMMARY_FILE}: {e}");
    }

    if json {
        let output = serde_json::json!({
            "mode": "summary",
            "summary": summary,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print!("{summary}");
    }
    Ok(())
}

/// Print every source file's full contents.
fn inject_full(files: &[PathBuf], stale: &[String], json: bool) -> Result<()> {
    if json {
        let mut entries = Vec::new();
        for file in files {
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
        println!("[memory-banking] Injecting project context from .ctx/memory-bank/");
        for file in files {
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
    let output = command_in(
        "git",
        &["log", "--since=2 hours ago", "--oneline"],
        git_root,
    )
    .output()?;

    let has_commits =
        output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty();

    if has_commits {
        if json {
            let msg = serde_json::json!({
                "reminder": true,
                "message": "Session had commits. Update .ctx/memory-bank/active-context.md and progress.md before ending.",
            });
            println!("{}", serde_json::to_string_pretty(&msg)?);
        } else {
            println!(
                "[memory-banking] Session had commits. Update .ctx/memory-bank/active-context.md and progress.md before ending."
            );
        }
    }

    Ok(())
}

/// Create `.ctx/memory-bank/` with empty template files.
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
        "[memory-banking] Initialized .ctx/memory-bank/ with {} template files.",
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
            "[memory-banking] {} files in .ctx/memory-bank/",
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
        TempDir::new().expect("create tempdir")
    }

    fn setup_with_memory_bank(files: &[(&str, &str)]) -> TempDir {
        let tmp = setup_fake_root();
        let mb = tmp.path().join(".ctx").join("godmode").join("memory-bank");
        fs::create_dir_all(&mb).expect("create memory-bank dir");
        for (name, content) in files {
            fs::write(mb.join(name), content).expect("write file");
        }
        tmp
    }

    #[test]
    fn memory_banking_dir_prefers_new_path() {
        let tmp = setup_fake_root();
        let new = tmp.path().join(".ctx").join("godmode").join("memory-bank");
        fs::create_dir_all(&new).unwrap();
        assert_eq!(memory_banking_dir(tmp.path()), new);
    }

    #[test]
    fn memory_banking_dir_falls_back_to_legacy() {
        let tmp = setup_fake_root();
        let legacy = tmp.path().join(".ctx").join("memory-banking");
        fs::create_dir_all(&legacy).unwrap();
        assert_eq!(memory_banking_dir(tmp.path()), legacy);
    }

    #[test]
    fn memory_banking_dir_defaults_to_new_when_neither_exists() {
        let p = Path::new("/some/project");
        assert_eq!(
            memory_banking_dir(p),
            PathBuf::from("/some/project/.ctx/memory-bank")
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
        let mb = tmp.path().join(".ctx").join("godmode").join("memory-bank");
        fs::create_dir_all(&mb).unwrap();
        assert!(!exists(tmp.path()));
    }

    #[test]
    fn exists_false_when_only_non_md_files() {
        let tmp = setup_fake_root();
        let mb = tmp.path().join(".ctx").join("godmode").join("memory-bank");
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

    fn skip_defaults() -> Vec<String> {
        vec!["Recently completed".to_string(), "What works".to_string()]
    }

    fn source(name: &str, content: &str) -> (String, String) {
        (name.to_string(), content.to_string())
    }

    const PROGRESS: &str = "# Progress\n\n## What works\n\n- shipped thing\n\n## Recently completed\n\n- old commit one\n- old commit two\n\n## In progress\n\n- current work\n\n## Known issues\n\n- open bug\n";

    #[test]
    fn summary_is_deterministic() {
        let sources = vec![source("progress.mbx.md", PROGRESS)];
        let a = render_summary(&sources, 6144, &skip_defaults());
        let b = render_summary(&sources, 6144, &skip_defaults());
        assert_eq!(a, b);
    }

    #[test]
    fn summary_skips_history_sections_and_keeps_current_ones() {
        let sources = vec![source("progress.mbx.md", PROGRESS)];
        let out = render_summary(&sources, 6144, &skip_defaults());
        assert!(!out.contains("old commit one"), "history leaked: {out}");
        assert!(
            !out.contains("shipped thing"),
            "skipped section leaked: {out}"
        );
        assert!(out.contains("### In progress"));
        assert!(out.contains("- current work"));
        assert!(out.contains("- open bug"));
        assert!(out.contains("full: progress.mbx.md ("));
    }

    #[test]
    fn summary_keeps_last_verified_line() {
        let sources = vec![source(
            "active-context.mbx.md",
            "# Active Context\n\nLast verified: 2026-11-22 against git.\n\n## Current focus\n\n- none\n",
        )];
        let out = render_summary(&sources, 6144, &skip_defaults());
        assert!(out.contains("Last verified: 2026-11-22 against git."));
    }

    #[test]
    fn summary_caps_section_lines_without_filler() {
        let body: String = (1..=40).map(|i| format!("- item {i}\n")).collect();
        let sources = vec![source(
            "tech-context.md",
            &format!("# Tech\n\n## Big\n\n{body}"),
        )];
        let out = render_summary(&sources, 400, &skip_defaults());
        assert!(!out.contains("more lines"), "filler marker: {out}");
        assert!(!out.contains("item 40"));
    }

    #[test]
    fn summary_omits_sections_with_no_body() {
        let sources = vec![source("a.md", "# A\n\n## Empty\n\n## Real\n\n- kept\n")];
        let out = render_summary(&sources, 6144, &skip_defaults());
        assert!(!out.contains("### Empty"), "{out}");
        assert!(out.contains("### Real"));
    }

    #[test]
    fn summary_finishes_a_wrapped_bullet_at_the_cap() {
        let content = "# T\n\n## S\n\n- first\n- second wraps\n  onto a continuation\n- third\n";
        let (kept, cut) = take_section_lines(&content.lines().skip(4).collect::<Vec<_>>(), 2);
        assert_eq!(
            kept,
            vec!["- first", "- second wraps", "  onto a continuation"]
        );
        assert_eq!(cut, 1);
    }

    #[test]
    fn summary_ignores_headings_inside_code_fences() {
        let content = "# T\n\n## Real\n\n```\n## not a heading\n```\n\n## Next\n\n- x\n";
        let headings: Vec<_> = split_sections(content)
            .into_iter()
            .filter_map(|(h, _)| h)
            .collect();
        assert_eq!(headings, vec!["Real", "Next"]);
    }

    #[test]
    fn summary_respects_the_byte_budget() {
        let body: String = (1..=200).map(|i| format!("- item number {i}\n")).collect();
        let sources: Vec<_> = (0..8)
            .map(|n| source(&format!("file{n}.md"), &format!("# F{n}\n\n## S\n\n{body}")))
            .collect();
        let out = render_summary(&sources, 400, &skip_defaults());
        assert!(out.len() <= 400, "{} bytes", out.len());
        assert!(out.contains("## F0"), "first source kept whole: {out}");
        assert!(out.contains("omitted: "), "dropped files are named: {out}");
        assert!(out.contains("file7.md"));
        assert!(!out.contains("[truncated"), "no fragments: {out}");
    }

    #[test]
    fn summary_cuts_only_when_the_first_source_alone_overflows() {
        let body: String = (1..=50).map(|i| format!("- item number {i}\n")).collect();
        let sources = vec![source("a.md", &format!("# A\n\n## S\n\n{body}"))];
        let out = render_summary(&sources, 60, &skip_defaults());
        assert!(out.len() <= 60, "{} bytes", out.len());
        assert!(out.ends_with("[truncated: see full files]\n"));
    }

    #[test]
    fn summary_truncation_never_splits_a_multibyte_char() {
        let body: String = (1..=50)
            .map(|i| format!("- caf\u{e9} \u{2014} item {i}\n"))
            .collect();
        let sources = vec![source("a.md", &format!("# A\n\n## S\n\n{body}"))];
        for budget in 40..120 {
            let out = render_summary(&sources, budget, &skip_defaults());
            assert!(out.len() <= budget, "{} bytes over {budget}", out.len());
        }
    }

    #[test]
    fn summary_orders_known_stems_first() {
        let files = vec![
            PathBuf::from("zeta.md"),
            PathBuf::from("progress.mbx.md"),
            PathBuf::from("project-brief.mbx.md"),
            PathBuf::from("alpha.md"),
        ];
        let names: Vec<_> = summary_ordered(files)
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        assert_eq!(
            names,
            vec![
                "progress.mbx.md",
                "project-brief.mbx.md",
                "alpha.md",
                "zeta.md"
            ]
        );
    }

    #[test]
    fn list_files_excludes_the_generated_summary() {
        let tmp =
            setup_with_memory_bank(&[("progress.md", "# p"), (SUMMARY_FILE, "# Context summary")]);
        let names: Vec<_> = list_files(tmp.path())
            .unwrap()
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(str::to_string))
            .collect();
        assert_eq!(names, vec!["progress.md"]);
    }

    #[test]
    fn summary_never_feeds_on_itself() {
        let tmp = setup_with_memory_bank(&[("progress.md", "# P\n\n## In progress\n\n- real\n")]);
        let first = build_summary(tmp.path()).unwrap().unwrap();
        write_summary(tmp.path(), &first).unwrap();
        let second = build_summary(tmp.path()).unwrap().unwrap();
        assert_eq!(first, second);
        assert!(!second.contains("Context summary\n\n## Context summary"));
    }

    #[test]
    fn write_summary_skips_unchanged_content() {
        let tmp = setup_with_memory_bank(&[("progress.md", "# p")]);
        assert!(write_summary(tmp.path(), "same").unwrap());
        assert!(!write_summary(tmp.path(), "same").unwrap());
        assert!(write_summary(tmp.path(), "different").unwrap());
    }

    /// Regression: staleness compares the memory-bank files of `git_root`
    /// against `git_root`'s own commit time, not against the commit time of
    /// the repository an ambient `GIT_DIR` points at.
    #[test]
    fn stale_files_uses_the_given_roots_commit_time_under_a_pinned_git_dir() {
        use crate::git_harness::{GitRepo, PinnedGitDir};

        let fixture = GitRepo::init();
        fixture.commit("fixture commit", Some("2020-01-01T00:00:00+00:00"));
        let bank = fixture.path().join(".ctx/memory-bank");
        fs::create_dir_all(&bank).expect("create memory bank");
        let file = bank.join("active-context.md");
        fs::write(&file, "# Active context\n").expect("write memory bank file");
        // 2010: older than the fixture's 2020 commit, so the file is stale.
        filetime::set_file_mtime(&file, filetime::FileTime::from_unix_time(1_262_304_000, 0))
            .expect("age the memory-bank file");

        // The decoy's only commit is older than the file, so a helper that
        // reads it concludes nothing is stale.
        let decoy = GitRepo::init();
        decoy.commit("decoy commit", Some("2005-01-01T00:00:00+00:00"));

        let _pin = PinnedGitDir::pointing_at(&decoy);

        assert_eq!(
            stale_files(fixture.path()).expect("stale files"),
            vec!["active-context.md".to_string()]
        );
    }
}
