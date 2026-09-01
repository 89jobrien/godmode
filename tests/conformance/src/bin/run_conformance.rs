//! CLI entry point: run all godmode conformance tests and print a report.
//!
//! Usage:
//! `cargo run -p godmode-conformance --bin run-conformance -- [--json] [--filter <name>]`

#![deny(missing_docs)]

use godmode_conformance::harness::{ReportConfig, ReportGenerator};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json_mode = args.iter().any(|a| a == "--json");
    let filter = args.windows(2).find_map(|w| {
        if w[0] == "--filter" {
            Some(w[1].clone())
        } else {
            None
        }
    });
    let ci_mode = std::env::var("CI").is_ok() || args.iter().any(|a| a == "--ci");

    let mut runner = godmode_conformance::all_tests();
    if let Some(ref pat) = filter {
        // Re-build with name filter — reconstruct since TestRunner is consumed
        runner = godmode_conformance::all_tests().filter_name(pat);
    }

    let summary = runner.run();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if json_mode {
        ReportGenerator::json(&mut out, &summary).unwrap();
    } else if ci_mode {
        ReportGenerator::github_actions(&mut out, &summary).unwrap();
    } else {
        let config = ReportConfig {
            verbose: args.iter().any(|a| a == "--verbose"),
            summary_only: args.iter().any(|a| a == "--summary"),
            show_timing: true,
        };
        ReportGenerator::text(&mut out, &summary, &config).unwrap();
    }

    if !summary.is_success() {
        std::process::exit(1);
    }
}
