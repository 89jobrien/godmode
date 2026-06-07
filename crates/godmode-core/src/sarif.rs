//! SARIF v2.1.0 output for verify and review reports.
//!
//! Implements the minimum SARIF schema needed to produce valid output
//! consumable by GitHub Code Scanning, VS Code SARIF Viewer, etc.

use serde::Serialize;

use crate::review::{self, Severity};
use crate::verify;

// ---------------------------------------------------------------------------
// SARIF v2.1.0 types (minimal subset)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    pub schema: &'static str,
    pub version: &'static str,
    pub runs: Vec<Run>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub tool: Tool,
    pub results: Vec<Result_>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub driver: ToolComponent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolComponent {
    pub name: String,
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ReportingDescriptor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportingDescriptor {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<Message>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Result_ {
    pub rule_id: String,
    pub level: &'static str,
    pub message: Message,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<Location>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub physical_location: PhysicalLocation,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalLocation {
    pub artifact_location: ArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
}

#[derive(Debug, Serialize)]
pub struct ArtifactLocation {
    pub uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub start_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SCHEMA: &str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json";
const VERSION: &str = "2.1.0";

// ---------------------------------------------------------------------------
// VerifyReport -> SARIF
// ---------------------------------------------------------------------------

/// Convert a `VerifyReport` into a SARIF log. Each verification step that
/// failed becomes a result; passing steps are omitted.
pub fn from_verify(report: &verify::VerifyReport) -> SarifLog {
    let mut results = Vec::new();
    let mut rules = Vec::new();

    for step in &report.steps {
        let rule_id = format!("godmode-verify/{}", step.name);
        rules.push(ReportingDescriptor {
            id: rule_id.clone(),
            short_description: Some(Message {
                text: format!("{} verification step", step.name),
            }),
        });

        if !step.ok {
            let message_text = if step.output.is_empty() {
                format!("{} failed", step.name)
            } else {
                // Truncate long output to keep SARIF readable
                let truncated = truncate(&step.output, 2000);
                truncated.to_string()
            };

            results.push(Result_ {
                rule_id,
                level: "error",
                message: Message { text: message_text },
                locations: vec![],
            });
        }
    }

    SarifLog {
        schema: SCHEMA,
        version: VERSION,
        runs: vec![Run {
            tool: Tool {
                driver: ToolComponent {
                    name: "godmode-verify".into(),
                    version: None,
                    rules,
                },
            },
            results,
        }],
    }
}

// ---------------------------------------------------------------------------
// ReviewReport -> SARIF
// ---------------------------------------------------------------------------

/// Convert a `ReviewReport` into a SARIF log. Each finding becomes a result
/// with level mapped from the finding's severity.
pub fn from_review(report: &review::ReviewReport) -> SarifLog {
    let mut results = Vec::new();
    let mut rule_ids_seen: Vec<String> = Vec::new();
    let mut rules = Vec::new();

    for finding in &report.findings {
        let rule_id = format!("godmode-review/{}", finding.check);

        if !rule_ids_seen.contains(&rule_id) {
            rule_ids_seen.push(rule_id.clone());
            rules.push(ReportingDescriptor {
                id: rule_id.clone(),
                short_description: Some(Message {
                    text: finding.check.clone(),
                }),
            });
        }

        let level = match finding.severity {
            Severity::Blocking => "error",
            Severity::Suggestion => "warning",
            Severity::Nitpick => "note",
        };

        // If we can extract a file path from the skill name, add a location
        let locations = location_from_skill(&finding.skill);

        results.push(Result_ {
            rule_id,
            level,
            message: Message {
                text: finding.message.clone(),
            },
            locations,
        });
    }

    SarifLog {
        schema: SCHEMA,
        version: VERSION,
        runs: vec![Run {
            tool: Tool {
                driver: ToolComponent {
                    name: "godmode-review".into(),
                    version: None,
                    rules,
                },
            },
            results,
        }],
    }
}

// ---------------------------------------------------------------------------
// Clippy JSON -> SARIF
// ---------------------------------------------------------------------------

/// Cargo diagnostic message (subset of `--message-format=json` output).
#[derive(Debug, serde::Deserialize)]
struct CargoMessage {
    reason: String,
    #[serde(default)]
    message: Option<DiagnosticMessage>,
}

#[derive(Debug, serde::Deserialize)]
struct DiagnosticMessage {
    message: String,
    #[serde(default)]
    code: Option<DiagnosticCode>,
    level: String,
    #[serde(default)]
    spans: Vec<DiagnosticSpan>,
}

#[derive(Debug, serde::Deserialize)]
struct DiagnosticCode {
    code: String,
}

#[derive(Debug, serde::Deserialize)]
struct DiagnosticSpan {
    file_name: String,
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
    is_primary: bool,
}

/// Run `cargo clippy --message-format=json` and convert the output to SARIF.
///
/// This gives richer results than `from_verify` for clippy — each diagnostic
/// includes the exact file, line, column, and lint code.
pub fn clippy_sarif(root: &std::path::Path, crate_name: Option<&str>) -> anyhow::Result<SarifLog> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("clippy");
    match crate_name {
        Some(name) => {
            cmd.args(["-p", name]);
        }
        None => {
            cmd.arg("--workspace");
        }
    }
    cmd.args(["--message-format=json", "--", "-D", "warnings"]);
    cmd.current_dir(root);
    let output = cmd.output()?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(clippy_sarif_from_json(&combined))
}

/// Parse cargo clippy JSON lines into a SARIF log. Exposed for testing
/// without running cargo.
pub fn clippy_sarif_from_json(json_lines: &str) -> SarifLog {
    let mut results = Vec::new();
    let mut rule_ids_seen: Vec<String> = Vec::new();
    let mut rules = Vec::new();

    for line in json_lines.lines() {
        let msg: CargoMessage = match serde_json::from_str(line) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if msg.reason != "compiler-message" {
            continue;
        }

        let diag = match msg.message {
            Some(d) => d,
            None => continue,
        };

        // Skip non-diagnostic noise (build-finished, etc.)
        let level = match diag.level.as_str() {
            "error" => "error",
            "warning" => "warning",
            "note" | "help" => "note",
            _ => continue,
        };

        let rule_id = diag
            .code
            .as_ref()
            .map(|c| c.code.clone())
            .unwrap_or_else(|| format!("clippy/{}", diag.level));

        if !rule_ids_seen.contains(&rule_id) {
            rule_ids_seen.push(rule_id.clone());
            rules.push(ReportingDescriptor {
                id: rule_id.clone(),
                short_description: None,
            });
        }

        // Use the primary span for location
        let locations: Vec<Location> = diag
            .spans
            .iter()
            .filter(|s| s.is_primary)
            .map(|s| Location {
                physical_location: PhysicalLocation {
                    artifact_location: ArtifactLocation {
                        uri: s.file_name.clone(),
                    },
                    region: Some(Region {
                        start_line: s.line_start,
                        start_column: Some(s.column_start),
                        end_line: Some(s.line_end),
                        end_column: Some(s.column_end),
                    }),
                },
            })
            .collect();

        results.push(Result_ {
            rule_id,
            level,
            message: Message { text: diag.message },
            locations,
        });
    }

    SarifLog {
        schema: SCHEMA,
        version: VERSION,
        runs: vec![Run {
            tool: Tool {
                driver: ToolComponent {
                    name: "clippy".into(),
                    version: None,
                    rules,
                },
            },
            results,
        }],
    }
}

// ---------------------------------------------------------------------------
// Globstar -> SARIF
// ---------------------------------------------------------------------------

/// Run `globstar check --checkers=local` and convert output to SARIF.
///
/// Globstar emits `file:line:col:message` on stderr with ANSI codes.
/// Returns `None` if globstar is not on PATH.
pub fn globstar_sarif(root: &std::path::Path) -> Option<SarifLog> {
    let globstar = which::which("globstar").ok()?;
    let output = std::process::Command::new(globstar)
        .args(["check", "--checkers=local"])
        .current_dir(root)
        .output()
        .ok()?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    Some(globstar_sarif_from_text(&combined, root))
}

/// Parse globstar text output into SARIF. Each line matching
/// `<file>:<line>:<col>:<message>` becomes a result.
/// Lines are stripped of ANSI escape codes and timestamp prefixes.
pub fn globstar_sarif_from_text(text: &str, root: &std::path::Path) -> SarifLog {
    let mut results = Vec::new();
    let mut rule_ids_seen: Vec<String> = Vec::new();
    let mut rules = Vec::new();
    let root_prefix = format!("{}/", root.display());

    for line in text.lines() {
        let clean = strip_ansi(line);
        // Globstar prefixes lines with timestamp + level, e.g.:
        // "9:06PM ERR /path/to/file.rs:15:25:message"
        // Find the first absolute path or relative path segment
        let diagnostic = extract_globstar_diagnostic(&clean);
        let Some((file, line_num, col, message)) = diagnostic else {
            continue;
        };

        // Make path relative to root
        let rel_path = file.strip_prefix(&root_prefix).unwrap_or(&file);

        let rule_id = "globstar/pattern-match".to_string();
        if !rule_ids_seen.contains(&rule_id) {
            rule_ids_seen.push(rule_id.clone());
            rules.push(ReportingDescriptor {
                id: rule_id.clone(),
                short_description: Some(Message {
                    text: "Globstar pattern match".into(),
                }),
            });
        }

        results.push(Result_ {
            rule_id,
            level: "warning",
            message: Message { text: message },
            locations: vec![Location {
                physical_location: PhysicalLocation {
                    artifact_location: ArtifactLocation {
                        uri: rel_path.to_string(),
                    },
                    region: Some(Region {
                        start_line: line_num,
                        start_column: Some(col),
                        end_line: None,
                        end_column: None,
                    }),
                },
            }],
        });
    }

    SarifLog {
        schema: SCHEMA,
        version: VERSION,
        runs: vec![Run {
            tool: Tool {
                driver: ToolComponent {
                    name: "globstar".into(),
                    version: None,
                    rules,
                },
            },
            results,
        }],
    }
}

/// Strip ANSI escape codes from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm' (SGR terminator) or end
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Extract `(file, line, col, message)` from a globstar output line.
/// Expected format after ANSI stripping:
/// `9:06PM ERR /abs/path/file.rs:15:25:message text`
fn extract_globstar_diagnostic(line: &str) -> Option<(String, u32, u32, String)> {
    // Find the file path — starts with '/' (absolute) after the level prefix
    // or look for a path-like segment containing '.rs', '.yml', etc.
    let trimmed = line.trim();

    // Skip empty lines
    if trimmed.is_empty() {
        return None;
    }

    // Find the absolute path start — first '/' that begins a file path
    let path_start = trimmed.find('/')?;
    let rest = &trimmed[path_start..];

    // Parse file:line:col:message — file path may contain colons only in
    // drive letters (Windows) which we don't handle here.
    // Strategy: split on ':', find the first two numeric segments after the path.
    parse_file_line_col_message(rest)
}

/// Parse `file:line:col:message` where file is an absolute path.
fn parse_file_line_col_message(s: &str) -> Option<(String, u32, u32, String)> {
    // Find the pattern `:digits:digits:` scanning from the end of what looks
    // like a file extension
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Walk forward to find `:number:number:` pattern
    while i < len {
        if bytes[i] == b':' {
            // Try to parse line number
            let line_start = i + 1;
            let mut j = line_start;
            while j < len && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > line_start && j < len && bytes[j] == b':' {
                // Try to parse column number
                let col_start = j + 1;
                let mut k = col_start;
                while k < len && bytes[k].is_ascii_digit() {
                    k += 1;
                }
                if k > col_start && k < len && bytes[k] == b':' {
                    let file = &s[..i];
                    let line: u32 = s[line_start..j].parse().ok()?;
                    let col: u32 = s[col_start..k].parse().ok()?;
                    let message = s[k + 1..].trim().to_string();
                    if !message.is_empty() && file.contains('.') {
                        return Some((file.to_string(), line, col, message));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        // Find a safe char boundary
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &s[..end]
    }
}

/// Best-effort location from a review finding's skill field.
/// Maps skill names like "brainstorm" to "skills/brainstorm/SKILL.md"
/// and special names like "_lib/trace.nu" to "skills/_lib/trace.nu".
fn location_from_skill(skill: &str) -> Vec<Location> {
    if skill.is_empty() || skill == "plugin.json" || skill == "index" {
        // These are special cases without a clean file mapping
        return vec![];
    }

    let uri = if skill.starts_with("_lib/") {
        format!("skills/{skill}")
    } else {
        format!("skills/{skill}/SKILL.md")
    };

    vec![Location {
        physical_location: PhysicalLocation {
            artifact_location: ArtifactLocation { uri },
            region: None,
        },
    }]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
        // Rules are still registered even when passing
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
        // Two results but only one rule (same check)
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
        // Multi-byte: euro sign is 3 bytes
        let mb = "a\u{20AC}b"; // 5 bytes total
        let t = truncate(mb, 2);
        assert_eq!(t, "a"); // can't split the euro sign
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
