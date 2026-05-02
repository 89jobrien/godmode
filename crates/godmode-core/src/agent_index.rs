use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntry {
    pub name: String,
    pub description: String,
    pub color: String,
    pub skills: Vec<String>,
    pub tools: Vec<String>,
    pub path: PathBuf,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Walk `<root>/agents/*.md`, parse YAML frontmatter, return sorted entries.
pub fn list_agents(root: &Path) -> Result<Vec<AgentEntry>> {
    let agents_dir = root.join("agents");
    if !agents_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<AgentEntry> = vec![];

    for entry in fs::read_dir(&agents_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        // Skip INDEX.md itself
        if path.file_name().and_then(|n| n.to_str()) == Some("INDEX.md") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        if let Some(agent) = parse_agent(&content, path) {
            entries.push(agent);
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Filter a list of agents by keyword (name or description, case-insensitive).
pub fn filter_agents(agents: Vec<AgentEntry>, keyword: &str) -> Vec<AgentEntry> {
    let kw = keyword.to_lowercase();
    agents
        .into_iter()
        .filter(|a| {
            a.name.to_lowercase().contains(&kw) || a.description.to_lowercase().contains(&kw)
        })
        .collect()
}

/// Write `<root>/agents/INDEX.md` from the given agent list.
pub fn generate_agent_index(root: &Path, agents: &[AgentEntry]) -> Result<()> {
    let index_path = root.join("agents/INDEX.md");

    let mut lines = vec![
        "# Agent Index".to_string(),
        String::new(),
        "| Name | Description | Skills | Color |".to_string(),
        "| ---- | ----------- | ------ | ----- |".to_string(),
    ];

    for a in agents {
        let desc = first_line(&a.description);
        let skills = a.skills.join(", ");
        lines.push(format!(
            "| {} | {} | {} | {} |",
            a.name, desc, skills, a.color
        ));
    }

    lines.push(String::new());
    fs::write(&index_path, lines.join("\n"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse YAML frontmatter from a markdown file and return an AgentEntry.
fn parse_agent(content: &str, path: PathBuf) -> Option<AgentEntry> {
    let (fm, _body) = split_frontmatter(content)?;

    // Use a simple serde_yaml parse into a generic map
    let map: serde_yaml::Value = serde_yaml::from_str(fm).ok()?;
    let obj = map.as_mapping()?;

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();

    if name.is_empty() {
        return None;
    }

    let description = obj
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let color = obj
        .get("color")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let skills = parse_skills_field(obj.get("skills"));
    let tools = parse_tools_field(obj.get("tools"));

    Some(AgentEntry {
        name,
        description,
        color,
        skills,
        tools,
        path,
    })
}

/// Split content into (frontmatter, body) strings.  Returns `None` if no `---` delimiters found.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let content = content.trim_start_matches('\n');
    if !content.starts_with("---") {
        return None;
    }
    let after_first = content.get(3..)?.trim_start_matches('\n');
    // Find the closing ---
    let close = after_first.find("\n---")?;
    let fm = &after_first[..close];
    let body_start = close + 4; // skip "\n---"
    let body = after_first.get(body_start..).unwrap_or("");
    Some((fm, body))
}

/// Parse `skills:` field — may be a comma-separated scalar or a YAML sequence.
fn parse_skills_field(val: Option<&serde_yaml::Value>) -> Vec<String> {
    match val {
        None => vec![],
        Some(serde_yaml::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect(),
        Some(serde_yaml::Value::String(s)) => s
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect(),
        _ => vec![],
    }
}

/// Parse `tools:` field — may be a YAML sequence or a scalar.
fn parse_tools_field(val: Option<&serde_yaml::Value>) -> Vec<String> {
    match val {
        None => vec![],
        Some(serde_yaml::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect(),
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        _ => vec![],
    }
}

/// Return first non-empty line from a possibly multi-line string.
fn first_line(s: &str) -> String {
    s.lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_agent(dir: &TempDir, filename: &str, content: &str) {
        let agents = dir.path().join("agents");
        fs::create_dir_all(&agents).unwrap();
        fs::write(agents.join(filename), content).unwrap();
    }

    #[test]
    fn list_agents_returns_sorted_entries() {
        let tmp = tempfile::tempdir().unwrap();
        make_agent(
            &tmp,
            "z-agent.md",
            "---\nname: godmode:z-agent\ndescription: Z agent description here\ncolor: red\n\
             skills: foo\ntools: [Read]\n---\n\nBody.\n",
        );
        make_agent(
            &tmp,
            "a-agent.md",
            "---\nname: godmode:a-agent\ndescription: A agent description here\ncolor: blue\n\
             skills: bar\ntools: [Write]\n---\n\nBody.\n",
        );
        let agents = list_agents(tmp.path()).unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "godmode:a-agent");
        assert_eq!(agents[1].name, "godmode:z-agent");
    }

    #[test]
    fn filter_agents_is_case_insensitive() {
        let tmp = tempfile::tempdir().unwrap();
        make_agent(
            &tmp,
            "task-agent.md",
            "---\nname: godmode:task-agent\ndescription: Manages TASKS for you\ncolor: green\n\
             skills: task\ntools: [Bash]\n---\n",
        );
        make_agent(
            &tmp,
            "other-agent.md",
            "---\nname: godmode:other-agent\ndescription: Completely unrelated\ncolor: blue\n\
             skills: other\ntools: [Read]\n---\n",
        );
        let agents = list_agents(tmp.path()).unwrap();
        let filtered = filter_agents(agents, "task");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "godmode:task-agent");
    }

    #[test]
    fn generate_agent_index_writes_table() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("agents")).unwrap();
        let agents = vec![AgentEntry {
            name: "godmode:test-agent".to_string(),
            description: "A test agent\nwith multiple lines".to_string(),
            color: "purple".to_string(),
            skills: vec!["test-skill".to_string()],
            tools: vec!["Read".to_string()],
            path: tmp.path().join("agents/test-agent.md"),
        }];
        generate_agent_index(tmp.path(), &agents).unwrap();
        let content = fs::read_to_string(tmp.path().join("agents/INDEX.md")).unwrap();
        assert!(content.contains("| godmode:test-agent |"));
        assert!(content.contains("A test agent"));
        assert!(content.contains("test-skill"));
        assert!(content.contains("purple"));
    }

    #[test]
    fn parse_skills_comma_separated() {
        let tmp = tempfile::tempdir().unwrap();
        make_agent(
            &tmp,
            "multi.md",
            "---\nname: godmode:multi\ndescription: multi skill agent desc\ncolor: teal\n\
             skills: foo, bar, baz\ntools: [Read]\n---\n",
        );
        let agents = list_agents(tmp.path()).unwrap();
        assert_eq!(agents[0].skills, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn skips_index_md_itself() {
        let tmp = tempfile::tempdir().unwrap();
        make_agent(
            &tmp,
            "real-agent.md",
            "---\nname: godmode:real\ndescription: A real agent with description\ncolor: blue\n\
             skills: real\ntools: [Read]\n---\n",
        );
        fs::write(
            tmp.path().join("agents/INDEX.md"),
            "# Agent Index\n| Name | Description | Skills | Color |\n",
        )
        .unwrap();
        let agents = list_agents(tmp.path()).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "godmode:real");
    }
}
