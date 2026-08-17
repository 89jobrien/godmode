use anyhow::Result;
use godmode_core::hooks;
use godmode_core::integrations::hook_runner;
use serde_json::Value;
use std::path::Path;

use crate::HookAction;

pub fn run_hook_action(root: &Path, json: bool, action: HookAction) -> Result<()> {
    match action {
        HookAction::List => list_hooks(root, json),
        HookAction::Log { tail } => log_hooks(root, json, tail),
        HookAction::Test { script } => test_hook(root, json, &script),
        HookAction::Migrate => migrate_hooks(root, json),
        HookAction::Run { name } => run_builtin_hook(root, json, &name),
    }
}

fn list_hooks(root: &Path, json: bool) -> Result<()> {
    let hooks_path = root.join("hooks").join("hooks.json");
    if !hooks_path.exists() {
        anyhow::bail!("hooks/hooks.json not found at {}", hooks_path.display());
    }
    let raw = std::fs::read_to_string(&hooks_path)?;
    let val: Value = serde_json::from_str(&raw)?;
    let entries = hook_runner::list_hooks_from_json(&val);
    if json {
        let arr: Vec<serde_json::Value> = entries
            .iter()
            .map(|(ev, mat, scr)| {
                serde_json::json!({
                    "event": ev,
                    "matcher": mat,
                    "script": scr,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
    } else {
        println!("{:<20} {:<10} SCRIPT", "EVENT", "MATCHER");
        println!("{}", "-".repeat(80));
        for (ev, mat, scr) in &entries {
            println!("{:<20} {:<10} {}", ev, mat, scr);
        }
    }
    Ok(())
}

fn log_hooks(root: &Path, json: bool, tail: usize) -> Result<()> {
    let lines = hook_runner::read_hook_log(root, tail).unwrap_or_default();
    if lines.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("No hook log entries.");
        }
        return Ok(());
    }
    if json {
        let vals: Vec<serde_json::Value> = lines
            .iter()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        println!("{}", serde_json::to_string_pretty(&vals)?);
    } else {
        for line in &lines {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                let hook = v["hook"].as_str().unwrap_or("?");
                let event = v["event"].as_str().unwrap_or("?");
                let code = v["exit_code"].as_i64().unwrap_or(-1);
                let ts = v["ts"].as_str().unwrap_or("?");
                let stderr = v["stderr"].as_str().unwrap_or("");
                if stderr.is_empty() {
                    println!("[{}] {} ({}) exit={}", ts, hook, event, code);
                } else {
                    println!("[{}] {} ({}) exit={} | {}", ts, hook, event, code, stderr);
                }
            } else {
                println!("{}", line);
            }
        }
    }
    Ok(())
}

fn test_hook(root: &Path, json: bool, script: &str) -> Result<()> {
    let script_lower = script.to_lowercase();
    let synthetic_stdin = if script_lower.contains("stop") {
        r#"{"stop_hook_active":false,"transcript_turns":[]}"#
    } else if script_lower.contains("session") {
        r#"{"session_id":"test-session"}"#
    } else {
        r#"{"tool_input":{"command":"echo test"},"tool_response":{"exit_code":0}}"#
    };

    let tmp = std::env::temp_dir().join("godmode_hook_test_stdin.json");
    std::fs::write(&tmp, synthetic_stdin)?;

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{} < {}", script, tmp.display()))
        .output();

    match output {
        Ok(out) => {
            let exit_code = out.status.code().unwrap_or(-1);
            let stderr_text = String::from_utf8_lossy(&out.stderr).to_string();
            let stdout_text = String::from_utf8_lossy(&out.stdout).to_string();
            let _ = hook_runner::append_hook_event(root, script, "test", exit_code, &stderr_text);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "script": script,
                        "exit_code": exit_code,
                        "stdout": stdout_text,
                        "stderr": stderr_text,
                    }))?
                );
            } else {
                println!("exit: {}", exit_code);
                if !stdout_text.is_empty() {
                    println!("stdout:\n{}", stdout_text);
                }
                if !stderr_text.is_empty() {
                    println!("stderr:\n{}", stderr_text);
                }
            }
        }
        Err(e) => {
            anyhow::bail!("failed to run script '{}': {}", script, e);
        }
    }
    Ok(())
}

fn migrate_hooks(root: &Path, json: bool) -> Result<()> {
    let hooks_dir = root.join("hooks");
    let migrations_dir = hooks_dir.join("migrations");
    if !migrations_dir.exists() {
        anyhow::bail!(
            "hooks/migrations/ not found at {}",
            migrations_dir.display()
        );
    }
    let mut migrated = 0usize;
    for entry in std::fs::read_dir(&migrations_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("sh") {
            continue;
        }
        let output = std::process::Command::new("sh").arg(&path).output()?;
        if output.status.success() {
            migrated += 1;
        }
    }
    if json {
        println!("{}", serde_json::json!({"ok": true, "migrated": migrated}));
    } else {
        println!("Migrated {} hook script(s).", migrated);
    }
    Ok(())
}

