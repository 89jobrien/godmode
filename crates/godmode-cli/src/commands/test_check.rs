//! Checks whether a Rust source path has corresponding test coverage.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Evaluates test coverage for the requested path and exits nonzero when coverage is absent.
pub fn handle(command: Cmd, _root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::TestCheck { path } => {
            use godmode_core::test_check;

            let git_root = detect::root_or_cwd()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
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
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
