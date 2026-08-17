use anyhow::Result;
use godmode_core::session::Session;
use godmode_core::{detect, graph, integrations, model, templates};
use std::path::Path;

use crate::{TaskAction, exit_empty, filter_tasks};

pub fn run_task_action(root: &Path, json: bool, action: TaskAction) -> Result<()> {
    let mut session = Session::open(root)?;

    match action {
        TaskAction::List { priority, filter } => {
            let tasks_all = &session.graph().tasks;
            let tasks: Vec<&model::Task> =
                filter_tasks(tasks_all, priority.as_ref(), filter.as_deref());
            if tasks.is_empty() {
                if json {
                    println!("[]");
                } else {
                    println!("No tasks.");
                }
                return Ok(());
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&tasks)?);
            } else {
                for t in &tasks {
                    let crate_tag = t
                        .crate_name
                        .as_deref()
                        .map(|c| format!(" ({})", c))
                        .unwrap_or_default();
                    println!(
                        "[{}] {:8} {}{}",
                        t.id,
                        t.status.to_string(),
                        t.title,
                        crate_tag
                    );
                }
            }
        }
        TaskAction::Add {
            id,
            title,
            depends_on,
            crate_name,
        } => {
            let id = match id {
                Some(id) => id,
                None => graph::next_task_id(session.graph())?,
            };
            let mut task = model::Task::new(id.clone(), title);
            task.depends_on = depends_on;
            task.crate_name = crate_name;
            session.add_task(task)?;
            session.save()?;
            if json {
                println!("{}", serde_json::json!({"ok": true, "id": id}));
            } else {
                println!("Task '{}' added.", id);
            }
        }
        TaskAction::Start { id } => {
            session.start_task(&id)?;
            session.save()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "id": id, "status": "running"})
                );
            } else {
                println!("Task '{}' is now running.", id);
            }
        }
        TaskAction::Done { id, commit, notes } => {
            session.complete_task(&id, commit.as_deref(), notes.as_deref())?;
            session.save()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "id": id, "status": "done"})
                );
            } else {
                println!("Task '{}' marked done.", id);
            }
        }
        TaskAction::Block { id, reason } => {
            session.block_task(&id, &reason)?;
            session.save()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "id": id, "status": "blocked"})
                );
            } else {
                println!("Task '{}' blocked: {}", id, reason);
            }
        }
        TaskAction::Unblock { id } => {
            session.unblock_task(&id)?;
            session.save()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "id": id, "status": "pending"})
                );
            } else {
                println!("Task '{}' unblocked.", id);
            }
        }
        TaskAction::Remove { id } => {
            session.remove_task(&id)?;
            session.save()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "id": id, "removed": true})
                );
            } else {
                println!("Task '{}' removed.", id);
            }
        }
        TaskAction::Clear { done, all } => {
            if !done && !all {
                anyhow::bail!(
                    "specify --done to clear completed tasks or --all to clear everything"
                );
            }
            let count = session.clear_tasks(done);
            session.save()?;
            if json {
                println!("{}", serde_json::json!({"ok": true, "removed": count}));
            } else {
                println!("Removed {} task(s).", count);
            }
        }
        TaskAction::Next { priority } => {
            let runnable = graph::runnable(session.graph());
            let next: Vec<&model::Task> = match &priority {
                None => runnable,
                Some(p) => runnable.into_iter().filter(|t| &t.priority == p).collect(),
            };
            if next.is_empty() {
                exit_empty(json);
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&next)?);
            } else {
                for t in next {
                    let crate_tag = t
                        .crate_name
                        .as_deref()
                        .map(|c| format!(" ({})", c))
                        .unwrap_or_default();
                    println!("[{}] {}{}", t.id, t.title, crate_tag);
                }
            }
        }
        TaskAction::Run { id, auto_done } => {
            let run_cmd = session
                .graph()
                .tasks
                .iter()
                .find(|t| t.id == id)
                .ok_or_else(|| anyhow::anyhow!("task '{}' not found", id))?
                .run
                .clone()
                .ok_or_else(|| anyhow::anyhow!("task '{}' has no `run:` field", id))?;
            let exit = integrations::rx::run_cmd(&run_cmd)?;
            if exit.success() && auto_done {
                session.complete_task(&id, None, None)?;
                session.save()?;
                if !json {
                    println!("Task '{}' marked done.", id);
                }
            } else if !exit.success() {
                std::process::exit(exit.code().unwrap_or(2));
            }
        }
        TaskAction::Pull {
            project,
            github,
            repo,
            label,
        } => {
            let tasks = if github {
                integrations::gh::pull_issues(repo.as_deref(), label.as_deref())?
            } else {
                let project = match project {
                    Some(p) => p,
                    None => detect::package_name(root)?,
                };
                let todos = integrations::doob::todo_list(&project)?;
                integrations::doob::todos_to_tasks(&todos)
            };
            let mut imported = 0usize;
            for task in tasks {
                match session.add_task(task) {
                    Ok(()) => imported += 1,
                    Err(e) if e.to_string().contains("already exists") => {}
                    Err(e) => return Err(e),
                }
            }
            session.save()?;
            if json {
                println!("{}", serde_json::json!({"ok": true, "imported": imported}));
            } else if github {
                println!("Imported {} issue(s) from GitHub.", imported);
            } else {
                println!("Imported {} pending todos from doob.", imported);
            }
        }
        TaskAction::PushDone => {
            let mut pushed = 0usize;
            for task in session
                .graph()
                .tasks
                .iter()
                .filter(|t| t.status == model::Status::Done)
            {
                if let Some(uuid) = task.notes.strip_prefix("doob:") {
                    integrations::doob::todo_done(uuid.trim())?;
                    pushed += 1;
                }
            }
            if json {
                println!("{}", serde_json::json!({"ok": true, "pushed": pushed}));
            } else {
                println!("Pushed {} completed tasks to doob.", pushed);
            }
        }
        TaskAction::UnblockAll => {
            let count = session.unblock_all();
            session.save()?;
            if json {
                println!("{}", serde_json::json!({"ok": true, "unblocked": count}));
            } else if count == 0 {
                println!("No blocked tasks.");
            } else {
                println!("Unblocked {} task(s).", count);
            }
        }
        TaskAction::Apply { name, vars } => {
            let path = templates::find(root, &name)?;
            let tmpl = templates::load(&path, &vars)?;
            let tmpl_name = tmpl.meta.name.clone();
            let (applied, skipped) = session.apply_template(tmpl)?;
            session.save()?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "applied": applied, "skipped": skipped})
                );
            } else {
                println!(
                    "Applied {} task(s) from template '{}'. ({} skipped)",
                    applied, tmpl_name, skipped
                );
            }
        }
        TaskAction::ListTemplates => {
            let entries = templates::list(root)?;
            if entries.is_empty() {
                if json {
                    println!("[]");
                } else {
                    println!("No templates found.");
                }
                return Ok(());
            }
            if json {
                let arr: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "name": e.meta.name,
                            "description": e.meta.description,
                            "source": e.source.to_string(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr)?);
            } else {
                for e in &entries {
                    println!("[{}] {} — {}", e.source, e.meta.name, e.meta.description);
                }
            }
        }
    }

    Ok(())
}
