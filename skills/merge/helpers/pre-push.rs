#!/usr/bin/env rust-script
//! Pre-push hook — resolves the commit range being pushed and prints it to stdout.
//!
//! Git calls this hook with:
//!   $1  remote name
//!   $2  remote URL
//!
//! Stdin contains one line per ref being pushed:
//!   <local_ref> <local_oid> <remote_ref> <remote_oid>
//!
//! Output: one "base..tip" range per ref, written to stdout.
//! Downstream tools (e.g. obfsck) read this range to determine what to inspect.
//!
//! Range resolution rules:
//!   local_oid  == zero  → deletion, skip (nothing to inspect)
//!   remote_oid == zero  → new branch: base = merge-base against main (fallback: initial commit)
//!   otherwise           → existing branch: base = remote_oid
//!
//! ```cargo
//! [dependencies]
//! ```

use std::io::{self, BufRead};
use std::process::{Command, Stdio};

fn zero_oid() -> String {
    let out = Command::new("git")
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::null())
        .output()
        .expect("git hash-object failed");
    let hex = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Replace every hex digit with '0'.
    hex.chars().map(|_| '0').collect()
}

fn merge_base(oid: &str) -> String {
    // Try merge-base against main first.
    let result = Command::new("git")
        .args(["merge-base", oid, "main"])
        .output();

    if let Ok(out) = result {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }

    // Fallback: initial commit.
    let out = Command::new("git")
        .args(["rev-list", "--max-parents=0", "HEAD"])
        .output()
        .expect("git rev-list failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn resolve_ranges() {
    let zero = zero_oid();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.expect("stdin read error");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let local_oid = parts[1];
        let remote_oid = parts[3];

        if local_oid == zero {
            // Deletion — nothing to inspect.
            continue;
        }

        let base = if remote_oid == zero {
            // New branch — inspect commits since divergence from main.
            merge_base(local_oid)
        } else {
            // Existing branch — inspect only new commits.
            remote_oid.to_string()
        };

        println!("{base}..{local_oid}");
    }
}

fn main() {
    resolve_ranges();

    // Quality gate — mirrors .git/hooks/pre-push
    let status = Command::new("cargo")
        .args(["xtask", "prepush"])
        .status()
        .expect("failed to run cargo xtask prepush");

    if !status.success() {
        std::process::exit(1);
    }
}
