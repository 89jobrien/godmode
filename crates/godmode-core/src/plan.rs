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
//! line sets `crate_name`. Dependencies are inferred sequentially (task N depends on N-1) unless
//! an explicit `**Depends-on**:` annotation overrides them.

use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::graph;
use crate::model::{Task, TaskGraph};

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

/// Result of importing one parsed plan into a task graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngestReport {
    /// Number of tasks parsed from the plan.
    pub parsed: usize,
    /// Number of tasks added to the graph.
    pub added: usize,
    /// Number of tasks already present from the same plan.
    pub skipped: usize,
    /// Stable IDs assigned to every parsed task, in plan order.
    pub ids: Vec<String>,
}

/// Import parsed plan tasks into an existing graph.
pub fn ingest(graph: &mut TaskGraph, tasks: Vec<Task>, source: &str) -> Result<IngestReport> {
    if tasks.is_empty() {
        bail!("no tasks found in plan");
    }
    let tasks = prepare_for_ingest(tasks, graph, source)?;
    let parsed = tasks.len();
    let ids: Vec<String> = tasks.iter().map(|task| task.id.clone()).collect();
    let mut added = 0;
    let mut skipped = 0;
    let mut updated = graph.clone();

    for task in tasks {
        if graph_contains_same_task(&updated, &task) {
            skipped += 1;
        } else {
            graph::add_imported(&mut updated, task)?;
            added += 1;
        }
    }

    updated.record_plan_import(source.to_string(), ids.clone());
    *graph = updated;

    Ok(IngestReport {
        parsed,
        added,
        skipped,
        ids,
    })
}

/// Prepare parsed tasks for insertion into an existing graph.
///
/// Tasks keep short `tN` IDs when those IDs are unused. Colliding plans use a deterministic
/// namespace. Graph-level plan provenance makes repeated ingestion idempotent even when distinct
/// plans contain identical task content.
pub fn prepare_for_ingest(
    mut tasks: Vec<Task>,
    graph: &TaskGraph,
    source: &str,
) -> Result<Vec<Task>> {
    let namespace = source
        .rsplit(['/', '\\'])
        .next()
        .map(|name| name.strip_suffix(".md").unwrap_or(name))
        .map(normalize_namespace)
        .unwrap_or_default();
    if namespace.is_empty() {
        bail!("plan namespace must contain at least one letter or digit");
    }

    let existing_ids = graph.plan_task_ids(source);
    if existing_ids.is_some_and(|ids| tasks.len() < ids.len()) {
        bail!(
            "plan now contains fewer tasks than its existing import; remove or reconcile old tasks explicitly"
        );
    }
    let legacy_plan_match = existing_ids.is_none()
        && tasks.iter().all(|candidate| {
            graph
                .tasks
                .iter()
                .find(|task| task.id == candidate.id)
                .is_some_and(|task| {
                    !graph.is_imported_plan_task(&task.id) && same_task_content(task, candidate)
                })
        });
    let short_ids_available = existing_ids.is_none()
        && (legacy_plan_match
            || tasks.iter().all(|candidate| {
                graph.tasks.iter().all(|task| task.id != candidate.id)
                    && !graph.is_imported_plan_task(&candidate.id)
            }));
    let namespace_collides = existing_ids.is_none()
        && !short_ids_available
        && tasks.iter().any(|task| {
            let candidate = format!("{namespace}-{}", task.id);
            graph.tasks.iter().any(|existing| existing.id == candidate)
                || graph.is_imported_plan_task(&candidate)
        });
    let namespace = if namespace_collides {
        format!("{namespace}-{:016x}", stable_source_hash(source))
    } else {
        namespace
    };

    let id_map: HashMap<String, String> = tasks
        .iter()
        .enumerate()
        .map(|(index, task)| {
            let id = existing_ids
                .and_then(|ids| ids.get(index))
                .cloned()
                .unwrap_or_else(|| {
                    if short_ids_available {
                        task.id.clone()
                    } else {
                        format!("{namespace}-{}", task.id)
                    }
                });
            (task.id.clone(), id)
        })
        .collect();

    for task in &mut tasks {
        task.id = id_map[&task.id].clone();
        for dependency in &mut task.depends_on {
            if let Some(external) = dependency.strip_prefix("graph:") {
                if external.is_empty() || graph.tasks.iter().all(|task| task.id != external) {
                    bail!("external dependency '{dependency}' does not exist in the graph");
                }
                *dependency = external.to_string();
            } else if let Some(rebased) = id_map.get(dependency) {
                *dependency = rebased.clone();
            }
        }
    }

    for task in &tasks {
        if existing_ids.is_none() && graph.is_imported_plan_task(&task.id) {
            bail!(
                "task '{}' is reserved by another imported plan; rename the plan file",
                task.id
            );
        }
        if let Some(existing) = graph.tasks.iter().find(|existing| existing.id == task.id)
            && !same_task_identity(existing, task)
            && !(legacy_plan_match && same_task_content(existing, task))
        {
            bail!(
                "task '{}' already exists with different plan content; rename the plan file",
                task.id
            );
        }
    }

    Ok(tasks)
}

