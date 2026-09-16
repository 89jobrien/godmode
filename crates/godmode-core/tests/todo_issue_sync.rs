use anyhow::{Result, bail};
use godmode_core::todo_issue_sync::{
    CreatedIssue, GitHubIssue, GitHubIssuePort, NewIssue, SyncStatus, sync,
};
use godmode_core::write_mode::WriteMode;
use std::cell::RefCell;
use tempfile::TempDir;

#[derive(Default)]
struct FakeGitHub {
    issues: RefCell<Vec<GitHubIssue>>,
    pages: RefCell<Vec<usize>>,
    creates: RefCell<Vec<NewIssue>>,
    fail_create: RefCell<Option<usize>>,
}

impl GitHubIssuePort for FakeGitHub {
    fn list_open_issues_page(&self, page: usize, per_page: usize) -> Result<Vec<GitHubIssue>> {
        self.pages.borrow_mut().push(page);
        let offset = (page - 1) * per_page;
        let issues = self.issues.borrow();
        Ok(issues[offset.min(issues.len())..issues.len().min(offset + per_page)].to_vec())
    }

    fn create_issue(&self, issue: &NewIssue) -> Result<CreatedIssue> {
        let attempt = self.creates.borrow().len() + 1;
        if self.fail_create.borrow().is_some_and(|n| n == attempt) {
            bail!("interrupted creation")
        }
        self.creates.borrow_mut().push(issue.clone());
        let number = 100 + attempt as u64;
        let created = CreatedIssue {
            number,
            url: format!("https://example.test/issues/{number}"),
        };
        self.issues.borrow_mut().push(GitHubIssue {
            number,
            title: issue.title.clone(),
            body: issue.body.clone(),
            url: created.url.clone(),
        });
        Ok(created)
    }
}

fn write_source(dir: &TempDir, body: &str) {
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), body).unwrap();
}

#[test]
fn fingerprint_is_stable_when_unrelated_lines_move() {
    let dir = TempDir::new().unwrap();
    write_source(
        &dir,
        "// TODO: add retries
",
    );
    let first = sync(dir.path(), &FakeGitHub::default(), WriteMode::Preview).unwrap();
    write_source(
        &dir,
        "// unrelated

// TODO: add retries
",
    );
    let second = sync(dir.path(), &FakeGitHub::default(), WriteMode::Preview).unwrap();
    assert_eq!(first.items[0].fingerprint, second.items[0].fingerprint);
}

#[test]
fn preview_paginates_and_uses_lowest_number_duplicate_without_writes() {
    let dir = TempDir::new().unwrap();
    write_source(
        &dir,
        "// TODO: add retries
",
    );
    let initial = sync(dir.path(), &FakeGitHub::default(), WriteMode::Preview).unwrap();
    let fingerprint = &initial.items[0].fingerprint;
    let marker = format!("<!-- godmode-todo:{fingerprint} -->");
    let github = FakeGitHub::default();
    for number in 1..=100 {
        github.issues.borrow_mut().push(GitHubIssue {
            number,
            title: format!("issue {number}"),
            body: String::new(),
            url: format!("u{number}"),
        });
    }
    github.issues.borrow_mut().push(GitHubIssue {
        number: 202,
        title: String::from("duplicate"),
        body: marker.clone(),
        url: String::from("u202"),
    });
    github.issues.borrow_mut().push(GitHubIssue {
        number: 201,
        title: String::from("duplicate"),
        body: marker,
        url: String::from("u201"),
    });
    let result = sync(dir.path(), &github, WriteMode::Preview).unwrap();
    assert_eq!(*github.pages.borrow(), vec![1, 2]);
    assert!(github.creates.borrow().is_empty());
    assert_eq!(
        result.items[0].status,
        SyncStatus::Covered {
            issue_number: 201,
            url: String::from("u201")
        }
    );
    assert!(
        !dir.path()
            .join(".ctx/godmode/todo-issue-sync.json")
            .exists()
    );
}

#[test]
fn apply_resumes_after_an_interrupted_creation_without_duplicates() {
    let dir = TempDir::new().unwrap();
    write_source(
        &dir,
        "// TODO: first task
// TODO: second task
",
    );
    let github = FakeGitHub::default();
    *github.fail_create.borrow_mut() = Some(2);
    assert!(sync(dir.path(), &github, WriteMode::Apply).is_err());
    assert_eq!(github.creates.borrow().len(), 1);
    assert!(
        dir.path()
            .join(".ctx/godmode/todo-issue-sync.json")
            .exists()
    );
    *github.fail_create.borrow_mut() = None;
    let resumed = sync(dir.path(), &github, WriteMode::Apply).unwrap();
    assert_eq!(github.creates.borrow().len(), 2);
    assert_eq!(resumed.created, 1);
    assert_eq!(resumed.covered, 1);
}

#[test]
fn inline_issue_reference_is_covered_without_creation() {
    let dir = TempDir::new().unwrap();
    write_source(&dir, "// TODO(#42): retain this marker\n");
    let github = FakeGitHub::default();
    let result = sync(dir.path(), &github, WriteMode::Apply).unwrap();
    assert_eq!(result.covered, 1);
    assert_eq!(result.created, 0);
    assert!(github.creates.borrow().is_empty());
}
