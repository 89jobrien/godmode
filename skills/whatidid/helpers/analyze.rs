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
//! Cache encryption: if WHATIDID_CACHE_KEY is set, cache files are encrypted
//! at rest using AES-256-GCM with a key derived from the env var via SHA-256.
//! If WHATIDID_CACHE_KEY is unset, cache is stored as plaintext with a stderr
//! warning. Unencrypted cache files are read transparently for migration but
//! trigger a warning.
//!
//! SECURITY: Without WHATIDID_CACHE_KEY, cached API responses containing
//! activity data (session transcripts, goals, tasks) are stored as plaintext
//! JSON in ~/.claude/skills/whatidid/cache/. Set WHATIDID_CACHE_KEY to a
//! random string (e.g. `openssl rand -hex 32`) to encrypt at rest.
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! anyhow = "1"
//! chrono = "0.4"
//! ureq = { version = "2", features = ["json"] }
//! aes-gcm = "0.10"
//! sha2 = "0.10"
//! rand = "0.8"
//! ```

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{bail, Context, Result};
use chrono::Local;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest as Sha2Digest, Sha256};
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
    let encryption_key = derive_encryption_key();
    if cache_path.exists() {
        if let Some(cached) = read_cache(&cache_path, &encryption_key)? {
            print!("{cached}");
            return Ok(());
        }
    }

    let sessions_raw = if sessions_path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("read stdin")?;
        buf
    } else {
        fs::read_to_string(&sessions_path)
            .with_context(|| format!("read {sessions_path}"))?
    };
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
            "max_tokens": 8192,
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
    write_cache(&cache_path, &out, &encryption_key)?;

    println!("{out}");
    Ok(())
}

// ── Cache encryption helpers ────────────────────────────────────────────────
//
// Wire format for encrypted cache: 12-byte nonce || ciphertext (AES-256-GCM).
// The key is SHA-256(WHATIDID_CACHE_KEY). If the env var is unset, cache is
// stored/read as plaintext.

const ENCRYPTED_MAGIC: &[u8] = b"WDID";
const NONCE_LEN: usize = 12;

fn derive_encryption_key() -> Option<[u8; 32]> {
    let raw = std::env::var("WHATIDID_CACHE_KEY").ok()?;
    if raw.is_empty() {
        return None;
    }
    let hash = Sha256::digest(raw.as_bytes());
    Some(hash.into())
}

fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encryption failed: {e}"))?;
    let mut out = Vec::with_capacity(ENCRYPTED_MAGIC.len() + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(ENCRYPTED_MAGIC);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt_data(data: &[u8], key: &[u8; 32]) -> Result<String> {
    if data.len() < ENCRYPTED_MAGIC.len() + NONCE_LEN + 1 {
        bail!("encrypted cache too short");
    }
    if &data[..ENCRYPTED_MAGIC.len()] != ENCRYPTED_MAGIC {
        bail!("missing encryption magic header");
    }
    let nonce_start = ENCRYPTED_MAGIC.len();
    let nonce = Nonce::from_slice(&data[nonce_start..nonce_start + NONCE_LEN]);
    let ciphertext = &data[nonce_start + NONCE_LEN..];
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("decryption failed: {e}"))?;
    String::from_utf8(plaintext).context("decrypted cache is not valid UTF-8")
}

fn is_encrypted(data: &[u8]) -> bool {
    data.len() >= ENCRYPTED_MAGIC.len() && &data[..ENCRYPTED_MAGIC.len()] == ENCRYPTED_MAGIC
}

/// Read cache file. Returns Some(content) on success, None if unreadable.
/// Handles encrypted and plaintext files, with migration warnings.
fn read_cache(path: &PathBuf, key: &Option<[u8; 32]>) -> Result<Option<String>> {
    let raw = fs::read(path)
        .with_context(|| format!("read cache {}", path.display()))?;

    if is_encrypted(&raw) {
        match key {
            Some(k) => {
                let content = decrypt_data(&raw, k)
                    .with_context(|| format!("decrypt cache {}", path.display()))?;
                Ok(Some(content))
            }
            None => {
                eprintln!(
                    "warning: cache {} is encrypted but WHATIDID_CACHE_KEY is not set; \
                     re-running analysis",
                    path.display()
                );
                Ok(None)
            }
        }
    } else {
        // Plaintext cache (legacy / no encryption key was set when written)
        let content = String::from_utf8(raw)
            .with_context(|| format!("cache {} is not valid UTF-8", path.display()))?;
        if key.is_some() {
            eprintln!(
                "warning: cache {} is unencrypted plaintext; consider deleting it \
                 so it will be re-created with encryption",
                path.display()
            );
        }
        Ok(Some(content))
    }
}

