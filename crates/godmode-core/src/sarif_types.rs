//! SARIF v2.1.0 serde structs (minimal subset).

use serde::Serialize;

/// Top-level SARIF log document.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    /// URI of the SARIF JSON schema.
    pub schema: &'static str,
    /// SARIF format version.
    pub version: &'static str,
    /// Analysis runs contained in the log.
    pub runs: Vec<Run>,
}

/// One analysis invocation and its results.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    /// Tool that produced the run.
    pub tool: Tool,
    /// Findings produced by the run.
    pub results: Vec<Result_>,
}

/// SARIF tool wrapper.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Primary component that produced results.
    pub driver: ToolComponent,
}

/// Identity and rules of an analysis tool component.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolComponent {
    /// Tool component name.
    pub name: String,
    /// Optional tool component version.
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    /// Rules reported or evaluated by the tool.
    pub rules: Vec<ReportingDescriptor>,
}

/// Metadata describing one reporting rule.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportingDescriptor {
    /// Stable rule identifier.
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional short description of the rule.
    pub short_description: Option<Message>,
}

/// One SARIF finding.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Result_ {
    /// Identifier of the rule that produced the finding.
    pub rule_id: String,
    /// SARIF severity level.
    pub level: &'static str,
    /// Finding message.
    pub message: Message,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    /// Source locations associated with the finding.
    pub locations: Vec<Location>,
}

/// Textual SARIF message.
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    /// Plain-text message content.
    pub text: String,
}

/// Source location associated with a finding.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    /// Physical file and region information.
    pub physical_location: PhysicalLocation,
}

/// File location and optional source region.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalLocation {
    /// Artifact containing the finding.
    pub artifact_location: ArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional region within the artifact.
    pub region: Option<Region>,
}

/// URI of an artifact referenced by a SARIF result.
#[derive(Debug, Serialize)]
pub struct ArtifactLocation {
    /// Artifact URI, typically a project-relative path.
    pub uri: String,
}

/// One-based source coordinates for a finding.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    /// First line of the region.
    pub start_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// First column of the region, when known.
    pub start_column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Last line of the region, when known.
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Last column of the region, when known.
    pub end_column: Option<u32>,
}

/// Canonical JSON schema URI for SARIF v2.1.0.
pub const SCHEMA: &str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json";
/// SARIF version emitted by this module.
pub const VERSION: &str = "2.1.0";