fn graph_contains_same_task(graph: &TaskGraph, candidate: &Task) -> bool {
    graph
        .tasks
        .iter()
        .find(|task| task.id == candidate.id)
        .is_some_and(|task| same_task_identity(task, candidate))
}

fn same_task_identity(left: &Task, right: &Task) -> bool {
    same_task_content(left, right)
}

fn same_task_content(left: &Task, right: &Task) -> bool {
    left.title == right.title
        && left.crate_name == right.crate_name
        && left.run == right.run
        && left.depends_on == right.depends_on
}

fn stable_source_hash(source: &str) -> u64 {
    source
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
}

fn normalize_namespace(namespace: &str) -> String {
    let mut normalized = String::new();
    let mut previous_dash = false;
    for character in namespace.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            normalized.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !normalized.is_empty() {
            normalized.push('-');
            previous_dash = true;
        }
    }
    normalized.trim_end_matches('-').to_string()
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

    #[test]
    fn prepare_for_ingest_namespaces_colliding_plan() {
        let first = parse(SAMPLE).unwrap();
        let mut graph = TaskGraph::default();
        graph.tasks = first;
        let second =
            parse("### Task 1: Other alpha\n### Task 2: Other beta\n**Crate**: `other-core`")
                .unwrap();

        let prepared = prepare_for_ingest(second, &graph, "Second Plan").unwrap();

        assert_eq!(prepared[0].id, "second-plan-t1");
        assert_eq!(prepared[1].id, "second-plan-t2");
        assert_eq!(prepared[1].depends_on, vec!["second-plan-t1"]);
    }

    #[test]
    fn prepare_for_ingest_keeps_identical_plan_idempotent() {
        let tasks = parse(SAMPLE).unwrap();
        let mut graph = TaskGraph::default();
        ingest(&mut graph, tasks.clone(), "same-plan").unwrap();

        let prepared = prepare_for_ingest(tasks, &graph, "same-plan").unwrap();

        assert_eq!(prepared[0].id, "t1");
        assert_eq!(prepared[1].depends_on, vec!["t1"]);
    }

    #[test]
    fn identical_content_from_distinct_plans_gets_distinct_ids() {
        let tasks = parse(SAMPLE).unwrap();
        let mut graph = TaskGraph::default();
        ingest(&mut graph, tasks.clone(), "first-plan").unwrap();

        let prepared = prepare_for_ingest(tasks, &graph, "second-plan").unwrap();

        assert_eq!(prepared[0].id, "second-plan-t1");
        assert_eq!(prepared[1].depends_on, vec!["second-plan-t1"]);
    }

    #[test]
    fn explicit_graph_dependency_is_not_rebased() {
        let mut graph = TaskGraph::default();
        graph.tasks.push(Task::new("t1", "Existing task"));
        let tasks = parse("### Task 1: New task\n**Depends-on**: `graph:t1`").unwrap();

        let prepared = prepare_for_ingest(tasks, &graph, "dependent-plan").unwrap();

        assert_eq!(prepared[0].id, "dependent-plan-t1");
        assert_eq!(prepared[0].depends_on, vec!["t1"]);
    }

    #[test]
    fn namespaced_plan_remains_idempotent_after_short_ids_are_removed() {
        let mut graph = TaskGraph::default();
        ingest(&mut graph, parse(SAMPLE).unwrap(), "first-plan").unwrap();
        let second = parse("### Task 1: Second alpha\n### Task 2: Second beta").unwrap();
        ingest(&mut graph, second.clone(), "second-plan").unwrap();
        graph
            .tasks
            .retain(|task| task.id.starts_with("second-plan-"));

        let report = ingest(&mut graph, second, "second-plan").unwrap();

        assert_eq!(report.added, 0);
        assert_eq!(report.skipped, 2);
        assert_eq!(report.ids, ["second-plan-t1", "second-plan-t2"]);
    }

    #[test]
    fn legacy_matching_tasks_are_adopted_without_duplication() {
        let tasks = parse(SAMPLE).unwrap();
        let mut graph = TaskGraph::default();
        graph.tasks = tasks.clone();

        let report = ingest(&mut graph, tasks, "/repo/plans/legacy.md").unwrap();

        assert_eq!(report.added, 0);
        assert_eq!(report.skipped, 3);
        assert_eq!(graph.tasks.len(), 3);
        assert_eq!(
            graph.plan_task_ids("/repo/plans/legacy.md").unwrap(),
            ["t1", "t2", "t3"]
        );
    }

    #[test]
    fn failed_ingest_leaves_graph_unchanged() {
        let mut graph = TaskGraph::default();
        ingest(&mut graph, parse(SAMPLE).unwrap(), "/repo/plans/same.md").unwrap();
        let before = serde_yaml::to_string(&graph).unwrap();
        let changed = parse(
            "### Task 1: Changed task\n### Task 2: Implement FooAdapter\n### Task 3: Wire into service layer",
        )
        .unwrap();

        let error = ingest(&mut graph, changed, "/repo/plans/same.md").unwrap_err();

        assert!(error.to_string().contains("different plan content"));
        assert_eq!(serde_yaml::to_string(&graph).unwrap(), before);
    }

    #[test]
    fn missing_external_graph_dependency_is_rejected() {
        let graph = TaskGraph::default();
        let tasks = parse("### Task 1: New task\n**Depends-on**: `graph:missing`").unwrap();

        let error = prepare_for_ingest(tasks, &graph, "/repo/plan.md").unwrap_err();

        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn removed_imported_ids_remain_reserved_for_their_source() {
        let mut graph = TaskGraph::default();
        ingest(&mut graph, parse(SAMPLE).unwrap(), "/repo/first.md").unwrap();
        graph.tasks.clear();
        let other = parse("### Task 1: Other task").unwrap();

        let prepared = prepare_for_ingest(other, &graph, "/repo/other.md").unwrap();

        assert_eq!(prepared[0].id, "other-t1");
        let restored = ingest(&mut graph, parse(SAMPLE).unwrap(), "/repo/first.md").unwrap();
        assert_eq!(restored.ids, ["t1", "t2", "t3"]);
    }

    #[test]
    fn shortening_an_imported_plan_is_rejected_atomically() {
        let mut graph = TaskGraph::default();
        ingest(&mut graph, parse(SAMPLE).unwrap(), "/repo/plan.md").unwrap();
        let before = serde_yaml::to_string(&graph).unwrap();
        let shortened = parse("### Task 1: Write failing test for FooAdapter").unwrap();

        let error = ingest(&mut graph, shortened, "/repo/plan.md").unwrap_err();

        assert!(error.to_string().contains("fewer tasks"));
        assert_eq!(serde_yaml::to_string(&graph).unwrap(), before);
    }

    #[test]
    fn empty_plan_is_rejected_without_recording_an_import() {
        let mut graph = TaskGraph::default();

        let error = ingest(&mut graph, Vec::new(), "/repo/empty.md").unwrap_err();

        assert!(error.to_string().contains("no tasks"));
        assert!(graph.plan_task_ids("/repo/empty.md").is_none());
    }
}
