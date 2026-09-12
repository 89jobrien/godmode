//! Agent definitions, catalog loading, migration, and generated agent documents.
//!
//! This module manages both Claude-compatible agent definitions and the OpenCode
//! workspace router catalog.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod opencode;

pub use opencode::{
    OpenCodeAgentCatalog, OpenCodePermission, OpenCodePermissions, OpenCodeProjectAgentDef,
    OpenCodeRouterDef, RenderedAgent, install_opencode_agents, install_opencode_agents_with_mode,
    load_opencode_catalog, render_opencode_agent_files, render_opencode_agents,
};

/// Attribution and classification metadata attached to an agent definition.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    /// Name of the agent definition's author.
    #[serde(default)]
    pub author: String,
    /// Searchable labels associated with the agent.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Hook invoked for a matching agent lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentHook {
    /// Lifecycle event that triggers the hook.
    pub event: String,
    /// Pattern used to decide whether the hook applies.
    pub matcher: String,
    /// Script executed when the hook matches.
    pub script: String,
}

/// Complete declarative configuration for a generated agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    /// Stable agent name exposed to users and tooling.
    pub name: String,
    /// Version of the agent definition.
    #[serde(default = "default_version")]
    pub version: String,
    /// Human-readable summary of the agent's purpose.
    #[serde(default)]
    pub description: String,
    /// Phrases or conditions that should activate the agent.
    #[serde(default)]
    pub triggers: Vec<String>,
    /// Model identifier or inheritance directive used by the agent.
    #[serde(default)]
    pub model: String,
    /// Display color used by compatible agent clients.
    #[serde(default)]
    pub color: String,
    /// Tools made available to the agent.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Skills preloaded for the agent.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Template name used to construct the agent prompt.
    #[serde(default)]
    pub template: String,
    /// Category used to group and name generated agent files.
    #[serde(default)]
    pub category: String,
    /// Optional inline prompt body.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Lifecycle hooks registered for the agent.
    #[serde(default)]
    pub hooks: Vec<AgentHook>,
    /// Attribution and classification metadata.
    #[serde(default)]
    pub metadata: AgentMetadata,
    /// Workflows exposed through this agent.
    #[serde(default)]
    pub workflows: Vec<WorkflowRef>,
}

/// Reference to a workflow exposed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRef {
    /// Workflow name shown to users.
    pub name: String,
    /// Path to the workflow definition.
    pub path: String,
    /// Optional slash command that invokes the workflow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slash_command: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Parse an agent YAML file.
///
/// # Errors
///
/// Returns an error when the file cannot be read or contains invalid YAML.
pub fn load(path: &Path) -> Result<AgentDef> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading agent file {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing agent YAML {}", path.display()))
}

/// Write an agent YAML file.
///
/// # Errors
///
/// Returns an error when serialization, directory creation, or writing fails.
pub fn save(path: &Path, agent: &AgentDef) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(agent)?;
    std::fs::write(path, yaml)?;
    Ok(())
}

/// Parse YAML frontmatter between `---` delimiters from a markdown file.
fn extract_frontmatter(content: &str) -> Option<&str> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    // Skip optional newline after opening ---
    let rest = rest.trim_start_matches('\n').trim_start_matches('\r');
    // Find closing ---
    if let Some(end) = rest.find("\n---") {
        Some(&rest[..end])
    } else {
        None
    }
}

/// Frontmatter as a raw serde_yaml::Value (loosely typed for migration).
fn parse_frontmatter_value(content: &str) -> Result<serde_yaml::Value> {
    let fm =
        extract_frontmatter(content).ok_or_else(|| anyhow::anyhow!("no YAML frontmatter found"))?;
    Ok(serde_yaml::from_str(fm)?)
}

