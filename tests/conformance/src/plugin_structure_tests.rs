//! Conformance tests for plugin structure — ports the 14 checks from
//! `tests/conformance/plugin-structure.nu` into typed Rust tests.
//!
//! Run from repo root (CARGO_MANIFEST_DIR set by cargo).

use std::path::{Path, PathBuf};

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to tests/conformance/ — go up two levels.
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest)
        .to_path_buf()
}

fn skill_dirs(root: &Path) -> Vec<PathBuf> {
    let skills = root.join("skills");
    let Ok(entries) = std::fs::read_dir(&skills) else {
        return vec![];
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.file_name() != "_lib")
        .map(|e| e.path())
        .collect()
}

fn extract_fm_name(content: &str) -> Option<String> {
    let mut in_fm = false;
    let mut past_first = false;
    for line in content.lines() {
        if line == "---" {
            if !past_first {
                in_fm = true;
                past_first = true;
            } else {
                break;
            }
        } else if in_fm && line.starts_with("name:") {
            let val = line["name:".len()..].trim().trim_matches('"').to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

// ── Check 1: every skill dir has a SKILL.md ──────────────────────────────

pub struct EverySkillHasSkillMd;
impl ConformanceTest for EverySkillHasSkillMd {
    fn name(&self) -> &str {
        "every_skill_has_skill_md"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                let name = dir.file_name().unwrap_or_default().to_string_lossy();
                ctx.fail(&format!("[{}] missing SKILL.md", name));
            }
        }
        ctx.result()
    }
}

// ── Check 2: frontmatter name field is present ────────────────────────────

pub struct SkillMdHasFrontmatterName;
impl ConformanceTest for SkillMdHasFrontmatterName {
    fn name(&self) -> &str {
        "skill_md_has_frontmatter_name"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if extract_fm_name(&content).is_none() {
                ctx.fail(&format!("[{}] missing frontmatter name:", name));
            }
        }
        ctx.result()
    }
}

// ── Check 3: skill names appear in skill-index.md and using-godmode ───────

pub struct SkillNamesInIndex;
impl ConformanceTest for SkillNamesInIndex {
    fn name(&self) -> &str {
        "skill_names_in_index"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        let index_path = root
            .join("skills")
            .join("using-godmode")
            .join("references")
            .join("skill-index.md");
        let using_path = root.join("skills").join("using-godmode").join("SKILL.md");
        if !index_path.exists() || !using_path.exists() {
            return TestResult::Skipped {
                reason: "skill-index.md or using-godmode SKILL.md not found".into(),
            };
        }
        let index = std::fs::read_to_string(&index_path).unwrap_or_default();
        let using = std::fs::read_to_string(&using_path).unwrap_or_default();

        for dir in skill_dirs(&root) {
            let skill_name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if skill_name == "using-godmode" {
                continue;
            }
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let Some(full_name) = extract_fm_name(&content) else {
                continue;
            };
            if !index.contains(&full_name) {
                ctx.fail(&format!(
                    "[{}] name '{}' not found in skill-index.md",
                    skill_name, full_name
                ));
            }
            if !using.contains(&full_name) {
                ctx.fail(&format!(
                    "[{}] name '{}' not found in using-godmode/SKILL.md",
                    skill_name, full_name
                ));
            }
        }
        ctx.result()
    }
}

// ── Check 4: no orphan index entries ─────────────────────────────────────

pub struct NoOrphanIndexEntries;
impl ConformanceTest for NoOrphanIndexEntries {
    fn name(&self) -> &str {
        "no_orphan_index_entries"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        let index_path = root
            .join("skills")
            .join("using-godmode")
            .join("references")
            .join("skill-index.md");
        if !index_path.exists() {
            return TestResult::Skipped {
                reason: "skill-index.md not found".into(),
            };
        }
        let index = std::fs::read_to_string(&index_path).unwrap_or_default();
        let dirs: Vec<String> = skill_dirs(&root)
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();

        // Extract `godmode:XXX` entries (skip fenced code block lines)
        let re = regex_lite(r"`(godmode:[^`]+)`");
        for line in non_fence_lines(&index) {
            for cap in re.captures_iter(line) {
                let full = &cap[1];
                let short = full.trim_start_matches("godmode:");
                if !dirs.iter().any(|d| d == short) {
                    ctx.fail(&format!(
                        "[index] orphan entry '{}' — no skills/{}/",
                        full, short
                    ));
                }
            }
        }
        ctx.result()
    }
}

// ── Check 5 & 6: references/ and helpers/ links resolve ──────────────────

pub struct ReferencesLinksResolve;
impl ConformanceTest for ReferencesLinksResolve {
    fn name(&self) -> &str {
        "references_links_resolve"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        check_link_pattern(&root, r"`(references/[^`]+)`", ctx);
        ctx.result()
    }
}

