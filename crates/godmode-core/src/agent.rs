use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

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
    pub template: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub hooks: Vec<AgentHook>,
    #[serde(default)]
    pub metadata: AgentMetadata,
    #[serde(default)]
    pub workflows: Vec<WorkflowRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRef {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slash_command: Option<String>,
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

/// Declarative source for one OpenCode router and its project specialists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeAgentCatalog {
    pub router: OpenCodeRouterDef,
    #[serde(default)]
    pub projects: Vec<OpenCodeProjectAgentDef>,
}

/// OpenCode primary-agent metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeRouterDef {
    pub name: String,
    pub description: String,
}

/// OpenCode subagent metadata for one workspace project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeProjectAgentDef {
    pub name: String,
    pub description: String,
    pub project: String,
    pub repo_path: String,
    #[serde(default)]
    pub visible: bool,
}

/// Load the project-agent catalog from a path or the embedded default.
pub fn load_opencode_catalog(path: Option<&Path>) -> Result<OpenCodeAgentCatalog> {
    let raw = if let Some(path) = path {
        std::fs::read_to_string(path)
            .with_context(|| format!("reading OpenCode agent catalog {}", path.display()))?
    } else {
        include_str!("../../../agents/opencode-projects.yaml").to_string()
    };
    let catalog = serde_yaml::from_str(&raw).context("parsing OpenCode agent catalog")?;
    validate_opencode_catalog(&catalog)?;
    Ok(catalog)
}

/// Render the router and project specialists as OpenCode Markdown agents.
pub fn render_opencode_agents(catalog: &OpenCodeAgentCatalog) -> Vec<(String, String)> {
    let mut rendered = Vec::with_capacity(catalog.projects.len() + 1);
    rendered.push((
        format!("{}.md", catalog.router.name),
        render_opencode_router(catalog),
    ));
    rendered.extend(catalog.projects.iter().map(|project| {
        (
            format!("{}.md", project.name),
            render_opencode_project_agent(project),
        )
    }));
    rendered
}

/// Install rendered OpenCode agents into `output_dir`, or preview paths in dry-run mode.
pub fn install_opencode_agents(
    catalog: &OpenCodeAgentCatalog,
    output_dir: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>> {
    validate_opencode_catalog(catalog)?;
    let rendered = render_opencode_agents(catalog);
    if !dry_run {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("creating OpenCode agent dir {}", output_dir.display()))?;
    }
    let mut paths = Vec::with_capacity(rendered.len());
    for (file_name, content) in rendered {
        let path = output_dir.join(file_name);
        if !dry_run {
            std::fs::write(&path, content)
                .with_context(|| format!("writing OpenCode agent {}", path.display()))?;
        }
        paths.push(path);
    }
    Ok(paths)
}

fn render_opencode_router(catalog: &OpenCodeAgentCatalog) -> String {
    let routes = catalog
        .projects
        .iter()
        .map(|project| format!("- `{}`: {}", project.name, project.description))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"---
description: "{}"
mode: primary
color: primary
permission:
  "*": deny
  edit: deny
  bash: deny
  task:
    "*": deny
    "workspace-*": allow
---

You are the workspace router for Joe's development projects. Delegate project-specific work to the matching `workspace-*` specialist. Do not implement, edit, or run shell commands yourself. When a request spans projects, dispatch independent specialists in parallel and synthesize their results.

Available routes:
{routes}
<!-- generated by godmode agent install-opencode -->
"#,
        yaml_double_quoted(&catalog.router.description),
    )
}

fn render_opencode_project_agent(project: &OpenCodeProjectAgentDef) -> String {
    let hidden = if project.visible { "false" } else { "true" };
    format!(
        r#"---
description: "{}"
mode: subagent
hidden: {hidden}
color: info
permission:
  "*": deny
  read: allow
  glob: allow
  grep: allow
  edit: ask
  bash: ask
  task: deny
  external_directory: ask
  personal_project_list: allow
  personal_project_describe: allow
  personal_project_run: allow
  personal_git_status: allow
  personal_git_diff: allow
  personal_env_health: allow
---
"#,
        yaml_double_quoted(&project.description),
    ) + &format!(
        concat!(
            "\nYou are the `{project}` specialist for Joe's workspace. Prefer the installed ",
            "project CLI as a governed tool before inspecting source. Start by calling ",
            "`personal_project_describe` with project `{project}`, then use ",
            "`personal_project_run` for a matching allowlisted read-only action. Never replace ",
            "a registry action with unrestricted shell execution.\n\n",
            "Only modify source when the user explicitly asks to develop, fix, refactor, or ",
            "document this project. For source work, use `$HOME/dev/{repo_path}`, read its ",
            "`AGENTS.md` and project guidance first, preserve unrelated changes, and follow its ",
            "native quality commands. Do not commit, push, release, deploy, or run destructive ",
            "operations unless explicitly requested.\n",
            "<!-- generated by godmode agent install-opencode -->\n"
        ),
        project = project.project,
        repo_path = project.repo_path,
    )
}