/// Migrate an existing `agents/*.md` (frontmatter + prose) to an `agents/*.yaml` stub.
/// Returns the path of the written YAML file.
///
/// # Errors
///
/// Returns an error when Markdown cannot be read, frontmatter is absent or invalid, or YAML cannot be written.
pub fn migrate_md_to_yaml(md_path: &Path, out_dir: &Path) -> Result<PathBuf> {
    let content = std::fs::read_to_string(md_path)
        .with_context(|| format!("reading {}", md_path.display()))?;

    let val = parse_frontmatter_value(&content)
        .with_context(|| format!("frontmatter in {}", md_path.display()))?;

    let get_str = |key: &str| -> String {
        val.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let get_str_vec = |key: &str| -> Vec<String> {
        match val.get(key) {
            Some(serde_yaml::Value::Sequence(seq)) => seq
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            Some(serde_yaml::Value::String(s)) => {
                s.split(',').map(|s| s.trim().to_string()).collect()
            }
            _ => vec![],
        }
    };

    let name = get_str("name");
    if name.is_empty() {
        anyhow::bail!("frontmatter missing 'name' in {}", md_path.display());
    }

    let agent = AgentDef {
        name,
        version: "1.0.0".to_string(),
        description: get_str("description"),
        triggers: get_str_vec("triggers"),
        model: get_str("model"),
        color: get_str("color"),
        tools: get_str_vec("tools"),
        skills: get_str_vec("skills"),
        template: get_str("template"),
        category: get_str("category"),
        prompt: None,
        hooks: vec![],
        metadata: AgentMetadata::default(),
        workflows: vec![],
    };

    let stem = md_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("agent");
    let out_path = out_dir.join(format!("{}.yaml", stem));
    save(&out_path, &agent)?;
    Ok(out_path)
}

/// Generate a Claude Code compatible `.md` file from an `AgentDef` and optional prompt text.
///
/// If `prompt` is `Some`, it is used as the body. Otherwise falls back to `agent.prompt`,
/// then a placeholder comment.
pub fn generate_md_with_prompt(agent: &AgentDef, prompt: Option<&str>) -> String {
    let mut fm_fields = vec![
        format!("name: \"{}\"", agent.name),
        format!("description: \"{}\"", agent.description.replace('"', "'")),
    ];
    if !agent.model.is_empty() {
        fm_fields.push(format!("model: {}", agent.model));
    }
    if !agent.color.is_empty() {
        fm_fields.push(format!("color: {}", agent.color));
    }
    if !agent.tools.is_empty() {
        fm_fields.push(format!("tools: [{}]", quote_list(&agent.tools)));
    }
    if !agent.skills.is_empty() {
        fm_fields.push(format!("skills: {}", agent.skills.join(", ")));
    }

    let frontmatter = fm_fields.join("\n");
    let body = prompt
        .or(agent.prompt.as_deref())
        .unwrap_or("<!-- generated from YAML — add prompt here -->");

    format!("---\n{}\n---\n\n{}\n", frontmatter, body)
}

/// Generate a Claude Code compatible `.md` file from an `AgentDef`.
pub fn generate_md(agent: &AgentDef) -> String {
    generate_md_with_prompt(agent, None)
}

/// Load an agent from `agents/cfg/<name>.cfg.yaml`, pair with
/// `agents/prompts/<name>.txt`, and generate the top-level `.md`.
///
/// Returns `(md_content, output_path)`.
///
/// # Errors
///
/// Returns an error when the agent definition or optional prompt cannot be read or parsed.
pub fn generate_from_cfg(agents_dir: &Path, name: &str) -> Result<(String, PathBuf)> {
    let cfg_path = agents_dir.join("cfg").join(format!("{name}.cfg.yaml"));
    let prompt_path = agents_dir
        .join("prompts")
        .join(format!("{name}.prompt.txt"));

    let def = load(&cfg_path)?;
    let out_path = if def.category.is_empty() {
        agents_dir.join(format!("{name}.md"))
    } else {
        agents_dir.join(format!("{}__{name}.md", def.category))
    };
    let prompt = if prompt_path.exists() {
        Some(
            std::fs::read_to_string(&prompt_path)
                .with_context(|| format!("reading prompt {}", prompt_path.display()))?,
        )
    } else {
        None
    };

    let md = generate_md_with_prompt(&def, prompt.as_deref());
    Ok((md, out_path))
}

/// List all agent names available in `agents/cfg/`.
///
/// # Errors
///
/// Returns an error when the configuration directory or one of its entries cannot be read.
pub fn list_cfg_agents(agents_dir: &Path) -> Result<Vec<String>> {
    let cfg_dir = agents_dir.join("cfg");
    if !cfg_dir.exists() {
        return Ok(vec![]);
    }
    let mut names = vec![];
    for entry in std::fs::read_dir(&cfg_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|f| f.strip_suffix(".cfg.yaml"))
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

fn quote_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn generic_agent_module_does_not_implement_opencode_catalog() {
        let source = include_str!("agent.rs");
        for marker in [
            concat!("pub enum ", "OpenCodePermission"),
            concat!("pub type ", "OpenCodePermissions"),
            concat!("pub struct ", "OpenCodeAgentCatalog"),
            concat!("pub struct ", "OpenCodeRouterDef"),
            concat!("pub struct ", "OpenCodeProjectAgentDef"),
            concat!("fn render_opencode_", "router"),
            concat!("fn render_opencode_", "project_agent"),
            concat!("fn validate_opencode_", "catalog"),
            concat!("fn yaml_", "double_quoted"),
        ] {
            assert!(
                !source.contains(marker),
                "generic agent module still implements OpenCode marker: {marker}"
            );
        }
    }

    #[test]
    fn default_version_is_stable() {
        assert_eq!(default_version(), "1.0.0");
    }

    fn sample_agent() -> AgentDef {
        AgentDef {
            name: "test-agent".to_string(),
            version: "1.0.0".to_string(),
            description: "a test agent".to_string(),
            triggers: vec!["trigger1".to_string()],
            model: "inherit".to_string(),
            color: "blue".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string()],
            skills: vec!["brainstorm".to_string()],
            template: "plan".to_string(),
            category: "plan".to_string(),
            prompt: Some("You are a test agent.".to_string()),
            hooks: vec![],
            metadata: AgentMetadata::default(),
            workflows: vec![],
        }
    }

    #[test]
    fn roundtrip_yaml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("agent.yaml");
        let agent = sample_agent();
        save(&path, &agent).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.name, "test-agent");
        assert_eq!(loaded.color, "blue");
        assert_eq!(loaded.tools.len(), 2);
    }

    #[test]
    fn generate_md_produces_frontmatter() {
        let agent = sample_agent();
        let md = generate_md(&agent);
        assert!(md.starts_with("---\n"));
        assert!(md.contains("name: \"test-agent\""));
        assert!(md.contains("You are a test agent."));
    }

    #[test]
    fn migrate_md_to_yaml_roundtrip() {
        let dir = TempDir::new().unwrap();
        let md_content = r#"---
name: "godmode:brainstorm-agent"
description: "Design specialist"
model: inherit
color: blue
tools: ["Read", "Write"]
skills: brainstorm
---

Some prose here.
"#;
        let md_path = dir.path().join("brainstorm-agent.md");
        std::fs::write(&md_path, md_content).unwrap();

        let yaml_path = migrate_md_to_yaml(&md_path, dir.path()).unwrap();
        assert!(yaml_path.exists());
        let loaded = load(&yaml_path).unwrap();
        assert_eq!(loaded.name, "godmode:brainstorm-agent");
        assert_eq!(loaded.color, "blue");
    }

    #[test]
    fn extract_frontmatter_none_when_missing() {
        assert!(extract_frontmatter("no frontmatter here").is_none());
    }
}
