use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

#[derive(Debug, serde::Serialize)]
pub struct Finding {
    pub skill: String,
    pub check: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ReviewReport {
    pub checks: u32,
    pub findings: Vec<Finding>,
    pub passed: bool,
}

impl ReviewReport {
    fn new() -> Self {
        Self {
            checks: 0,
            findings: vec![],
            passed: true,
        }
    }

    fn fail(&mut self, skill: impl Into<String>, check: impl Into<String>, msg: impl Into<String>) {
        self.findings.push(Finding {
            skill: skill.into(),
            check: check.into(),
            message: msg.into(),
        });
        self.passed = false;
    }

    fn merge(&mut self, other: ReviewReport) {
        self.checks += other.checks;
        self.findings.extend(other.findings);
        if !other.passed {
            self.passed = false;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skills_dir(root: &Path) -> PathBuf {
    root.join("skills")
}

fn agents_dir(root: &Path) -> PathBuf {
    root.join("agents")
}

/// Return all skill dirs excluding `_lib`.
fn skill_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = skills_dir(root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            if name != "_lib" {
                out.push(entry.path());
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Return all agent `.md` files in `agents/`.
fn agent_files(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = agents_dir(root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Extract frontmatter `name:` value from SKILL.md content.
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
        } else if in_fm && let Some(rest) = line.strip_prefix("name:") {
            let val = rest.trim().trim_matches('"').trim().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

/// Extract frontmatter field value.
fn extract_fm_field<'a>(content: &'a str, field: &str) -> Option<&'a str> {
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
        } else if in_fm {
            let prefix = format!("{field}:");
            if let Some(rest) = line.strip_prefix(&prefix) {
                return Some(rest.trim());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run all conformance checks (equivalent to `just conformance`).
pub fn run_all(root: &Path) -> Result<ReviewReport> {
    let mut report = ReviewReport::new();
    report.merge(check_skills(root)?);
    report.merge(check_agents(root)?);
    report.merge(check_plugin_json(root)?);
    report.merge(check_lib(root)?);
    Ok(report)
}

/// Check skill dirs: SKILL.md present, frontmatter name, index/using-godmode entries,
/// link resolution, CLI subcommand validity, cross-skill consistency.
pub fn check_skills(root: &Path) -> Result<ReviewReport> {
    let mut r = ReviewReport::new();
    let dirs = skill_dirs(root)?;

    let index_path = root.join("skills/using-godmode/references/skill-index.md");
    let using_path = root.join("skills/using-godmode/SKILL.md");
    let index_content = fs::read_to_string(&index_path).unwrap_or_default();
    let using_content = fs::read_to_string(&using_path).unwrap_or_default();

    // Collect valid skill names for orphan check
    let mut known_names: Vec<String> = vec![];

    for dir in &dirs {
        let skill_name = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let skill_md = dir.join("SKILL.md");

        // Check 1: SKILL.md exists
        r.checks += 1;
        if !skill_md.exists() {
            r.fail(
                &skill_name,
                "missing SKILL.md",
                format!("[{skill_name}] missing SKILL.md"),
            );
            continue;
        }

        let content = fs::read_to_string(&skill_md)?;

        // Check 2: frontmatter name present
        r.checks += 1;
        let fm_name = extract_fm_name(&content);
        if fm_name.is_none() {
            r.fail(
                &skill_name,
                "missing frontmatter name",
                format!("[{skill_name}] missing or empty frontmatter name:"),
            );
        }

        if let Some(ref full_name) = fm_name {
            known_names.push(full_name.clone());

            // Check 3: name in skill-index.md (skip using-godmode itself)
            if skill_name != "using-godmode" {
                r.checks += 1;
                if !index_content.contains(full_name.as_str()) {
                    r.fail(
                        &skill_name,
                        "not in skill-index.md",
                        format!("[{skill_name}] name '{full_name}' not found in skill-index.md"),
                    );
                }

                // Check 4: name in using-godmode/SKILL.md
                r.checks += 1;
                if !using_content.contains(full_name.as_str()) {
                    r.fail(
                        &skill_name,
                        "not in using-godmode SKILL.md",
                        format!(
                            "[{skill_name}] name '{full_name}' not found in using-godmode/SKILL.md"
                        ),
                    );
                }
            }
        }

        // Check 5: references/ links resolve
        for line in content.lines() {
            if let Some(cap) = extract_backtick_path(line, "references/") {
                r.checks += 1;
                let resolved = dir.join(&cap);
                if !resolved.exists() {
                    r.fail(
                        &skill_name,
                        "broken references link",
                        format!("[{skill_name}] broken references link: {cap}"),
                    );
                }
            }
        }

        // Check 6: helpers/ links resolve
        for line in content.lines() {
            if let Some(cap) = extract_backtick_path(line, "helpers/") {
                r.checks += 1;
                let resolved = dir.join(&cap);
                if !resolved.exists() {
                    r.fail(
                        &skill_name,
                        "broken helpers link",
                        format!("[{skill_name}] broken helpers link: {cap}"),
                    );
                }
            }
        }

        // Check 7: CLI subcommand validity
        for (lineno, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("godmode ") {
                continue;
            }
            let parts: Vec<&str> = trimmed
                .split_whitespace()
                .skip(1)
                .filter(|p| !p.is_empty())
                .collect();
            if parts.is_empty() {
                continue;
            }
            let first = parts[0];
            if first.starts_with('-')
                || first.starts_with('<')
                || first.starts_with('$')
                || first.starts_with('#')
            {
                continue;
            }
            let two = if parts.len() >= 2 {
                let second = parts[1];
                if !second.starts_with('-')
                    && !second.starts_with('<')
                    && !second.starts_with('$')
                    && !second.starts_with('[')
                {
                    format!("{first} {second}")
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            r.checks += 1;
            let matched = if !two.is_empty() {
                CANONICAL_SUBCOMMANDS.contains(&two.as_str())
                    || CANONICAL_SUBCOMMANDS.contains(&first)
            } else {
                CANONICAL_SUBCOMMANDS.contains(&first)
            };
            if !matched {
                let shown = if !two.is_empty() { &two } else { first };
                r.fail(
                    &skill_name,
                    "unknown subcommand",
                    format!(
                        "[{skill_name}:{}] unknown subcommand: godmode {shown}",
                        lineno + 1
                    ),
                );
            }
        }

        // Check 8: merge strategy — git merge must use --no-ff
        r.checks += 1;
        if content.contains("git merge") && !content.contains("--no-ff") {
            r.fail(
                &skill_name,
                "merge strategy",
                format!(
                    "[{skill_name}] consistency violation: merge strategy — mentions 'git merge' \
                     but not '--no-ff'"
                ),
            );
        }

        // Check 9: branch guard — git commit requires git branch --show-current
        r.checks += 1;
        if content.contains("git commit") && !content.contains("git branch --show-current") {
            r.fail(
                &skill_name,
                "branch guard",
                format!(
                    "[{skill_name}] consistency violation: branch guard — has 'git commit' but no \
                     'git branch --show-current' check"
                ),
            );
        }

        // Check 10: _lib/ references resolve
        for line in content.lines() {
            if let Some(cap) = extract_backtick_path(line, "skills/_lib/") {
                r.checks += 1;
                let resolved = root.join(&cap);
                if !resolved.exists() {
                    r.fail(
                        &skill_name,
                        "broken _lib link",
                        format!("[{skill_name}] broken _lib link: {cap}"),
                    );
                }
            }
        }
    }

    // Check 11: orphan index entries
    for entry_name in extract_godmode_names_from_index(&index_content) {
        r.checks += 1;
        let short = entry_name.trim_start_matches("godmode:");
        let exists = dirs
            .iter()
            .any(|d| d.file_name().unwrap_or_default().to_string_lossy() == short);
        if !exists {
            r.fail(
                "index",
                "orphan entry",
                format!("[index] orphan entry '{entry_name}' — no matching skills/{short}/ dir"),
            );
        }
    }

    Ok(r)
}

/// Check agent frontmatter: name, model, tools fields all present and non-empty.
pub fn check_agents(root: &Path) -> Result<ReviewReport> {
    let mut r = ReviewReport::new();
    let files = agent_files(root)?;

    for path in &files {
        let agent_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let content = fs::read_to_string(path)?;

        for field in ["name", "model", "tools"] {
            r.checks += 1;
            match extract_fm_field(&content, field) {
                None => {
                    r.fail(
                        &agent_name,
                        format!("missing {field}"),
                        format!("[{agent_name}] agent missing frontmatter field: {field}"),
                    );
                }
                Some("") => {
                    r.fail(
                        &agent_name,
                        format!("empty {field}"),
                        format!("[{agent_name}] agent frontmatter field empty: {field}"),
                    );
                }
                _ => {}
            }
        }
    }

    Ok(r)
}

/// Check plugin.json allowed fields and _lib/*.nu parse.
pub fn check_plugin_json(root: &Path) -> Result<ReviewReport> {
    let mut r = ReviewReport::new();
    let plugin_path = root.join(".claude-plugin/plugin.json");

    r.checks += 1;
    if !plugin_path.exists() {
        r.fail(
            "plugin.json",
            "file not found",
            "[plugin.json] file not found at .claude-plugin/plugin.json",
        );
        return Ok(r);
    }

    let content = fs::read_to_string(&plugin_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let allowed = ["name", "version", "author", "description"];
    if let Some(obj) = json.as_object() {
        for key in obj.keys() {
            r.checks += 1;
            if !allowed.contains(&key.as_str()) {
                r.fail(
                    "plugin.json",
                    "disallowed field",
                    format!("[plugin.json] disallowed field: {key}"),
                );
            }
        }
        if let Some(author) = obj.get("author").and_then(|a| a.as_object()) {
            for key in author.keys() {
                r.checks += 1;
                if key != "name" {
                    r.fail(
                        "plugin.json",
                        "disallowed author field",
                        format!("[plugin.json] disallowed author field: {key}"),
                    );
                }
            }
        }
    }

    Ok(r)
}

/// Check _lib/*.nu files parse without error.
pub fn check_lib(root: &Path) -> Result<ReviewReport> {
    let mut r = ReviewReport::new();
    let lib_dir = root.join("skills/_lib");
    if !lib_dir.exists() {
        return Ok(r);
    }

    for entry in fs::read_dir(&lib_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("nu") {
            continue;
        }
        let lib_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        r.checks += 1;

        let out = Command::new("nu")
            .args(["-c", &format!("use {}", path.display())])
            .output();

        match out {
            Err(_) => {} // nu not available — skip
            Ok(o) if !o.status.success() => {
                let err = String::from_utf8_lossy(&o.stderr);
                let first_line = err.lines().next().unwrap_or("").trim().to_string();
                r.fail(
                    format!("_lib/{lib_name}"),
                    "parse error",
                    format!("[_lib/{lib_name}] parse error: {first_line}"),
                );
            }
            _ => {}
        }
    }

    Ok(r)
}

// ---------------------------------------------------------------------------
// Internal utilities
// ---------------------------------------------------------------------------

const CANONICAL_SUBCOMMANDS: &[&str] = &[
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

/// Extract a backtick-quoted path that starts with the given prefix from a line.
fn extract_backtick_path(line: &str, prefix: &str) -> Option<String> {
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('`') {
            let inner = &rest[..end];
            if inner.starts_with(prefix) {
                return Some(inner.to_string());
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    None
}

/// Extract all `godmode:*` names from the skill-index content.
fn extract_godmode_names_from_index(content: &str) -> Vec<String> {
    let mut names = vec![];
    for line in content.lines() {
        if !line.contains('`') {
            continue;
        }
        let mut rest = line;
        while let Some(start) = rest.find('`') {
            rest = &rest[start + 1..];
            if let Some(end) = rest.find('`') {
                let inner = &rest[..end];
                if inner.starts_with("godmode:") && !names.contains(&inner.to_string()) {
                    names.push(inner.to_string());
                }
                rest = &rest[end + 1..];
            } else {
                break;
            }
        }
    }
    names
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_skill(dir: &Path, name: &str, fm_name: &str) {
        let skill_dir = dir.join("skills").join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: \"{fm_name}\"\n---\n\nContent.\n"),
        )
        .unwrap();
    }

    fn make_index(dir: &Path, entries: &[&str]) {
        let ref_dir = dir.join("skills/using-godmode/references");
        fs::create_dir_all(&ref_dir).unwrap();
        let lines: String = entries.iter().map(|e| format!("`{e}`\n")).collect();
        fs::write(ref_dir.join("skill-index.md"), lines).unwrap();
    }

    fn make_using_godmode(dir: &Path, entries: &[&str]) {
        let skill_dir = dir.join("skills/using-godmode");
        fs::create_dir_all(&skill_dir).unwrap();
        let body: String = entries.iter().map(|e| format!("{e}\n")).collect();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: \"godmode:using-godmode\"\n---\n\n{body}"),
        )
        .unwrap();
    }

    fn minimal_fixture() -> TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        make_skill(root, "brainstorm", "godmode:brainstorm");
        make_index(root, &["godmode:brainstorm"]);
        make_using_godmode(root, &["godmode:brainstorm"]);
        tmp
    }

    #[test]
    fn all_checks_pass_on_clean_fixture() {
        let tmp = minimal_fixture();
        let r = check_skills(tmp.path()).unwrap();
        assert!(r.passed, "findings: {:?}", r.findings);
    }

    #[test]
    fn detects_missing_skill_md() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("skills/empty-skill")).unwrap();
        make_index(root, &[]);
        make_using_godmode(root, &[]);
        let r = check_skills(root).unwrap();
        assert!(!r.passed);
        assert!(r.findings.iter().any(|f| f.check == "missing SKILL.md"));
    }

    #[test]
    fn detects_missing_frontmatter_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let skill_dir = root.join("skills/no-name");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\n---\n\nContent.\n").unwrap();
        make_index(root, &[]);
        make_using_godmode(root, &[]);
        let r = check_skills(root).unwrap();
        assert!(!r.passed);
        assert!(
            r.findings
                .iter()
                .any(|f| f.check == "missing frontmatter name")
        );
    }

    #[test]
    fn detects_orphan_index_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // index references a skill dir that doesn't exist
        make_index(root, &["godmode:ghost"]);
        make_using_godmode(root, &[]);
        let r = check_skills(root).unwrap();
        assert!(!r.passed);
        assert!(r.findings.iter().any(|f| f.check == "orphan entry"));
    }

    #[test]
    fn detects_broken_references_link() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let skill_dir = root.join("skills/miskill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: \"godmode:miskill\"\n---\n\nSee `references/missing.md`.\n",
        )
        .unwrap();
        make_index(root, &["godmode:miskill"]);
        make_using_godmode(root, &["godmode:miskill"]);
        let r = check_skills(root).unwrap();
        assert!(!r.passed);
        assert!(
            r.findings
                .iter()
                .any(|f| f.check == "broken references link")
        );
    }

    #[test]
    fn detects_agent_missing_model_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let agents_dir = root.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        // name and tools present, model missing
        fs::write(
            agents_dir.join("my-agent.md"),
            "---\nname: my-agent\ntools: [Read]\n---\n\nBody.\n",
        )
        .unwrap();
        let r = check_agents(root).unwrap();
        assert!(!r.passed);
        assert!(r.findings.iter().any(|f| f.check == "missing model"));
    }

    #[test]
    fn extract_fm_name_parses_quoted_and_unquoted() {
        assert_eq!(
            extract_fm_name("---\nname: \"godmode:foo\"\n---\n"),
            Some("godmode:foo".to_string())
        );
        assert_eq!(
            extract_fm_name("---\nname: godmode:bar\n---\n"),
            Some("godmode:bar".to_string())
        );
        assert_eq!(extract_fm_name("---\n---\n"), None);
    }

    #[test]
    fn plugin_json_check_passes_on_valid_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".claude-plugin")).unwrap();
        fs::write(
            root.join(".claude-plugin/plugin.json"),
            r#"{"name":"godmode","version":"1.0.0","author":{"name":"Joe"},"description":"x"}"#,
        )
        .unwrap();
        let r = check_plugin_json(root).unwrap();
        assert!(r.passed, "findings: {:?}", r.findings);
    }

    #[test]
    fn plugin_json_check_fails_on_extra_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".claude-plugin")).unwrap();
        fs::write(
            root.join(".claude-plugin/plugin.json"),
            r#"{"name":"x","version":"1.0.0","author":{"name":"Joe"},"description":"y","extra":"bad"}"#,
        )
        .unwrap();
        let r = check_plugin_json(root).unwrap();
        assert!(!r.passed);
        assert!(r.findings.iter().any(|f| f.check == "disallowed field"));
    }
}
