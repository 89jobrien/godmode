//! Test-coverage check for a single Rust source file.

use anyhow::Result;
use godmode_core::test_check;
use std::path::Path;

pub fn run_test_check(root: &Path, json: bool, path: String) -> Result<()> {
    match test_check::check_test_coverage(&path, root) {
        Some(msg) => {
            if json {
                println!("{}", serde_json::json!({"covered": false, "message": msg}));
            } else {
                eprintln!("{msg}");
            }
            std::process::exit(2);
        }
        None => {
            if json {
                println!("{}", serde_json::json!({"covered": true}));
            }
            Ok(())
        }
    }
}
