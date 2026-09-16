//! Native synchronization of source TODO markers to GitHub issues.

use crate::write_mode::WriteMode;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PER_PAGE: usize = 100;
const PREFIX: &str = "<!-- godmode-todo:";
const EXCLUDED: &[&str] = &[
    ".git",
    ".ctx",
    ".worktrees",
    "target",
    "node_modules",
    "vendor",
];
const EXTENSIONS: &[&str] = &["rs", "go", "nu", "ts", "tsx", "js", "jsx", "py"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default, alias = "html_url")]
    pub url: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewIssue {
    pub title: String,
    pub body: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedIssue {
    pub number: u64,
    pub url: String,
}

pub trait GitHubIssuePort {
    fn list_open_issues_page(&self, page: usize, per_page: usize) -> Result<Vec<GitHubIssue>>;
    fn create_issue(&self, issue: &NewIssue) -> Result<CreatedIssue>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncStatus {
    Missing,
    Covered { issue_number: u64, url: String },
    Created { issue_number: u64, url: String },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SyncItem {
    pub path: String,
    pub line: usize,
    pub text: String,
    pub fingerprint: String,
    pub status: SyncStatus,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SyncResult {
    pub mode: &'static str,
    pub total: usize,
    pub covered: usize,
    pub missing: usize,
    pub created: usize,
    pub items: Vec<SyncItem>,
}
#[derive(Clone, Debug)]
struct TodoMarker {
    path: String,
    line: usize,
    text: String,
    fingerprint: String,
    issue_ref: Option<u64>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SyncState {
    #[serde(default)]
    created: BTreeMap<String, CreatedRecord>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreatedRecord {
    number: u64,
    url: String,
}

pub fn sync(root: &Path, github: &dyn GitHubIssuePort, mode: WriteMode) -> Result<SyncResult> {
    let todos = scan(root)?;
    let mut issues = fetch_all(github)?;
    issues.sort_by_key(|issue| issue.number);
    let mut state = load_state(root)?;
    let by_number: BTreeMap<_, _> = issues.iter().map(|issue| (issue.number, issue)).collect();
    let mut indexed = BTreeMap::new();
    for issue in &issues {
        for value in issue_fingerprints(&issue.body) {
            indexed.entry(value).or_insert(issue);
        }
    }
    let mut result = SyncResult {
        mode: if mode.is_apply() { "apply" } else { "preview" },
        total: todos.len(),
        covered: 0,
        missing: 0,
        created: 0,
        items: Vec::with_capacity(todos.len()),
    };
    for todo in todos {
        let covered = todo
            .issue_ref
            .map(|number| {
                let url = by_number
                    .get(&number)
                    .map_or_else(String::new, |issue| issue.url.clone());
                (number, url)
            })
            .or_else(|| {
                indexed
                    .get(&todo.fingerprint)
                    .map(|issue| (issue.number, issue.url.clone()))
            })
            .or_else(|| {
                state
                    .created
                    .get(&todo.fingerprint)
                    .map(|record| (record.number, record.url.clone()))
            });
        let status = if let Some((issue_number, url)) = covered {
            result.covered += 1;
            SyncStatus::Covered { issue_number, url }
        } else if mode.is_apply() {
            let created = github
                .create_issue(&issue_for(&todo))
                .with_context(|| format!("creating issue for {}:{}", todo.path, todo.line))?;
            state.created.insert(
                todo.fingerprint.clone(),
                CreatedRecord {
                    number: created.number,
                    url: created.url.clone(),
                },
            );
            save_state(root, &state)?;
            result.created += 1;
            SyncStatus::Created {
                issue_number: created.number,
                url: created.url,
            }
        } else {
            result.missing += 1;
            SyncStatus::Missing
        };
        result.items.push(SyncItem {
            path: todo.path,
            line: todo.line,
            text: todo.text,
            fingerprint: todo.fingerprint,
            status,
        });
    }
    Ok(result)
}

fn fetch_all(github: &dyn GitHubIssuePort) -> Result<Vec<GitHubIssue>> {
    let mut all = Vec::new();
    let mut page = 1;
    loop {
        let batch = github.list_open_issues_page(page, PER_PAGE)?;
        let done = batch.len() < PER_PAGE;
        all.extend(batch);
        if done {
            break;
        }
        page += 1;
    }
    Ok(all)
}

fn scan(root: &Path) -> Result<Vec<TodoMarker>> {
    let matcher =
        Regex::new(r"^(?://|#|/\*|\*)\s*(TODO|FIXME|HACK|XXX)(?:\(#(\d+)\))?\s*:?\s*(.*)")?;
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    let mut seen = BTreeMap::new();
    let mut todos = Vec::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let source = std::fs::read_to_string(&file)
            .with_context(|| format!("reading {}", file.display()))?;
        for (index, line) in source.lines().enumerate() {
            let Some(capture) = matcher.captures(line.trim_start()) else {
                continue;
            };
            let marker = capture.get(1).expect("marker").as_str().to_uppercase();
            let issue_ref = capture.get(2).and_then(|value| value.as_str().parse().ok());
            let detail = capture.get(3).map_or("", |value| value.as_str()).trim();
            let text = if detail.is_empty() {
                marker
            } else {
                format!("{marker}: {detail}")
            };
            let ordinal = seen
                .entry((relative.clone(), text.clone()))
                .or_insert(0_usize);
            let fingerprint = fingerprint(&relative, &text, *ordinal);
            *ordinal += 1;
            todos.push(TodoMarker {
                path: relative.clone(),
                line: index + 1,
                text,
                fingerprint,
                issue_ref,
            });
        }
    }
    todos.sort_by(|a, b| {
        a.fingerprint
            .cmp(&b.fingerprint)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.line.cmp(&b.line))
    });
    Ok(todos)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = std::fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !EXCLUDED.contains(&entry.file_name().to_string_lossy().as_ref()) {
                collect_files(&path, files)?;
            }
        } else if kind.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| EXTENSIONS.contains(&ext))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn fingerprint(path: &str, text: &str, ordinal: usize) -> String {
    let canonical = format!(
        "{path}\n{}\n{ordinal}",
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in canonical.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn issue_fingerprints(body: &str) -> impl Iterator<Item = String> + '_ {
    body.lines().filter_map(|line| {
        let value = line.trim().strip_prefix(PREFIX)?.strip_suffix(" -->")?;
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn issue_for(todo: &TodoMarker) -> NewIssue {
    let module = Path::new(&todo.path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("todo");
    let detail = todo
        .text
        .split_once(':')
        .map_or(todo.text.as_str(), |(_, value)| value.trim());
    NewIssue {
        title: format!("feat({module}): {detail}"),
        body: format!(
            "## Context\n\n`{}:{}` has an unresolved TODO.\n\n## TODO\n\n```\n{}\n```\n\n## Action\n\nResolve or implement the TODO at `{}:{}`.\n\n{PREFIX}{} -->\n",
            todo.path, todo.line, todo.text, todo.path, todo.line, todo.fingerprint
        ),
    }
}

fn state_path(root: &Path) -> PathBuf {
    root.join(".ctx/godmode/todo-issue-sync.json")
}
fn load_state(root: &Path) -> Result<SyncState> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(SyncState::default());
    }
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}
fn save_state(root: &Path, state: &SyncState) -> Result<()> {
    let path = state_path(root);
    std::fs::create_dir_all(path.parent().expect("state parent"))?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(
        &temporary,
        format!("{}\n", serde_json::to_string_pretty(state)?),
    )?;
    std::fs::rename(&temporary, &path)?;
    Ok(())
}