pub struct HelpersLinksResolve;
impl ConformanceTest for HelpersLinksResolve {
    fn name(&self) -> &str {
        "helpers_links_resolve"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        check_link_pattern(&root, r"`(helpers/[^`]+)`", ctx);
        ctx.result()
    }
}

fn check_link_pattern(root: &Path, pattern: &str, ctx: &mut TestContext) {
    let re = regex_lite(pattern);
    for dir in skill_dirs(root) {
        let skill_md = dir.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
        let skill_name = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        for line in non_fence_lines(&content) {
            for cap in re.captures_iter(line) {
                let link = &cap[1];
                // Skip bare directory references like "references/" with no filename
                if link.ends_with('/') {
                    continue;
                }
                let resolved = dir.join(link);
                if !resolved.exists() {
                    ctx.fail(&format!("[{}] broken link: {}", skill_name, link));
                }
            }
        }
    }
}

// ── Check 7: plugin.json allowed fields only ─────────────────────────────

pub struct PluginJsonAllowedFields;
impl ConformanceTest for PluginJsonAllowedFields {
    fn name(&self) -> &str {
        "plugin_json_allowed_fields"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        let path = root.join(".claude-plugin").join("plugin.json");
        if !path.exists() {
            ctx.fail("[plugin.json] file not found at .claude-plugin/plugin.json");
            return ctx.result();
        }
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        let val: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                ctx.fail(&format!("parse error: {}", e));
                return ctx.result();
            }
        };
        let allowed = ["name", "version", "author", "description"];
        if let Some(obj) = val.as_object() {
            for key in obj.keys() {
                if !allowed.contains(&key.as_str()) {
                    ctx.fail(&format!("[plugin.json] disallowed field: {}", key));
                }
            }
        }
        ctx.result()
    }
}

// ── Check 8: CLI subcommand conformance ──────────────────────────────────

pub struct CliSubcommandConformance;
impl ConformanceTest for CliSubcommandConformance {
    fn name(&self) -> &str {
        "cli_subcommand_conformance"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let canonical = &[
            "handon",
            "handoff",
            "status",
            "task list",
            "task next",
            "task add",
            "task start",
            "task done",
            "task block",
            "task unblock",
            "task unblock-all",
            "task run",
            "task remove",
            "task clear",
            "task pull",
            "task push-done",
            "task apply",
            "task list-templates",
            "plan ingest",
            "dispatch",
            "agent",
            "verify",
            "wave init",
            "wave status",
            "wave done",
            "wave block",
            "wave check",
            "worktree add",
            "worktree remove",
            "ci triage",
            "issue list",
            "issue close",
            "graph build",
            "review self",
            "review skills",
            "review agents",
            "release current",
            "release bump",
            "release tag",
            "release push",
        ];
        let root = repo_root();
        let re_cmd = regex_lite(r"^\s*godmode\s+\S");
        let re_skip = regex_lite(r"^[-<$#]");

        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let skill_name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let prose_lines: Vec<&str> = non_fence_lines(&content);
            for (lineno, line) in prose_lines.into_iter().enumerate() {
                if !re_cmd.is_match(line) {
                    continue;
                }
                let trimmed = line.trim();
                let parts: Vec<&str> = trimmed
                    .split_whitespace()
                    .skip(1)
                    .filter(|p| !p.is_empty())
                    .collect();
                if parts.is_empty() {
                    continue;
                }
                let first = parts[0];
                if re_skip.is_match(first) {
                    continue;
                }

                let two = parts.get(1).and_then(|s| {
                    if !re_skip.is_match(s) {
                        Some(format!("{} {}", first, s))
                    } else {
                        None
                    }
                });

                let matched = two
                    .as_deref()
                    .map(|t| canonical.contains(&t))
                    .unwrap_or(false)
                    || canonical.contains(&first);

                if !matched {
                    let shown = two.as_deref().unwrap_or(first);
                    ctx.fail(&format!(
                        "[{}:{}] unknown subcommand: godmode {}",
                        skill_name,
                        lineno + 1,
                        shown
                    ));
                }
            }
        }
        ctx.result()
    }
}

// ── Checks 9–12: cross-skill consistency ─────────────────────────────────

pub struct MergeStrategyNoFf;
impl ConformanceTest for MergeStrategyNoFf {
    fn name(&self) -> &str {
        "merge_strategy_no_ff"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if content.contains("git merge") && !content.contains("--no-ff") {
                ctx.fail(&format!("[{}] has 'git merge' but not '--no-ff'", name));
            }
        }
        ctx.result()
    }
}

