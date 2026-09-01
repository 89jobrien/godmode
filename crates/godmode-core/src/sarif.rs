//! SARIF v2.1.0 output for verify and review reports.
//!
//! Implements the minimum SARIF schema needed to produce valid output
//! consumable by GitHub Code Scanning, VS Code SARIF Viewer, etc.

#[path = "sarif_builder.rs"]
/// Converters from godmode reports and tool output into SARIF logs.
pub mod sarif_builder;
#[path = "sarif_types.rs"]
/// Serializable types for the supported SARIF v2.1.0 subset.
pub mod sarif_types;

// Re-export public types so callers using `use godmode_core::sarif::*` are unaffected.
pub use sarif_builder::{
    clippy_sarif, clippy_sarif_from_json, from_review, from_verify, globstar_sarif,
    globstar_sarif_from_text,
};
pub use sarif_types::{
    ArtifactLocation, Location, Message, PhysicalLocation, Region, ReportingDescriptor, Result_,
    Run, SarifLog, Tool, ToolComponent,
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::sarif_builder::{
        location_from_skill, parse_file_line_col_message, strip_ansi, truncate,
    };
    use super::*;
    use crate::review::{Finding, ReviewReport, Severity};
    use crate::verify::{StepResult, VerifyReport};

    fn make_step(name: &str, ok: bool, output: &str) -> StepResult {
        StepResult {
            name: name.into(),
            ok,
            output: output.into(),
        }
    }

    #[test]
    fn verify_all_pass_produces_empty_results() {
        let report = VerifyReport {
            steps: vec![
                make_step("nextest", true, ""),
                make_step("clippy", true, ""),
            ],
            passed: true,
        };
        let sarif = from_verify(&report);
        assert_eq!(sarif.version, "2.1.0");
        assert_eq!(sarif.runs.len(), 1);
        assert!(sarif.runs[0].results.is_empty());
        assert_eq!(sarif.runs[0].tool.driver.rules.len(), 2);
    }

    #[test]
    fn verify_failure_produces_error_result() {
        let report = VerifyReport {
            steps: vec![
                make_step("nextest", false, "test xyz failed"),
                make_step("clippy", true, ""),
            ],
            passed: false,
        };
        let sarif = from_verify(&report);
        assert_eq!(sarif.runs[0].results.len(), 1);
        let r = &sarif.runs[0].results[0];
        assert_eq!(r.rule_id, "godmode-verify/nextest");
        assert_eq!(r.level, "error");
        assert!(r.message.text.contains("test xyz failed"));
    }

    #[test]
    fn review_findings_map_severity_to_level() {
        let report = ReviewReport {
            checks: 3,
            findings: vec![
                Finding {
                    skill: "brainstorm".into(),
                    check: "missing SKILL.md".into(),
                    message: "[brainstorm] missing SKILL.md".into(),
                    severity: Severity::Blocking,
                },
                Finding {
                    skill: "cap".into(),
                    check: "naming convention".into(),
                    message: "[cap] bad name".into(),
                    severity: Severity::Suggestion,
                },
                Finding {
                    skill: "merge".into(),
                    check: "whitespace".into(),
                    message: "[merge] trailing ws".into(),
                    severity: Severity::Nitpick,
                },
            ],
            passed: false,
        };
        let sarif = from_review(&report);
        let results = &sarif.runs[0].results;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].level, "error");
        assert_eq!(results[1].level, "warning");
        assert_eq!(results[2].level, "note");
    }

    #[test]
    fn review_deduplicates_rules() {
        let report = ReviewReport {
            checks: 2,
            findings: vec![
                Finding {
                    skill: "a".into(),
                    check: "missing SKILL.md".into(),
                    message: "a".into(),
                    severity: Severity::Blocking,
                },
                Finding {
                    skill: "b".into(),
                    check: "missing SKILL.md".into(),
                    message: "b".into(),
                    severity: Severity::Blocking,
                },
            ],
            passed: false,
        };
        let sarif = from_review(&report);
        assert_eq!(sarif.runs[0].results.len(), 2);
        assert_eq!(sarif.runs[0].tool.driver.rules.len(), 1);
    }

    #[test]
    fn location_maps_skill_to_file() {
        let locs = location_from_skill("brainstorm");
        assert_eq!(locs.len(), 1);
        assert_eq!(
            locs[0].physical_location.artifact_location.uri,
            "skills/brainstorm/SKILL.md"
        );
    }

    #[test]
    fn location_maps_lib_to_file() {
        let locs = location_from_skill("_lib/trace.nu");
        assert_eq!(locs.len(), 1);
        assert_eq!(
            locs[0].physical_location.artifact_location.uri,
            "skills/_lib/trace.nu"
        );
    }

    #[test]
    fn location_empty_for_special_names() {
        assert!(location_from_skill("plugin.json").is_empty());
        assert!(location_from_skill("index").is_empty());
        assert!(location_from_skill("").is_empty());
    }

    #[test]
    fn sarif_serializes_to_valid_json() {
        let report = VerifyReport {
            steps: vec![make_step("fmt", false, "diff found")],
            passed: false,
        };
        let sarif = from_verify(&report);
        let json = serde_json::to_string_pretty(&sarif).unwrap();
        assert!(json.contains("\"$schema\""));
        assert!(json.contains("\"version\": \"2.1.0\""));
        assert!(json.contains("godmode-verify/fmt"));
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        let s = "hello world";
        assert_eq!(truncate(s, 5), "hello");
        assert_eq!(truncate(s, 100), s);
        let mb = "a\u{20AC}b";
        let t = truncate(mb, 2);
        assert_eq!(t, "a");
    }

    #[test]
    fn clippy_json_parses_warning() {
        let line = r#"{"reason":"compiler-message","package_id":"foo 0.1.0","target":{"kind":["lib"],"name":"foo"},"message":{"message":"unused variable: `x`","code":{"code":"unused_variables","explanation":null},"level":"warning","spans":[{"file_name":"src/lib.rs","byte_start":100,"byte_end":101,"line_start":10,"line_end":10,"column_start":9,"column_end":10,"is_primary":true,"text":[],"label":null}],"children":[],"rendered":"warning: unused variable"}}"#;
        let sarif = clippy_sarif_from_json(line);
        assert_eq!(sarif.runs[0].results.len(), 1);
        let r = &sarif.runs[0].results[0];
        assert_eq!(r.rule_id, "unused_variables");
        assert_eq!(r.level, "warning");
        assert!(r.message.text.contains("unused variable"));
        assert_eq!(r.locations.len(), 1);
        let loc = &r.locations[0].physical_location;
        assert_eq!(loc.artifact_location.uri, "src/lib.rs");
        let region = loc.region.as_ref().unwrap();
        assert_eq!(region.start_line, 10);
        assert_eq!(region.start_column, Some(9));
    }

    #[test]
    fn clippy_json_skips_non_diagnostic_lines() {
        let input = concat!(
            r#"{"reason":"compiler-artifact","package_id":"foo"}"#,
            "\n",
            r#"{"reason":"build-finished","success":true}"#,
            "\n",
            "not json at all\n",
        );
        let sarif = clippy_sarif_from_json(input);
        assert!(sarif.runs[0].results.is_empty());
    }

    #[test]
    fn clippy_json_deduplicates_rules() {
        let line1 = r#"{"reason":"compiler-message","package_id":"a","target":{"kind":["lib"],"name":"a"},"message":{"message":"unused x","code":{"code":"unused_variables","explanation":null},"level":"warning","spans":[{"file_name":"a.rs","byte_start":0,"byte_end":1,"line_start":1,"line_end":1,"column_start":1,"column_end":2,"is_primary":true,"text":[],"label":null}],"children":[],"rendered":""}}"#;
        let line2 = r#"{"reason":"compiler-message","package_id":"a","target":{"kind":["lib"],"name":"a"},"message":{"message":"unused y","code":{"code":"unused_variables","explanation":null},"level":"warning","spans":[{"file_name":"b.rs","byte_start":0,"byte_end":1,"line_start":5,"line_end":5,"column_start":1,"column_end":2,"is_primary":true,"text":[],"label":null}],"children":[],"rendered":""}}"#;
        let input = format!("{line1}\n{line2}");
        let sarif = clippy_sarif_from_json(&input);
        assert_eq!(sarif.runs[0].results.len(), 2);
        assert_eq!(sarif.runs[0].tool.driver.rules.len(), 1);
    }

    #[test]
    fn globstar_parses_ansi_output() {
        let input = "\x1b[90m9:06PM\x1b[0m \x1b[31mERR\x1b[0m \x1b[1m/home/user/project/src/lib.rs:15:25:Prefer error handling over .unwrap()\x1b[0m\n";
        let root = std::path::Path::new("/home/user/project");
        let sarif = globstar_sarif_from_text(input, root);
        assert_eq!(sarif.runs[0].results.len(), 1);
        let r = &sarif.runs[0].results[0];
        assert_eq!(r.level, "warning");
        assert!(r.message.text.contains("unwrap"));
        let loc = &r.locations[0].physical_location;
        assert_eq!(loc.artifact_location.uri, "src/lib.rs");
        let region = loc.region.as_ref().unwrap();
        assert_eq!(region.start_line, 15);
        assert_eq!(region.start_column, Some(25));
    }

    #[test]
    fn globstar_skips_non_diagnostic_lines() {
        let input = "some random log line\n\n";
        let root = std::path::Path::new("/tmp");
        let sarif = globstar_sarif_from_text(input, root);
        assert!(sarif.runs[0].results.is_empty());
    }

    #[test]
    fn globstar_multiple_findings() {
        let input = concat!(
            "9:06PM ERR /proj/a.rs:1:1:msg one\n",
            "9:06PM ERR /proj/b.rs:10:5:msg two\n",
        );
        let root = std::path::Path::new("/proj");
        let sarif = globstar_sarif_from_text(input, root);
        assert_eq!(sarif.runs[0].results.len(), 2);
        assert_eq!(
            sarif.runs[0].results[0].locations[0]
                .physical_location
                .artifact_location
                .uri,
            "a.rs"
        );
        assert_eq!(
            sarif.runs[0].results[1].locations[0]
                .physical_location
                .artifact_location
                .uri,
            "b.rs"
        );
    }

    #[test]
    fn strip_ansi_removes_escapes() {
        let s = "\x1b[31mred\x1b[0m normal";
        assert_eq!(strip_ansi(s), "red normal");
    }

    #[test]
    fn parse_file_line_col_message_works() {
        let r = parse_file_line_col_message("/a/b.rs:10:5:hello world");
        assert_eq!(
            r,
            Some(("/a/b.rs".to_string(), 10, 5, "hello world".to_string()))
        );
    }
}
