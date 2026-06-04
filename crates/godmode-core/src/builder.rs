//! Interactive and file-driven task graph construction.
//!
//! `build_interactive` runs a three-phase loop on stdin/stdout — shape, wire, validate.
//! Each confirmed answer persists immediately to the graph.
//!
//! `build_from_file` ingests a template YAML (same format as `templates/`) then runs
//! the validate phase and returns a `BuildSummary`.

use std::io::{BufRead, Write};
use std::path::Path;

use anyhow::Result;

use crate::dispatch;
use crate::graph;
use crate::model::{Status, Task, TaskGraph};
use crate::templates;

// ── public types ───────────────────────────────────────────────────────────

/// Summary returned after a build session.
#[derive(Debug, serde::Serialize)]
pub struct BuildSummary {
    /// Tasks successfully added during this session.
    pub added: usize,
    /// Dependency links wired during Phase 2.
    pub wired: usize,
    /// Validation findings (informational strings).
    pub findings: Vec<String>,
    /// IDs of tasks that are now runnable.
    pub next: Vec<String>,
}

// ── public API ─────────────────────────────────────────────────────────────

/// Drive interactive graph construction on stdin/stdout.
///
/// Runs three phases: shape (add tasks), wire (set deps), validate (check graph health).
/// Each confirmed task is persisted immediately.
pub fn build_interactive(root: &Path) -> Result<BuildSummary> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    build_interactive_io(root, &mut stdin.lock(), &mut stdout.lock())
}

/// Drive non-interactive graph construction from a template YAML file.
///
/// Phases 1 and 2 are driven by the file. Phase 3 (validate) always runs.
/// Returns an error if any required template var is missing.
pub fn build_from_file(root: &Path, path: &Path, vars: &[String]) -> Result<BuildSummary> {
    let tmpl = templates::load(path, vars)?;
    let mut g = graph::load(root)?;
    let (added, _skipped) = templates::apply(&mut g, tmpl)?;
    graph::save(root, &g)?;

    let findings = validate(&g);
    let next = graph::runnable(&g).iter().map(|t| t.id.clone()).collect();

    Ok(BuildSummary {
        added,
        wired: 0, // deps come from the template file, not prompted
        findings,
        next,
    })
}

/// Inspect a graph and return human-readable finding strings.
///
/// Findings are informational — callers decide whether to treat any as blocking.
pub fn validate(graph: &TaskGraph) -> Vec<String> {
    let mut findings = Vec::new();

    let pending: Vec<&Task> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Pending)
        .collect();

    let runnable = graph::runnable(graph);

    // No runnable tasks but pending tasks exist → blocked or missing dep.
    if runnable.is_empty() && !pending.is_empty() {
        findings.push(format!(
            "No runnable tasks — {} pending task(s) have unmet dependencies. \
             Check for missing tasks or add `godmode task unblock-all` if blocked.",
            pending.len()
        ));
    }

    // Over-wide: more than 5 independent root tasks.
    let chains = dispatch::independent_chains(graph, usize::MAX);
    let root_count = chains
        .iter()
        .filter(|c| {
            // A chain is rooted if its first task has no pending deps.
            !c.tasks.is_empty()
        })
        .count();
    if root_count > 5 {
        findings.push(format!(
            "Graph is wide: {} independent chains. Consider grouping or sequencing \
             some tasks to reduce parallel load.",
            root_count
        ));
    }

    // Single critical path with no parallelism opportunity.
    let cp = dispatch::critical_path(graph);
    let active_count = graph
        .tasks
        .iter()
        .filter(|t| matches!(t.status, Status::Pending | Status::Running))
        .count();
    if chains.len() == 1 && active_count > 2 {
        findings.push(format!(
            "Everything is sequential: critical path is {} tasks deep with no \
             parallel opportunities. Consider splitting independent work into separate chains.",
            cp.len()
        ));
    }

    // Orphaned: pending tasks whose deps are all done but not showing as runnable.
    // This catches tasks blocked by status (Status::Blocked) but not in runnable.
    let done_ids: std::collections::HashSet<&str> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.as_str())
        .collect();
    let runnable_ids: std::collections::HashSet<&str> =
        runnable.iter().map(|t| t.id.as_str()).collect();

    let orphans: Vec<&str> = graph
        .tasks
        .iter()
        .filter(|t| {
            t.status == Status::Blocked
                && t.depends_on
                    .iter()
                    .all(|dep| done_ids.contains(dep.as_str()))
        })
        .map(|t| t.id.as_str())
        .filter(|id| !runnable_ids.contains(id))
        .collect();

    if !orphans.is_empty() {
        findings.push(format!(
            "Blocked task(s) with all deps done: {}. Run `godmode task unblock-all` \
             to reset them to pending.",
            orphans.join(", ")
        ));
    }

    findings
}

// ── interactive implementation ─────────────────────────────────────────────

