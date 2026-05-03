use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentHook {
    pub event: String,
    pub matcher: String,
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub hooks: Vec<AgentHook>,
    #[serde(default)]
    pub metadata: AgentMetadata,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Parse an agent YAML file.
pub fn load(path: &Path) -> Result<AgentDef> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading agent file {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing agent YAML {}", path.display()))
}

/// Write an agent YAML file.
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
        prompt: None,
        hooks: vec![],
        metadata: AgentMetadata::default(),
    };

    let stem = md_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("agent");
    let out_path = out_dir.join(format!("{}.yaml", stem));
    save(&out_path, &agent)?;
    Ok(out_path)
}

/// Generate a Claude Code compatible `.md` file from an `AgentDef`.
pub fn generate_md(agent: &AgentDef) -> String {
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
    let body = agent
        .prompt
        .as_deref()
        .unwrap_or("<!-- generated from YAML — add prompt here -->");

    format!("---\n{}\n---\n\n{}\n", frontmatter, body)
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
            prompt: Some("You are a test agent.".to_string()),
            hooks: vec![],
            metadata: AgentMetadata::default(),
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