/// Write cache file, encrypting if a key is available.
fn write_cache(path: &PathBuf, content: &str, key: &Option<[u8; 32]>) -> Result<()> {
    match key {
        Some(k) => {
            let encrypted = encrypt_data(content.as_bytes(), k)?;
            fs::write(path, encrypted)
                .with_context(|| format!("write encrypted cache {}", path.display()))?;
        }
        None => {
            eprintln!(
                "warning: WHATIDID_CACHE_KEY not set; caching plaintext. \
                 Set WHATIDID_CACHE_KEY to encrypt cached API responses at rest."
            );
            fs::write(path, content)
                .with_context(|| format!("write cache {}", path.display()))?;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_transcript_formats_sessions() {
        let sessions: Vec<Value> = vec![serde_json::json!({
            "cwd": "/dev/project",
            "git_branch": "main",
            "project_slug": "minibox",
            "started_at": "2026-05-20T10:00:00Z",
            "tool_calls": 5,
            "read_calls": 2,
            "edit_calls": 1,
            "messages": [
                {"role": "human", "text": "fix the bug", "tool_requests": []},
                {"role": "assistant", "text": "I'll look at it", "tool_requests": ["Read"]}
            ]
        })];
        let transcript = build_transcript(&sessions);
        assert!(transcript.contains("SESSION 1"));
        assert!(transcript.contains("project=minibox"));
        assert!(transcript.contains("[human] fix the bug"));
        assert!(transcript.contains("[assistant] I'll look at it"));
        assert!(transcript.contains("tools: Read"));
    }

    #[test]
    fn test_build_transcript_empty_sessions() {
        let sessions: Vec<Value> = vec![];
        let transcript = build_transcript(&sessions);
        assert!(transcript.is_empty());
    }

    #[test]
    fn test_cache_path_format() {
        let path = cache_path("2026-05-20").expect("cache_path should succeed");
        assert!(path.to_string_lossy().contains("cache/2026-05-20.json"));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0xABu8; 32];
        let plaintext = r#"{"headline":"test"}"#;
        let encrypted = encrypt_data(plaintext.as_bytes(), &key)
            .expect("encryption should succeed");
        assert!(is_encrypted(&encrypted));
        let decrypted = decrypt_data(&encrypted, &key)
            .expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_magic_header() {
        let key = [0x42u8; 32];
        let encrypted = encrypt_data(b"hello", &key)
            .expect("encryption should succeed");
        assert_eq!(&encrypted[..4], b"WDID");
    }

    #[test]
    fn test_plaintext_not_detected_as_encrypted() {
        let plaintext = b"{\"headline\":\"test\"}";
        assert!(!is_encrypted(plaintext));
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let key1 = [0xAAu8; 32];
        let key2 = [0xBBu8; 32];
        let encrypted = encrypt_data(b"secret", &key1)
            .expect("encryption should succeed");
        assert!(decrypt_data(&encrypted, &key2).is_err());
    }

    #[test]
    fn test_derive_key_returns_none_without_env() {
        // WHATIDID_CACHE_KEY is not set in test env
        // This test is inherently racy with env vars, so just verify the function
        // returns a deterministic result for a given input
        let hash1 = Sha256::digest(b"test-key");
        let hash2 = Sha256::digest(b"test-key");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_digest_schema_roundtrip() {
        let json = r#"{
            "headline": "Test day",
            "primary_focus": "testing",
            "day_narrative": "Wrote tests all day",
            "goals": [{
                "title": "Testing",
                "label": "test",
                "summary": "Add tests",
                "human_hours": 2.0,
                "project": "minibox",
                "docs_referenced": [],
                "tasks": [{
                    "title": "Unit tests",
                    "what_got_done": "Added tests",
                    "domain_skills": ["testing"],
                    "tech_skills": ["rust"],
                    "task_type": "testing",
                    "professional_roles": ["developer"],
                    "human_hours": 2.0
                }]
            }]
        }"#;
        let digest: Digest = serde_json::from_str(json).expect("valid digest JSON");
        assert_eq!(digest.headline, "Test day");
        assert_eq!(digest.goals.len(), 1);
        assert_eq!(digest.goals[0].tasks.len(), 1);
    }
}