pub(crate) fn build_interactive_io<R, W>(
    root: &Path,
    reader: &mut R,
    writer: &mut W,
) -> Result<BuildSummary>
where
    R: BufRead,
    W: Write,
{
    let mut g = graph::load(root)?;
    let mut added = 0usize;
    let mut wired = 0usize;
    let mut new_task_ids: Vec<String> = Vec::new();
    let mut id_counter = next_id_counter(&g);

    // ── Phase 1: Shape ─────────────────────────────────────────────────────
    writeln!(
        writer,
        "\n=== Phase 1: Shape — what tasks need to happen? ==="
    )?;
    writeln!(
        writer,
        "(Press Enter with no input to finish this phase.)\n"
    )?;

    loop {
        write!(writer, "What's the next thing that needs to happen? ")?;
        writer.flush()?;
        let title = read_line(reader)?;
        if title.is_empty() {
            break;
        }

        write!(writer, "Which crate? (blank to skip) ")?;
        writer.flush()?;
        let crate_input = read_line(reader)?;
        let crate_name = if crate_input.is_empty() {
            None
        } else {
            Some(crate_input)
        };

        let default_id = format!("t{}", id_counter);
        write!(writer, "Task ID [{}]: ", default_id)?;
        writer.flush()?;
        let id_input = read_line(reader)?;
        let id = if id_input.is_empty() {
            default_id
        } else {
            id_input
        };

        let mut task = Task::new(id.clone(), title.clone());
        task.crate_name = crate_name.clone();

        match graph::add(&mut g, task) {
            Ok(()) => {
                graph::save(root, &g)?;
                added += 1;
                id_counter += 1;
                new_task_ids.push(id.clone());
                let crate_tag = crate_name
                    .as_deref()
                    .map(|c| format!(" ({})", c))
                    .unwrap_or_default();
                writeln!(writer, "  Added [{}] {}{}\n", id, title, crate_tag)?;
            }
            Err(e) if e.to_string().contains("already exists") => {
                writeln!(writer, "  Skipped — task '{}' already exists.\n", id)?;
            }
            Err(e) => return Err(e),
        }
    }

    if new_task_ids.is_empty() {
        writeln!(writer, "\nNo tasks added.")?;
        let findings = validate(&g);
        let next = graph::runnable(&g).iter().map(|t| t.id.clone()).collect();
        return Ok(BuildSummary {
            added: 0,
            wired: 0,
            findings,
            next,
        });
    }

    // ── Phase 2: Wire ──────────────────────────────────────────────────────
    writeln!(writer, "\n=== Phase 2: Wire — set dependencies ===")?;
    writeln!(
        writer,
        "(Enter comma-separated task IDs that must complete first, or blank to skip.)\n"
    )?;

    for id in &new_task_ids {
        let task_title = g
            .tasks
            .iter()
            .find(|t| t.id == *id)
            .map(|t| t.title.as_str())
            .unwrap_or("");

        write!(
            writer,
            "What must be done before [{}] \"{}\"? ",
            id, task_title
        )?;
        writer.flush()?;
        let deps_input = read_line(reader)?;

        if deps_input.is_empty() {
            continue;
        }

        let deps: Vec<String> = deps_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if deps.is_empty() {
            continue;
        }

        // Update the task's depends_on in place.
        if let Some(task) = g.tasks.iter_mut().find(|t| t.id == *id) {
            for dep in &deps {
                if !task.depends_on.contains(dep) {
                    task.depends_on.push(dep.clone());
                    wired += 1;
                }
            }
        }
        graph::save(root, &g)?;
        writeln!(
            writer,
            "  Updated [{}] depends_on: [{}]\n",
            id,
            deps.join(", ")
        )?;
    }

    // Show critical path after wiring.
    let cp = dispatch::critical_path(&g);
    if !cp.is_empty() {
        writeln!(
            writer,
            "Critical path ({} tasks): {}",
            cp.len(),
            cp.iter()
                .map(|t| t.id.as_str())
                .collect::<Vec<_>>()
                .join(" → ")
        )?;
    }

    // ── Phase 3: Validate ──────────────────────────────────────────────────
    writeln!(writer, "\n=== Phase 3: Validate ===")?;

    let summary = g.summary();
    writeln!(
        writer,
        "{} done  {} running  {} pending  {} blocked",
        summary.done, summary.running, summary.pending, summary.blocked
    )?;

    let findings = validate(&g);
    if findings.is_empty() {
        writeln!(writer, "Graph looks healthy.")?;
    } else {
        for f in &findings {
            writeln!(writer, "  ! {}", f)?;
        }
    }

    let next: Vec<String> = graph::runnable(&g).iter().map(|t| t.id.clone()).collect();

    if !next.is_empty() {
        writeln!(writer, "\nGraph ready. Run: godmode task next")?;
    }

    Ok(BuildSummary {
        added,
        wired,
        findings,
        next,
    })
}

// ── helpers ────────────────────────────────────────────────────────────────

