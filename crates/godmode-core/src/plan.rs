//! Parse a plan markdown file and extract tasks for the task graph.
//!
//! Expected format (produced by `godmode:writing-plans`):
//!
//! ```markdown
//! ## Tasks
//! ### Task 1: Write failing test for FooAdapter
//! **Crate**: `foo-core`
//! ...
//! ### Task 2: Implement FooAdapter
//! **Crate**: `foo-core`
//! ...
//! ```
//!
//! Each `### Task N: <title>` line becomes one Task. An optional `**Crate**: \`name\``
//! line sets `crate_name`. Dependencies are inferred sequentially (task N depends on N-1)
//! unless the title contains "independent".

use anyhow::Result;

use crate::model::Task;

/// Parse tasks from a plan markdown string.
pub fn parse(markdown: &str) -> Result<Vec<Task>> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_crate: Option<String> = None;
    let mut current_run: Option<String> = None;
    let mut current_deps: Option<Vec<String>> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Detect task headings: `### Task N: Title` or `### N. Title`
        if let Some(rest) = trimmed.strip_prefix("### Task ") {
            // flush previous
            if let Some(title) = current_title.take() {
                push_task(
                    &mut tasks,
                    title,
                    current_crate.take(),
                    current_run.take(),
                    current_deps.take(),
                );
            }
            // strip leading "N: " or "N. "
            let title = rest
                .split_once(": ")
                .map(|x| x.1)
                .or_else(|| rest.split_once(". ").map(|x| x.1))
                .unwrap_or(rest)
                .trim()
                .to_string();
            current_title = Some(title);
            current_crate = None;
            current_deps = None;
            continue;
        }

        // Detect crate annotation: `**Crate**: `name``
        if trimmed.starts_with("**Crate**:") {
            let crate_name = trimmed
                .trim_start_matches("**Crate**:")
                .trim()
                .trim_matches('`')
                .to_string();
            current_crate = Some(crate_name);
        }

        // Detect run annotation: `**Run**: `command``
        if trimmed.starts_with("**Run**:") {
            let run_cmd = trimmed
                .trim_start_matches("**Run**:")
                .trim()
                .trim_matches('`')
                .to_string();
            current_run = Some(run_cmd);
        }

        // Detect depends-on annotation: `**Depends-on**: `t1,t2``
        if trimmed.starts_with("**Depends-on**:") {
            let raw = trimmed
                .trim_start_matches("**Depends-on**:")
                .trim()
                .trim_matches('`');
            let ids: Vec<String> = raw
                .split(',')
                .map(|s| s.trim().trim_matches('`').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            current_deps = Some(ids);
        }
    }

    // flush last
    if let Some(title) = current_title.take() {
        push_task(
            &mut tasks,
            title,
            current_crate.take(),
            current_run.take(),
            current_deps.take(),
        );
    }

    Ok(tasks)
}

fn push_task(
    tasks: &mut Vec<Task>,
    title: String,
    crate_name: Option<String>,
    run: Option<String>,
    deps: Option<Vec<String>>,
) {
    let idx = tasks.len() + 1;
    let id = format!("t{idx}");
    let mut task = Task::new(id, title);
    task.crate_name = crate_name;
    task.run = run;
    task.depends_on = if let Some(explicit) = deps {
        explicit
    } else if idx > 1 {
        // Sequential dependency: each task depends on the previous one.
        vec![format!("t{}", idx - 1)]
    } else {
        vec![]
    };
    tasks.push(task);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# Plan: Foo feature

## Tasks

### Task 1: Write failing test for FooAdapter
**Crate**: `foo-core`

Some description here.

### Task 2: Implement FooAdapter
**Crate**: `foo-core`

### Task 3: Wire into service layer
**Crate**: `foo-service`
"#;

    #[test]
    fn parses_task_count() {
        let tasks = parse(SAMPLE).unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn parses_titles() {
        let tasks = parse(SAMPLE).unwrap();
        assert_eq!(tasks[0].title, "Write failing test for FooAdapter");
        assert_eq!(tasks[1].title, "Implement FooAdapter");
        assert_eq!(tasks[2].title, "Wire into service layer");
    }

    #[test]
    fn parses_crate_names() {
        let tasks = parse(SAMPLE).unwrap();
        assert_eq!(tasks[0].crate_name.as_deref(), Some("foo-core"));
        assert_eq!(tasks[2].crate_name.as_deref(), Some("foo-service"));
    }

    #[test]
    fn sequential_deps() {
        let tasks = parse(SAMPLE).unwrap();
        assert!(tasks[0].depends_on.is_empty());
        assert_eq!(tasks[1].depends_on, vec!["t1"]);
        assert_eq!(tasks[2].depends_on, vec!["t2"]);
    }
}
