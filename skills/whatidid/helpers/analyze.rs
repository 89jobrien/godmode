#!/usr/bin/env rust-script
//! Analyze harvested Claude Code session data via the Anthropic API.
//!
//! Usage: analyze.rs <sessions.json> [YYYY-MM-DD]
//!
//! Reads JSON produced by harvest.rs, builds a transcript, and calls
//! claude-haiku-4-5 to produce a structured digest (goals, tasks, hours).
//! Caches the result to ~/.claude/skills/whatidid/cache/YYYY-MM-DD.json.
//! Emits the digest JSON to stdout.
//!
//! Requires: ANTHROPIC_API_KEY in environment (or via op run).
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! anyhow = "1"
//! chrono = "0.4"
//! ureq = { version = "2", features = ["json"] }
//! ```

use anyhow::{bail, Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

// ── Digest types (mirrors analysis.txt OUTPUT SCHEMA) ───────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct Digest {
    headline: String,
    primary_focus: String,
    day_narrative: String,
    goals: Vec<Goal>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Goal {
    title: String,
    label: String,
    summary: String,
    human_hours: f32,
    project: String,
    docs_referenced: Vec<String>,
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    title: String,
    what_got_done: String,
    domain_skills: Vec<String>,
    tech_skills: Vec<String>,
    task_type: String,
    professional_roles: Vec<String>,
    human_hours: f32,
}

fn main() -> Result<()> {
    let sessions_path = std::env::args()
        .nth(1)
        .context("usage: analyze.rs <sessions.json> [YYYY-MM-DD]")?;

    let date_str = std::env::args()
        .nth(2)
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

    // Cache check
    let cache_path = cache_path(&date_str)?;
    if cache_path.exists() {
        let cached = fs::read_to_string(&cache_path)
            .with_context(|| format!("read cache {}", cache_path.display()))?;
        print!("{cached}");
        return Ok(());
    }

    let sessions_raw = fs::read_to_string(&sessions_path)
        .with_context(|| format!("read {sessions_path}"))?;
    let sessions: Vec<Value> = serde_json::from_str(&sessions_raw)
        .context("parse sessions JSON")?;

    if sessions.is_empty() {
        bail!("no sessions found for {date_str}");
    }

    let transcript = build_transcript(&sessions);
    let analysis_prompt = load_analysis_prompt()?;
    let prompt = analysis_prompt.replace("{transcript}", &transcript);

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY not set — run via: op run -- analyze.rs ...")?;

    let response = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .context("Anthropic API call failed")?;

    let body: Value = response.into_json().context("parse API response")?;
    let content_raw = body["content"][0]["text"]
        .as_str()
        .context("missing content[0].text in response")?;

    // Strip markdown fences if present (gpt-4o-mini wraps despite prompt instruction)
    let content = content_raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Validate it's parseable JSON before caching
    let digest: Digest = serde_json::from_str(content)
        .with_context(|| format!("model returned invalid JSON:\n{content}"))?;

    let out = serde_json::to_string_pretty(&digest)?;
    fs::create_dir_all(cache_path.parent().unwrap())?;
    fs::write(&cache_path, &out)
        .with_context(|| format!("write cache {}", cache_path.display()))?;

    println!("{out}");
    Ok(())
}

fn build_transcript(sessions: &[Value]) -> String {
    let mut parts = Vec::new();

    for (i, session) in sessions.iter().enumerate() {
        let cwd = session["cwd"].as_str().unwrap_or("unknown");
        let branch = session["git_branch"].as_str().unwrap_or("");
        let project = session["project_slug"].as_str().unwrap_or("unknown");
        let started = session["started_at"].as_str().unwrap_or("");
        let tools = session["tool_calls"].as_u64().unwrap_or(0);
        let reads = session["read_calls"].as_u64().unwrap_or(0);
        let edits = session["edit_calls"].as_u64().unwrap_or(0);

        parts.push(format!(
            "SESSION {} — project={project}  cwd={cwd}  branch={branch}  started={started}\n\
             SIGNALS: tool_calls={tools}  read_calls={reads}  edit_calls={edits}",
            i + 1
        ));

        if let Some(messages) = session["messages"].as_array() {
            for msg in messages {
                let role = msg["role"].as_str().unwrap_or("?");
                let text = msg["text"].as_str().unwrap_or("").trim();
                if !text.is_empty() {
                    parts.push(format!("[{role}] {text}"));
                }
                if let Some(tools) = msg["tool_requests"].as_array() {
                    let names: Vec<&str> = tools
                        .iter()
                        .filter_map(|t| t.as_str())
                        .collect();
                    if !names.is_empty() {
                        parts.push(format!("  tools: {}", names.join(", ")));
                    }
                }
            }
        }
        parts.push(String::new());
    }

    parts.join("\n")
}

fn load_analysis_prompt() -> Result<String> {
    // Try skill-local path first, then fallback to relative
    let skill_dir = skill_dir()?;
    let prompt_path = skill_dir.join("prompts/analysis.whatidid.txt");
    fs::read_to_string(&prompt_path)
        .with_context(|| format!("read prompt {}", prompt_path.display()))
}

fn cache_path(date: &str) -> Result<PathBuf> {
    let skill_dir = skill_dir()?;
    Ok(skill_dir.join(format!("cache/{date}.json")))
}

fn skill_dir() -> Result<PathBuf> {
    // Resolve from this script's location at runtime
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home)
        .join("dev/minibox/.claude/skills/whatidid"))
}