pub struct ConcurrencyCapFive;
impl ConformanceTest for ConcurrencyCapFive {
    fn name(&self) -> &str {
        "concurrency_cap_five"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        let re = regex_lite(r"(\d+)\s+concurrent");
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            for line in non_fence_lines(&content) {
                // Only flag lines that explicitly state a cap/limit — not
                // incidental mentions like "2 concurrent sessions".
                let lower = line.to_ascii_lowercase();
                if !lower.contains("cap") && !lower.contains("limit") && !lower.contains("max") {
                    continue;
                }
                for cap in re.captures_iter(line) {
                    if &cap[1] != "5" {
                        ctx.fail(&format!(
                            "[{}] concurrency cap is {}, expected 5",
                            name, &cap[1]
                        ));
                    }
                }
            }
        }
        ctx.result()
    }
}

pub struct BlockedThreshold;
impl ConformanceTest for BlockedThreshold {
    fn name(&self) -> &str {
        "blocked_threshold_three"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        let re = regex_lite(r"(\d+)\s+(attempt|tries|retry|retries|failed)");
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            for line in content.lines() {
                if !line.contains("BLOCKED") {
                    continue;
                }
                for cap in re.captures_iter(line) {
                    if &cap[1] != "3" {
                        ctx.fail(&format!(
                            "[{}] BLOCKED.md threshold is {}, expected 3",
                            name, &cap[1]
                        ));
                    }
                }
            }
        }
        ctx.result()
    }
}

pub struct BranchGuard;
impl ConformanceTest for BranchGuard {
    fn name(&self) -> &str {
        "branch_guard_before_commit"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if content.contains("git commit") && !content.contains("git branch --show-current") {
                ctx.fail(&format!(
                    "[{}] has 'git commit' but no 'git branch --show-current'",
                    name
                ));
            }
        }
        ctx.result()
    }
}

// ── Check 14: _lib/ references in SKILL.md resolve ───────────────────────

pub struct LibReferencesResolve;
impl ConformanceTest for LibReferencesResolve {
    fn name(&self) -> &str {
        "lib_references_resolve"
    }
    fn crate_name(&self) -> &str {
        "plugin_structure"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let root = repo_root();
        let re = regex_lite(r"`(skills/_lib/[^`]+\.nu)[^`]*`");
        for dir in skill_dirs(&root) {
            let skill_md = dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let skill_name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            for cap in re.captures_iter(&content) {
                let link = &cap[1];
                let resolved = root.join(link);
                if !resolved.exists() {
                    ctx.fail(&format!("[{}] broken _lib link: {}", skill_name, link));
                }
            }
        }
        ctx.result()
    }
}

/// Returns lines from `text` that are NOT inside fenced code blocks (``` or ~~~).
fn non_fence_lines(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue; // skip fence delimiters themselves
        }
        if !in_fence {
            result.push(line);
        }
    }
    result
}

// ── Minimal regex helper (no external dep) ────────────────────────────────

struct Regex {
    pattern: String,
}

struct Captures<'a> {
    full: &'a str,
    groups: Vec<&'a str>,
}

impl<'a> std::ops::Index<usize> for Captures<'a> {
    type Output = str;
    fn index(&self, i: usize) -> &str {
        if i == 0 {
            self.full
        } else {
            self.groups[i - 1]
        }
    }
}

impl Regex {
    fn is_match(&self, text: &str) -> bool {
        !self.captures_iter(text).is_empty()
    }

    fn captures_iter<'a>(&self, text: &'a str) -> Vec<Captures<'a>> {
        // Use std::str::pattern is nightly-only; use a simple hand-rolled approach
        // for the specific patterns used in this file.
        // Delegate to the stdlib regex we simulate via string search.
        captures_all(&self.pattern, text)
    }
}

fn regex_lite(pattern: &str) -> Regex {
    Regex {
        pattern: pattern.to_string(),
    }
}

