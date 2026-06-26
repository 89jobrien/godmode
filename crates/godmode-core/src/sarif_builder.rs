//! Builder helpers that construct SARIF from ReviewReport / VerifyReport / clippy JSON.

use crate::review::{self, Severity};
use crate::sarif::sarif_types::{
    ArtifactLocation, Location, Message, PhysicalLocation, Region, ReportingDescriptor, Result_,
    Run, SCHEMA, SarifLog, Tool, ToolComponent, VERSION,
};
use crate::verify;

// ── VerifyReport -> SARIF ───────────────────────────────────────────

/// Convert a `VerifyReport` into a SARIF log.
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

// ── ReviewReport -> SARIF ───────────────────────────────────────────

/// Convert a `ReviewReport` into a SARIF log.
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

// ── Clippy JSON -> SARIF ────────────────────────────────────────────

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

/// Parse cargo clippy JSON lines into a SARIF log.
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

// ── Globstar -> SARIF ───────────────────────────────────────────────

/// Run `globstar check --checkers=local` and convert output to SARIF.
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

/// Parse globstar text output into SARIF.
pub fn globstar_sarif_from_text(text: &str, root: &std::path::Path) -> SarifLog {
    let mut results = Vec::new();
    let mut rule_ids_seen: Vec<String> = Vec::new();
    let mut rules = Vec::new();
    let root_prefix = format!("{}/", root.display());

    for line in text.lines() {
        let clean = strip_ansi(line);
        let diagnostic = extract_globstar_diagnostic(&clean);
        let Some((file, line_num, col, message)) = diagnostic else {
            continue;
        };

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

// ── Internal helpers ────────────────────────────────────────────────

pub(crate) fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &s[..end]
    }
}

pub(crate) fn location_from_skill(skill: &str) -> Vec<Location> {
    if skill.is_empty() || skill == "plugin.json" || skill == "index" {
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

pub(crate) fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
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

pub(crate) fn extract_globstar_diagnostic(line: &str) -> Option<(String, u32, u32, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path_start = trimmed.find('/')?;
    let rest = &trimmed[path_start..];
    parse_file_line_col_message(rest)
}

pub(crate) fn parse_file_line_col_message(s: &str) -> Option<(String, u32, u32, String)> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b':' {
            let line_start = i + 1;
            let mut j = line_start;
            while j < len && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > line_start && j < len && bytes[j] == b':' {
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