fn read_line<R: BufRead>(reader: &mut R) -> Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Compute the next auto-increment ID counter based on existing `tN` task IDs.
fn next_id_counter(g: &TaskGraph) -> usize {
    g.tasks
        .iter()
        .filter_map(|t| t.id.strip_prefix('t').and_then(|n| n.parse::<usize>().ok()))
        .max()
        .map(|n| n + 1)
        .unwrap_or(1)
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_graph(specs: &[(&str, Status, &[&str])]) -> TaskGraph {
        let mut g = TaskGraph::default();
        for (id, status, deps) in specs {
            let mut t = Task::new(*id, format!("Task {}", id));
            t.status = status.clone();
            t.depends_on = deps.iter().map(|s| s.to_string()).collect();
            g.tasks.push(t);
        }
        g
    }

    #[test]
    fn validate_empty_graph_no_findings() {
        let g = TaskGraph::default();
        assert!(validate(&g).is_empty());
    }

    #[test]
    fn validate_healthy_graph_no_findings() {
        let g = make_graph(&[("t1", Status::Pending, &[])]);
        assert!(validate(&g).is_empty());
    }

    #[test]
    fn validate_detects_no_runnable() {
        // t2 depends on t1 which is also pending — t2 is blocked by t1.
        // t1 is runnable so this should NOT fire. Use blocked status instead.
        let g = make_graph(&[
            ("t1", Status::Blocked, &[]),
            ("t2", Status::Pending, &["t1"]),
        ]);
        // t1 is blocked (not pending) so runnable won't return it;
        // t2 depends on t1 which is not done → no runnable tasks.
        let findings = validate(&g);
        assert!(findings.iter().any(|f| f.contains("No runnable tasks")));
    }

    #[test]
    fn validate_detects_orphaned_blocked_tasks() {
        let g = make_graph(&[("t1", Status::Done, &[]), ("t2", Status::Blocked, &["t1"])]);
        let findings = validate(&g);
        assert!(findings.iter().any(|f| f.contains("Blocked task(s)")));
    }

    #[test]
    fn validate_detects_wide_graph() {
        // 6 independent pending tasks → over-wide
        let specs: Vec<(&str, Status, &[&str])> = vec![
            ("t1", Status::Pending, &[]),
            ("t2", Status::Pending, &[]),
            ("t3", Status::Pending, &[]),
            ("t4", Status::Pending, &[]),
            ("t5", Status::Pending, &[]),
            ("t6", Status::Pending, &[]),
        ];
        let g = make_graph(&specs);
        let findings = validate(&g);
        assert!(findings.iter().any(|f| f.contains("wide")));
    }

    #[test]
    fn validate_detects_single_long_chain() {
        // 5 sequential tasks — single chain, no parallelism
        let g = make_graph(&[
            ("t1", Status::Pending, &[]),
            ("t2", Status::Pending, &["t1"]),
            ("t3", Status::Pending, &["t2"]),
            ("t4", Status::Pending, &["t3"]),
            ("t5", Status::Pending, &["t4"]),
        ]);
        let findings = validate(&g);
        assert!(findings.iter().any(|f| f.contains("sequential")));
    }

    #[test]
    fn next_id_counter_starts_at_1_on_empty() {
        let g = TaskGraph::default();
        assert_eq!(next_id_counter(&g), 1);
    }

    #[test]
    fn next_id_counter_increments_past_existing() {
        let g = make_graph(&[("t3", Status::Pending, &[])]);
        assert_eq!(next_id_counter(&g), 4);
    }

    #[test]
    fn build_from_file_applies_template() {
        let root = TempDir::new().unwrap();
        std::fs::create_dir_all(root.path().join(".ctx").join("godmode")).unwrap();
        std::fs::create_dir_all(root.path().join("templates")).unwrap();

        let tmpl = r#"
meta:
  name: simple
  description: "Simple template"
  vars:
    - name: crate
      required: true

tasks:
  - id: "s1"
    title: "Task for {{crate}}"
    status: pending
    depends_on: []
    notes: ""
    crate_name: "{{crate}}"
"#;
        std::fs::write(root.path().join("templates/simple.yaml"), tmpl).unwrap();

        let path = root.path().join("templates/simple.yaml");
        let summary = build_from_file(root.path(), &path, &["crate=mylib".to_string()]).unwrap();

        assert_eq!(summary.added, 1);
        assert!(summary.findings.is_empty());
        assert_eq!(summary.next, vec!["s1"]);
    }

    #[test]
    fn build_interactive_adds_tasks() {
        let root = TempDir::new().unwrap();
        std::fs::create_dir_all(root.path().join(".ctx").join("godmode")).unwrap();

        // Simulate: title="do the thing", crate="", id="" (default t1), then blank to finish,
        // then blank for deps, then done.
        let input = "do the thing\n\n\n\n\n";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let mut output = Vec::new();

        let summary = build_interactive_io(root.path(), &mut reader, &mut output).unwrap();

        assert_eq!(summary.added, 1);
        assert_eq!(summary.next, vec!["t1"]);
    }
}