fn run_builtin_hook(root: &Path, json: bool, name: &str) -> Result<()> {
    let stdin = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
    let input: Value = serde_json::from_str(&stdin).unwrap_or_default();
    let tool_input = input
        .get("tool_input")
        .cloned()
        .unwrap_or_else(|| input.clone());
    let command = tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let file_path = tool_input
        .get("file_path")
        .and_then(|v| v.as_str())
        .or_else(|| tool_input.get("path").and_then(|v| v.as_str()))
        .unwrap_or("");
    let prompt = tool_input
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let exit_code = input
        .get("tool_response")
        .and_then(|v| v.get("exit_code"))
        .and_then(|v| v.as_i64())
        .or_else(|| input.get("exit_code").and_then(|v| v.as_i64()))
        .unwrap_or(0);
    let tool_response = input
        .get("tool_response")
        .cloned()
        .unwrap_or_else(|| input.clone());

    match name {
        "stop-guard" => {
            let decision = hooks::stop_guard::check(root);
            let (msg, code) = hooks::stop_guard::format_decision(&decision);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
            std::process::exit(code);
        }
        "auto-block" => {
            if let Some(ctx) = hooks::hook_context::HookContext::load_with_root(&stdin, root) {
                let result = hooks::auto_block::check(&ctx);
                if let Some(msg) = hooks::auto_block::format_result(&result) {
                    eprintln!("{msg}");
                }
                if let hooks::auto_block::AutoBlockResult::Blocked {
                    ref task_id,
                    ref reason,
                } = result
                    && let Ok(mut g) = godmode_core::graph::load(root)
                    && godmode_core::graph::block(&mut g, task_id, reason).is_ok()
                {
                    let _ = godmode_core::graph::save(root, &g);
                }
            }
        }
        "pre-commit" => {
            let result = hooks::pre_commit::run(root);
            let (msg, code) = hooks::pre_commit::format_result(&result);
            eprintln!("{msg}");
            std::process::exit(code);
        }
        "pre-commit-gate" => {
            let result = hooks::pre_commit::run_lint_gate(root);
            let (msg, code) = hooks::pre_commit::format_result(&result);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
            if json {
                let decision = if code == 0 { "approve" } else { "block" };
                println!(
                    "{}",
                    serde_json::json!({"decision": decision, "reason": msg})
                );
            }
            std::process::exit(code);
        }
        "quality-gate" => {
            if let Err(e) = hooks::quality_gate::run(root, None) {
                eprintln!("[godmode:quality-gate] {e}");
                std::process::exit(1);
            }
            eprintln!("[godmode:quality-gate] all gates passed.");
        }
        "task-management" => {
            let msg = hooks::task_management::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "parallel-agents" => {
            let msg = hooks::parallel_agents::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "moa" => {
            let msg = hooks::moa::run(root, command);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "wave-integration" => {
            let msg = hooks::wave_integration::run(root, command);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "introspection" => {
            let msg = hooks::introspection::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "agent-governance" => {
            let decision = hooks::agent_governance::check(root, &input);
            let msg = hooks::agent_governance::format_reminders(&decision, "Agent");
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
            if json {
                println!("{}", hooks::agent_governance::format_json(&decision));
            } else if decision.approved {
                println!("{}", decision.reason);
            } else {
                eprintln!("{}", decision.reason);
                std::process::exit(1);
            }
        }
        "brainstorm" => {
            let msg = hooks::brainstorm::run(root, file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "cap" => {
            let msg = hooks::cap::run(root, command);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "ci-fix" => {
            let msg = hooks::ci_fix::run(command, exit_code);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "code-review" => {
            let msg = hooks::code_review::run(command);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "context-map" => {
            let msg = hooks::context_map::run(root, file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "design" => {
            let msg = hooks::design::run(root, file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "doublecheck" => {
            let msg = hooks::doublecheck::run(command, exit_code);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "memory-banking" => {
            let msg = hooks::memory_banking::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "merge" => {
            let msg = hooks::merge::run(root, command, exit_code);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "mini-context-graph" => {
            let msg = hooks::mini_context_graph::run(root, file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "observability" => {
            let msg = hooks::observability::run(root, command, exit_code);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "receiving-review" => {
            let msg = hooks::receiving_review::run(file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "refactoring" => {
            let msg = hooks::refactoring::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "rust-conventions" => {
            let msg = hooks::rust_conventions::run(file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "self-reflect" => {
            let msg = hooks::self_reflect::run(prompt);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "systematic-debugging" => {
            let msg = hooks::systematic_debugging::run(command, exit_code);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "tackle-issues" => {
            let msg = hooks::tackle_issues::run(command);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "task-driven-dev" => {
            let msg = hooks::task_driven_dev::run(root, command);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "testing-philosophy" => {
            let msg = hooks::testing_philosophy::run(root, file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "todo-issue-sync" => {
            let msg = hooks::todo_issue_sync::run(root, command, &tool_response);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "using-godmode" => {
            let msg = hooks::using_godmode::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "verification" => {
            let msg = hooks::verification::run(root);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        "writing-plans" => {
            let msg = hooks::writing_plans::run(root, file_path);
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
        }
        other => anyhow::bail!("unknown built-in hook '{other}'"),
    }

    Ok(())
}
