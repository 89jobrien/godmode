//! Test-coverage check for a single Rust source file.

use anyhow::Result;
use godmode_core::{detect, test_check};

pub fn run_test_check(json: bool, path: String) -> Result<()> {
    let git_root =
        detect::root_or_cwd().unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    match test_check::check_test_coverage(&path, &git_root) {
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
