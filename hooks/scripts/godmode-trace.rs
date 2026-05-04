#!/usr/bin/env rust-script
//! Emit session.start / session.end JSONL events to .ctx/GODMODE.trace.jsonl
//!
//! Usage:
//!   godmode-trace start <git-root>
//!   godmode-trace end   <git-root>
//!
//! Always exits 0 — failures are silent.
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize)]
struct Session {
    session_id: String,
    started_at: String,
}

fn epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn iso_now() -> String {
    // RFC3339-ish via SystemTime — no chrono dep needed for hook use
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = secs / 3600 % 24;
    let m = secs / 60 % 60;
    let s = secs % 60;
    // Date portion via days since epoch
    let days = secs / 86400;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut y = 1970u64;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let dy = if leap { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let month_days: &[u64] = if leap {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1u64;
    for &md in month_days {
        if days < md {
            break;
        }
        days -= md;
        mo += 1;
    }
    (y, mo, days + 1)
}

fn git_short_sha(git_root: &PathBuf) -> String {
    process::Command::new("git")
        .args(["-C", git_root.to_str().unwrap_or("."), "rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn read_session_id(ctx_dir: &PathBuf) -> Option<String> {
    let session_file = ctx_dir.join("GODMODE.session.json");
    let raw = fs::read_to_string(session_file).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    v.get("session_id")?.as_str().map(|s| s.to_string())
}

fn append_event(ctx_dir: &PathBuf, event: Value) {
    let trace_file = ctx_dir.join("GODMODE.trace.jsonl");
    if let Ok(line) = serde_json::to_string(&event) {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&trace_file) {
            let _ = writeln!(f, "{}", line);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: godmode-trace <start|end> <git-root>");
        process::exit(0);
    }
    let cmd = &args[1];
    let git_root = PathBuf::from(&args[2]);
    let ctx_dir = git_root.join(".ctx");

    let _ = fs::create_dir_all(&ctx_dir);

    match cmd.as_str() {
        "start" => {
            // Always create a fresh session on start
            let sha = git_short_sha(&git_root);
            let sid = format!("{}-{}", sha, epoch_ms());
            let session = Session {
                session_id: sid.clone(),
                started_at: iso_now(),
            };
            if let Ok(json) = serde_json::to_string(&session) {
                let _ = fs::write(ctx_dir.join("GODMODE.session.json"), json);
            }
            append_event(&ctx_dir, serde_json::json!({
                "event": "session.start",
                "session_id": sid,
                "cwd": git_root.to_str().unwrap_or(""),
                "ts": iso_now(),
            }));
            eprintln!("[godmode] session started: {sid}");
        }
        "end" => {
            if let Some(session_id) = read_session_id(&ctx_dir) {
                append_event(&ctx_dir, serde_json::json!({
                    "event": "session.end",
                    "session_id": session_id,
                    "ts": iso_now(),
                }));
                eprintln!("[godmode] session ended: {session_id}");
            }
        }
        _ => {
            eprintln!("[godmode-trace] unknown command: {cmd}");
        }
    }

    process::exit(0);
}
