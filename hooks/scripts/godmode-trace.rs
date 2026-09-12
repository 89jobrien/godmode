#!/usr/bin/env rust-script
//! Emit session.start / session.end JSONL events to .ctx/godmode/traces/trace.jsonl
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
    io::{self, Write},
    path::PathBuf,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

const SECS_PER_MIN: u64 = 60;
const MINS_PER_HOUR: u64 = 60;
const SECS_PER_HOUR: u64 = SECS_PER_MIN * MINS_PER_HOUR;
const HOURS_PER_DAY: u64 = 24;
const SECS_PER_DAY: u64 = SECS_PER_HOUR * HOURS_PER_DAY;
const EPOCH_YEAR: u64 = 1970;
const DAYS_PER_YEAR: u64 = 365;
const DAYS_PER_LEAP_YEAR: u64 = 366;
const FIRST_MONTH: u64 = 1;
const FIRST_DAY: u64 = 1;

const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_MONTH_DAYS: u64 = 29;
const FEBRUARY_INDEX: usize = 1;
const LEAP_CYCLE_4: u64 = 4;
const LEAP_CYCLE_100: u64 = 100;
const LEAP_CYCLE_400: u64 = 400;

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
    let h = secs / SECS_PER_HOUR % HOURS_PER_DAY;
    let m = secs / SECS_PER_MIN % MINS_PER_HOUR;
    let s = secs % SECS_PER_MIN;
    // Date portion via days since epoch
    let days = secs / SECS_PER_DAY;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut y = EPOCH_YEAR;
    let is_leap =
        |y: u64| (y % LEAP_CYCLE_4 == 0 && y % LEAP_CYCLE_100 != 0) || y % LEAP_CYCLE_400 == 0;
    loop {
        let dy = if is_leap(y) {
            DAYS_PER_LEAP_YEAR
        } else {
            DAYS_PER_YEAR
        };
        if days < dy {
            break;
        }
        days -= dy;
        y += 1;
    }
    let leap = is_leap(y);
    let mut leap_month = DAYS_IN_MONTH;
    if leap {
        leap_month[FEBRUARY_INDEX] = LEAP_MONTH_DAYS;
    }
    let month_days: &[u64] = if leap { &leap_month } else { &DAYS_IN_MONTH };
    let mut mo = FIRST_MONTH;
    for &md in month_days {
        if days < md {
            break;
        }
        days -= md;
        mo += 1;
    }
    (y, mo, days + FIRST_DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_day_is_1970_01_01() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn leap_day_is_preserved() {
        assert_eq!(days_to_ymd(11_016), (2000, 2, 29));
    }

    #[test]
    fn non_leap_century_rolls_into_march() {
        assert_eq!(days_to_ymd(47_541), (2100, 3, 1));
    }

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "godmode-trace-{label}-{}-{}",
            process::id(),
            epoch_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn start_then_end_persists_one_session_lifecycle() {
        let root = temp_root("lifecycle");
        handle_command("start", &root).unwrap();
        let ctx = root.join(".ctx/godmode");
        let session = read_session_id(&ctx).expect("session id");
        handle_command("end", &root).unwrap();
        let lines = fs::read_to_string(ctx.join("traces/trace.jsonl")).unwrap();
        let events = lines
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event"], "session.start");
        assert_eq!(events[1]["event"], "session.end");
        assert_eq!(events[1]["session_id"], session);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_session_is_ignored_and_concurrent_appends_remain_jsonl() {
        let root = temp_root("io");
        let ctx = root.join(".ctx/godmode");
        fs::create_dir_all(ctx.join("traces")).unwrap();
        fs::write(ctx.join("session.json"), "{").unwrap();
        handle_command("end", &root).unwrap();
        assert!(!ctx.join("traces/trace.jsonl").exists());
        let mut threads = Vec::new();
        for index in 0..32 {
            let ctx = ctx.clone();
            threads.push(std::thread::spawn(move || {
                append_event(&ctx, serde_json::json!({"index": index}))
            }));
        }
        for thread in threads {
            thread.join().unwrap().unwrap();
        }
        let lines = fs::read_to_string(ctx.join("traces/trace.jsonl")).unwrap();
        assert_eq!(lines.lines().count(), 32);
        assert!(lines
            .lines()
            .all(|line| serde_json::from_str::<Value>(line).is_ok()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unknown_commands_are_rejected_without_creating_state() {
        let root = temp_root("unknown");
        let error = handle_command("bogus", &root).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(!root.join(".ctx").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn directory_and_open_failures_are_reported() {
        let root = temp_root("directory-failure");
        fs::write(root.join(".ctx"), "not a directory").unwrap();
        assert!(handle_command("start", &root).is_err());
        fs::remove_dir_all(&root).unwrap();

        let root = temp_root("open-failure");
        let ctx = root.join(".ctx/godmode");
        fs::create_dir_all(ctx.join("traces/trace.jsonl")).unwrap();
        assert!(append_event(&ctx, serde_json::json!({"event": "test"})).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn write_failures_are_reported() {
        struct FailingWriter;
        impl std::io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected write failure"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert!(
            write_json_line(&mut FailingWriter, &serde_json::json!({"event": "test"})).is_err()
        );
    }
}

fn git_short_sha(git_root: &PathBuf) -> String {
    process::Command::new("git")
        .args([
            "-C",
            git_root.to_str().unwrap_or("."),
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn read_session_id(ctx_dir: &PathBuf) -> Option<String> {
    let session_file = ctx_dir.join("session.json");
    let raw = fs::read_to_string(session_file).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    v.get("session_id")?.as_str().map(|s| s.to_string())
}

fn write_json_line(writer: &mut impl Write, event: &Value) -> io::Result<()> {
    let line = serde_json::to_string(event).map_err(io::Error::other)?;
    let mut buf = line.into_bytes();
    buf.push(b'\n');
    writer.write_all(&buf)
}

fn append_event(ctx_dir: &PathBuf, event: Value) -> io::Result<()> {
    let trace_file = ctx_dir.join("traces").join("trace.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(trace_file)?;
    write_json_line(&mut file, &event)
}

fn handle_command(cmd: &str, git_root: &PathBuf) -> io::Result<()> {
    if !matches!(cmd, "start" | "end") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown command: {cmd}"),
        ));
    }

    let ctx_dir = git_root.join(".ctx").join("godmode");
    fs::create_dir_all(ctx_dir.join("traces"))?;

    match cmd {
        "start" => {
            let sha = git_short_sha(git_root);
            let sid = format!("{}-{}", sha, epoch_ms());
            let session = Session {
                session_id: sid.clone(),
                started_at: iso_now(),
            };
            let json = serde_json::to_string(&session).map_err(io::Error::other)?;
            fs::write(ctx_dir.join("session.json"), json)?;
            append_event(
                &ctx_dir,
                serde_json::json!({
                    "event": "session.start", "session_id": sid,
                    "cwd": git_root.to_str().unwrap_or(""), "ts": iso_now(),
                }),
            )?;
            eprintln!("[godmode] session started: {sid}");
        }
        "end" => {
            if let Some(session_id) = read_session_id(&ctx_dir) {
                append_event(
                    &ctx_dir,
                    serde_json::json!({
                        "event": "session.end", "session_id": session_id, "ts": iso_now(),
                    }),
                )?;
                eprintln!("[godmode] session ended: {session_id}");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: godmode-trace <start|end> <git-root>");
        return;
    }
    if let Err(error) = handle_command(&args[1], &PathBuf::from(&args[2])) {
        if error.kind() == io::ErrorKind::InvalidInput {
            eprintln!("[godmode-trace] {error}");
        }
    }
}
