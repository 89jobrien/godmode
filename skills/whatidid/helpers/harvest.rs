#!/usr/bin/env rust-script
//! Harvest Claude Code session data for a given date.
//!
//! Usage: harvest.rs [YYYY-MM-DD]
//!
//! Scans ~/.claude/projects/*/*.jsonl for sessions whose first timestamp
//! matches the target date. Emits a JSON array of session records to stdout.
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! chrono = { version = "0.4", features = ["serde"] }
//! glob = "0.3"
//! anyhow = "1"
//! ```

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

// ── Output types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SessionRecord {
    session_id: String,
    project_slug: String,
    cwd: String,
    git_branch: String,
    slug: String,
    started_at: String,
    messages: Vec<Turn>,
    tool_calls: u32,
    read_calls: u32,
    edit_calls: u32,
    bash_calls: u32,
    model_metrics: Vec<ModelMetric>,
}

#[derive(Debug, Serialize)]
struct Turn {
    role: String, // "human" | "assistant"
    text: String,
    tool_requests: Vec<String>, // tool names used in this assistant turn
}

#[derive(Debug, Serialize)]
struct ModelMetric {
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
}

// ── JSONL event types we care about ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Event {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    timestamp: String,
    #[serde(rename = "sessionId", default)]
    session_id: String,
    #[serde(default)]
    cwd: String,
    #[serde(rename = "gitBranch", default)]
    git_branch: String,
    #[serde(default)]
    slug: String,
    // message content lives in `message` field for human/assistant turns
    message: Option<Value>,
}

fn main() -> Result<()> {
    let date_str = std::env::args()
        .nth(1)
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

    let _target = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .with_context(|| format!("invalid date: {date_str}"))?;

    let home = std::env::var("HOME").context("HOME not set")?;
    let projects_dir = PathBuf::from(&home).join(".claude/projects");

    let pattern = format!("{}/*/*.jsonl", projects_dir.display());
    let paths: Vec<PathBuf> = glob::glob(&pattern)
        .context("glob failed")?
        .filter_map(|r| r.ok())
        .collect();

    let mut sessions: HashMap<String, SessionRecord> = HashMap::new();

    for path in &paths {
        let file = fs::File::open(path)
            .with_context(|| format!("open {}", path.display()))?;
        let reader = BufReader::new(file);

        let project_slug = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        for line in reader.lines() {
            let line = match line {
                Ok(l) if l.trim().is_empty() => continue,
                Ok(l) => l,
                Err(_) => continue,
            };
            let event: Event = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Date filter — use first timestamp seen per session
            if event.timestamp.is_empty() {
                continue;
            }
            let event_date = event.timestamp.get(..10).unwrap_or("");
            if event_date != date_str {
                continue;
            }

            let record = sessions
                .entry(event.session_id.clone())
                .or_insert_with(|| SessionRecord {
                    session_id: event.session_id.clone(),
                    project_slug: project_slug.clone(),
                    cwd: event.cwd.clone(),
                    git_branch: event.git_branch.clone(),
                    slug: event.slug.clone(),
                    started_at: event.timestamp.clone(),
                    messages: Vec::new(),
                    tool_calls: 0,
                    read_calls: 0,
                    edit_calls: 0,
                    bash_calls: 0,
                    model_metrics: Vec::new(),
                });

            // Update context fields from richer events
            if !event.cwd.is_empty() {
                record.cwd = event.cwd.clone();
            }
            if !event.git_branch.is_empty() {
                record.git_branch = event.git_branch.clone();
            }
            if !event.slug.is_empty() {
                record.slug = event.slug.clone();
            }

            match event.event_type.as_str() {
                "human" => {
                    let text = extract_text(&event.message);
                    if should_include_human_turn(&text) {
                        record.messages.push(Turn {
                            role: "human".into(),
                            text,
                            tool_requests: Vec::new(),
                        });
                    }
                }
                "assistant" => {
                    let (text, tools) = extract_assistant(&event.message);
                    record.tool_calls += tools.len() as u32;
                    for t in &tools {
                        match t.as_str() {
                            "Read" => record.read_calls += 1,
                            "Edit" | "Write" => record.edit_calls += 1,
                            "Bash" => record.bash_calls += 1,
                            _ => {}
                        }
                    }
                    if !text.is_empty() || !tools.is_empty() {
                        record.messages.push(Turn {
                            role: "assistant".into(),
                            text,
                            tool_requests: tools,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    let mut result: Vec<SessionRecord> = sessions.into_values().collect();
    result.sort_by(|a, b| a.started_at.cmp(&b.started_at));

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// Extract plain text from a human message value.
fn extract_text(msg: &Option<Value>) -> String {
    let Some(msg) = msg else { return String::new() };
    // Human messages: { "role": "user", "content": [ {"type":"text","text":"..."}, ... ] }
    if let Some(content) = msg.get("content") {
        if let Some(arr) = content.as_array() {
            return arr
                .iter()
                .filter_map(|c| {
                    if c.get("type").and_then(|t| t.as_str()) == Some("text") {
                        c.get("text").and_then(|t| t.as_str()).map(str::to_string)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
        if let Some(s) = content.as_str() {
            return s.to_string();
        }
    }
    String::new()
}

/// Extract text + tool names from an assistant message.
fn extract_assistant(msg: &Option<Value>) -> (String, Vec<String>) {
    let Some(msg) = msg else {
        return (String::new(), Vec::new());
    };
    let mut text = String::new();
    let mut tools = Vec::new();

    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
        for block in content {
            match block.get("type").and_then(|t| t.as_str()) {
                Some("text") => {
                    if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                        text.push_str(t);
                    }
                }
                Some("tool_use") => {
                    if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                        tools.push(name.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    (text, tools)
}

/// Filter out noise from human turns.
fn should_include_human_turn(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return false;
    }
    // Single-word confirmations and injected context
    let noise = [
        "ok", "yes", "no", "go", "act", "done", "sure", "yep", "nope",
        "do it", "go ahead", "approved",
    ];
    if noise.iter().any(|n| t == *n) {
        return false;
    }
    // Injected system tags
    if t.starts_with("<current_datetime>") || t.starts_with("<system-reminder>") {
        return false;
    }
    true
}