// Hand-rolled captures for the exact patterns used above.
// Patterns: backtick-delimited captures and digit+word captures.
fn captures_all<'a>(pattern: &str, text: &'a str) -> Vec<Captures<'a>> {
    let mut results = Vec::new();
    // Pattern families:
    // 1. "`(godmode:[^`]+)`"  -> find `godmode:...` in backticks
    // 2. "`(references/[^`]+)`" -> find `references/...` in backticks
    // 3. "`(helpers/[^`]+)`"
    // 4. "`(skills/_lib/[^`]+\.nu)[^`]*`"
    // 5. r"(\d+)\s+concurrent"
    // 6. r"(\d+)\s+(attempt|tries|retry|retries|failed)"
    // 7. r"^\s*godmode\s+\S"
    // 8. r"^[-<$#]"

    if pattern.starts_with('`') && pattern.ends_with('`') {
        // backtick-delimited capture: e.g. `(godmode:[^`]+)` or `(references/[^`]+)`
        // Extract the literal prefix INSIDE the capture group (before any special chars).
        let inner = &pattern[1..pattern.len() - 1]; // strip outer backticks
        // inner looks like `(PREFIX[^`]+)` or `(PREFIX[^`]+)[^`]*`
        // Find the capture group content between first ( and last )
        let cap_start = inner.find('(').unwrap_or(0) + 1;
        let cap_end = inner.rfind(')').unwrap_or(inner.len());
        let cap_inner = &inner[cap_start..cap_end]; // e.g. "godmode:[^`]+" or "references/[^`]+"
        // Literal prefix is everything before the first regex special char
        let prefix_end = cap_inner
            .find(|c: char| "[.*+?\\".contains(c))
            .unwrap_or(cap_inner.len());
        let prefix = &cap_inner[..prefix_end];
        // Suffix outside the capture group (after last `)`)
        let suffix = if cap_end + 1 < inner.len() {
            &inner[cap_end + 1..]
        } else {
            ""
        };

        let mut search = text;
        let mut offset = 0;
        while let Some(bt_start) = search.find('`') {
            let rest = &search[bt_start + 1..];
            if let Some(bt_end) = rest.find('`') {
                let inner_text = &rest[..bt_end];
                if inner_text.starts_with(prefix) {
                    let cap_text = if suffix.is_empty() {
                        inner_text
                    } else {
                        inner_text.trim_end_matches(suffix)
                    };
                    let full_start = offset + bt_start;
                    let full_end = full_start + 1 + bt_end + 1;
                    let full = &text[full_start..full_end];
                    results.push(Captures {
                        full,
                        groups: vec![cap_text],
                    });
                }
                offset += bt_start + 1 + bt_end + 1;
                search = &rest[bt_end + 1..];
            } else {
                break;
            }
        }
    } else if pattern.contains(r"\d+") && pattern.contains("concurrent") {
        // (\d+)\s+concurrent
        for m in find_digit_word_pairs(text, "concurrent") {
            results.push(Captures {
                full: m.0,
                groups: vec![m.1],
            });
        }
    } else if pattern.contains(r"\d+") && pattern.contains("attempt") {
        // (\d+)\s+(attempt|tries|...)
        for word in &["attempt", "tries", "retry", "retries", "failed"] {
            for m in find_digit_word_pairs(text, word) {
                results.push(Captures {
                    full: m.0,
                    groups: vec![m.1, word],
                });
            }
        }
    } else if pattern.starts_with('^') {
        // line-start patterns — handled by is_match only
        if pattern == r"^\s*godmode\s+\S" {
            let trimmed = text.trim_start();
            if let Some(after) = trimmed.strip_prefix("godmode")
                && !after.trim_start().is_empty()
            {
                results.push(Captures {
                    full: text,
                    groups: vec![],
                });
            }
        } else if pattern == r"^[-<$#]"
            && text
                .chars()
                .next()
                .is_some_and(|c| matches!(c, '-' | '<' | '$' | '#'))
        {
            results.push(Captures {
                full: text,
                groups: vec![],
            });
        }
    }
    results
}

fn find_digit_word_pairs<'a>(text: &'a str, word: &str) -> Vec<(&'a str, &'a str)> {
    let mut results = Vec::new();
    let mut search = text;
    let mut base = 0;
    while let Some(pos) = search.find(word) {
        // look backwards for a digit sequence
        let before = &search[..pos].trim_end();
        if let Some(digit_end) = before.rfind(|c: char| c.is_ascii_digit()) {
            let digit_start = before[..=digit_end]
                .rfind(|c: char| !c.is_ascii_digit())
                .map(|i| i + 1)
                .unwrap_or(0);
            let _digit_str = &before[digit_start..=digit_end];
            let full_start = base + digit_start;
            let full_end = base + pos + word.len();
            results.push((
                &text[full_start..full_end],
                &text[base + digit_start..base + digit_end + 1],
            ));
        }
        base += pos + word.len();
        search = &search[pos + word.len()..];
    }
    results
}

pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(EverySkillHasSkillMd),
        Box::new(SkillMdHasFrontmatterName),
        Box::new(SkillNamesInIndex),
        Box::new(NoOrphanIndexEntries),
        Box::new(ReferencesLinksResolve),
        Box::new(HelpersLinksResolve),
        Box::new(PluginJsonAllowedFields),
        Box::new(CliSubcommandConformance),
        Box::new(MergeStrategyNoFf),
        Box::new(ConcurrencyCapFive),
        Box::new(BlockedThreshold),
        Box::new(BranchGuard),
        Box::new(LibReferencesResolve),
    ]
}