fn yaml_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn validate_opencode_catalog(catalog: &OpenCodeAgentCatalog) -> Result<()> {
    let mut names = HashSet::new();
    validate_opencode_name(&catalog.router.name)?;
    names.insert(catalog.router.name.as_str());
    for project in &catalog.projects {
        validate_opencode_name(&project.name)?;
        validate_project_id(&project.project)?;
        if !names.insert(project.name.as_str()) {
            anyhow::bail!("duplicate OpenCode agent name: {}", project.name);
        }
        let repo = Path::new(&project.repo_path);
        if repo.is_absolute()
            || repo
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            anyhow::bail!("invalid OpenCode agent repo path: {}", project.repo_path);
        }
    }
    Ok(())
}

fn validate_opencode_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-');
    if valid {
        Ok(())
    } else {
        anyhow::bail!("invalid OpenCode agent name: {name}")
    }
}

fn validate_project_id(project: &str) -> Result<()> {
    let valid = !project.is_empty()
        && project
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        anyhow::bail!("invalid OpenCode project identifier: {project}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    #[test]
    fn embedded_opencode_catalog_contains_all_project_agents() {
        let catalog = load_opencode_catalog(None).unwrap();
        assert_eq!(catalog.router.name, "workspace");
        assert_eq!(catalog.projects.len(), 23);
        assert!(
            catalog
                .projects
                .iter()
                .any(|project| project.project == "minibox" && project.visible)
        );
    }

    #[test]
    fn render_opencode_agents_includes_router_and_specialists() {
        let catalog = load_opencode_catalog(None).unwrap();
        let rendered = render_opencode_agents(&catalog);
        assert_eq!(rendered.len(), 24);

        let router = rendered
            .iter()
            .find(|(name, _)| name == "workspace.md")
            .map(|(_, content)| content)
            .unwrap();
        assert!(router.contains("mode: primary"));
        assert!(router.contains("\"workspace-*\": allow"));
        assert!(router.contains("  edit: deny"));
        assert!(router.contains("permission:\n  \"*\": deny"));
        assert!(router.contains("task:\n    \"*\": deny"));

        let minibox = rendered
            .iter()
            .find(|(name, _)| name == "workspace-minibox.md")
            .map(|(_, content)| content)
            .unwrap();
        assert!(minibox.contains("hidden: false"));
        assert!(minibox.contains("personal_project_describe"));
        assert!(minibox.contains("  read: allow"));
        assert!(minibox.contains("personal_project_run: allow"));
        assert!(!minibox.contains("personal_project_*"));

        let maestro = rendered
            .iter()
            .find(|(name, _)| name == "workspace-maestro.md")
            .map(|(_, content)| content)
            .unwrap();
        assert!(maestro.contains("hidden: true"));
    }

    #[test]
    fn dry_run_install_does_not_write_files() {
        let catalog = load_opencode_catalog(None).unwrap();
        let dir = TempDir::new().unwrap();
        let output = dir.path().join("agents");
        let paths = install_opencode_agents(&catalog, &output, true).unwrap();
        assert_eq!(paths.len(), 24);
        assert!(!output.exists());
    }

    #[test]
    fn install_writes_rendered_agents() {
        let catalog = load_opencode_catalog(None).unwrap();
        let dir = TempDir::new().unwrap();
        let output = dir.path().join("agents");
        let paths = install_opencode_agents(&catalog, &output, false).unwrap();
        assert_eq!(paths.len(), 24);
        assert!(output.join("workspace.md").exists());
        assert!(output.join("workspace-doob.md").exists());
    }

    #[test]
    fn catalog_rejects_unsafe_and_duplicate_agent_names() {
        let dir = TempDir::new().unwrap();
        let unsafe_catalog = dir.path().join("unsafe.yaml");
        std::fs::write(
            &unsafe_catalog,
            "router: { name: ../escape, description: bad }\nprojects: []\n",
        )
        .unwrap();
        assert!(load_opencode_catalog(Some(&unsafe_catalog)).is_err());

        let duplicate_catalog = dir.path().join("duplicate.yaml");
        std::fs::write(
            &duplicate_catalog,
            concat!(
                "router: { name: workspace, description: router }\n",
                "projects:\n",
                "  - { name: workspace, description: duplicate, project: crux, ",
                "repo_path: crux }\n"
            ),
        )
        .unwrap();
        assert!(load_opencode_catalog(Some(&duplicate_catalog)).is_err());
    }

    #[test]
    fn rendered_frontmatter_parses_as_yaml() {
        let catalog = load_opencode_catalog(None).unwrap();
        for (_, content) in render_opencode_agents(&catalog) {
            let frontmatter = extract_frontmatter(&content).expect("frontmatter");
            let value: serde_yaml::Value = serde_yaml::from_str(frontmatter).unwrap();
            assert!(value.get("permission").is_some());
        }
    }
}
