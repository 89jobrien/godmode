//! OpenCode catalog validation, rendering, and installation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

/// Declarative source for one OpenCode router and its project specialists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeAgentCatalog {
    /// Primary router responsible for delegating workspace requests.
    pub router: OpenCodeRouterDef,
    /// Project-specific subagents available to the router.
    #[serde(default)]
    pub projects: Vec<OpenCodeProjectAgentDef>,
}

/// OpenCode primary-agent metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeRouterDef {
    /// File-safe name of the primary router agent.
    pub name: String,
    /// Human-readable summary rendered into agent frontmatter.
    #[serde(default)]
    pub description: String,
}

/// OpenCode subagent metadata for one workspace project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeProjectAgentDef {
    /// File-safe name of the project specialist.
    pub name: String,
    /// Human-readable summary of the specialist's scope.
    #[serde(default)]
    pub description: String,
    /// Registry identifier for the governed project tools.
    #[serde(default)]
    pub project: String,
    /// Project path relative to the workspace root.
    #[serde(default)]
    pub repo_path: String,
    /// Whether the specialist is visible in OpenCode's agent picker.
    #[serde(default)]
    pub visible: bool,
}

/// Load the project-agent catalog from a path or the embedded default.
pub fn load_opencode_catalog(path: Option<&Path>) -> Result<OpenCodeAgentCatalog> {
    let raw = if let Some(path) = path {
        std::fs::read_to_string(path)
            .with_context(|| format!("reading OpenCode agent catalog {}", path.display()))?
    } else {
        include_str!("../../../../agents/opencode-projects.yaml").to_string()
    };
    let source = path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "embedded catalog".to_string());
    let catalog = serde_yaml::from_str(&raw)
        .with_context(|| format!("parsing OpenCode agent catalog {source}"))?;
    validate_opencode_catalog(&catalog)?;
    Ok(catalog)
}

/// Rendered OpenCode agent document at the filesystem boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RenderedAgent {
    /// Destination file name.
    pub file_name: String,
    /// Complete Markdown document.
    pub content: String,
}

/// Render typed router and specialist documents.
pub fn render_opencode_agent_files(catalog: &OpenCodeAgentCatalog) -> Vec<RenderedAgent> {
    let mut rendered = Vec::with_capacity(catalog.projects.len() + 1);
    rendered.push(RenderedAgent {
        file_name: format!("{}.md", catalog.router.name),
        content: render_opencode_router(catalog),
    });
    rendered.extend(catalog.projects.iter().map(|project| RenderedAgent {
        file_name: format!("{}.md", project.name),
        content: render_opencode_project_agent(project),
    }));
    rendered
}

/// Render the router and project specialists as legacy `(name, content)` pairs.
pub fn render_opencode_agents(catalog: &OpenCodeAgentCatalog) -> Vec<(String, String)> {
    render_opencode_agent_files(catalog)
        .into_iter()
        .map(|agent| (agent.file_name, agent.content))
        .collect()
}

/// Install OpenCode agents or preview paths from a legacy dry-run flag.
pub fn install_opencode_agents(
    catalog: &OpenCodeAgentCatalog,
    output_dir: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>> {
    install_opencode_agents_with_mode(catalog, output_dir, dry_run.into())
}

/// Atomically install the complete rendered agent set.
///
/// # Errors
///
/// Returns an error for an invalid catalog or any staging/swap failure.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// use godmode_core::{agent, write_mode::WriteMode};
/// let catalog = agent::load_opencode_catalog(None)?;
/// let _ = agent::install_opencode_agents_with_mode(
///     &catalog, std::path::Path::new("agents"), WriteMode::Preview,
/// )?;
/// # Ok(()) }
/// ```
pub fn install_opencode_agents_with_mode(
    catalog: &OpenCodeAgentCatalog,
    output_dir: &Path,
    mode: crate::write_mode::WriteMode,
) -> Result<Vec<PathBuf>> {
    validate_opencode_catalog(catalog)?;
    let rendered = render_opencode_agent_files(catalog);
    let paths = rendered
        .iter()
        .map(|agent| output_dir.join(&agent.file_name))
        .collect();
    if !mode.writes() {
        return Ok(paths);
    }
    if output_dir.is_file() {
        anyhow::bail!(
            "OpenCode agent output is not a directory: {}",
            output_dir.display()
        );
    }
    let parent = output_dir
        .parent()
        .context("OpenCode output directory has no parent")?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("creating OpenCode agent parent {}", parent.display()))?;
    let name = output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .context("invalid OpenCode output directory name")?;
    let staging = parent.join(format!(".{name}.{}.staging", std::process::id()));
    let backup = parent.join(format!(".{name}.{}.backup", std::process::id()));
    if staging.exists() || backup.exists() {
        anyhow::bail!(
            "OpenCode install transaction already exists for {}",
            output_dir.display()
        );
    }
    std::fs::create_dir(&staging)
        .with_context(|| format!("creating OpenCode staging dir {}", staging.display()))?;
    let staged = (|| -> Result<()> {
        for agent in &rendered {
            let path = staging.join(&agent.file_name);
            std::fs::write(&path, &agent.content)
                .with_context(|| format!("writing staged OpenCode agent {}", path.display()))?;
        }
        if output_dir.exists() {
            std::fs::rename(output_dir, &backup)?;
        }
        if let Err(error) = std::fs::rename(&staging, output_dir) {
            if backup.exists() {
                let _ = std::fs::rename(&backup, output_dir);
            }
            return Err(error).with_context(|| {
                format!("installing OpenCode agents into {}", output_dir.display())
            });
        }
        if backup.exists() {
            std::fs::remove_dir_all(&backup)?;
        }
        Ok(())
    })();
    if staged.is_err() && staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    staged?;
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
    let mut rendered = format!(
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
"#,
        yaml_double_quoted(&project.description),
    );
    for (tool, permission) in project_tool_permissions(&project.project) {
        rendered.push_str(&format!("  {tool}: {permission}\n"));
    }
    rendered.push_str("---\n");
    rendered
        + &format!(
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

fn project_tool_permissions(project: &str) -> &'static [(&'static str, &'static str)] {
    match project {
        "devloop" => &[
            ("personal_devloop_list_branches", "allow"),
            ("personal_devloop_get_branch", "allow"),
            ("personal_devloop_get_timeline", "allow"),
        ],
        "minibox" => &[
            ("personal_minibox_list_containers", "allow"),
            ("personal_minibox_get_logs", "allow"),
            ("personal_minibox_get_manifest", "allow"),
            ("personal_minibox_stop_container", "ask"),
        ],
        "crux" => &[
            ("personal_crux_list_pipelines", "allow"),
            ("personal_crux_validate_pipeline", "allow"),
            ("personal_crux_task_list", "allow"),
            ("personal_crux_task_update", "ask"),
        ],
        "taskit" => &[
            ("personal_taskit_health_inspect", "allow"),
            ("personal_taskit_health_drift", "allow"),
            ("personal_taskit_protocol_drift", "ask"),
        ],
        "mcpipe" => &[
            ("personal_mcpipe_scan_catalog", "ask"),
            ("personal_mcpipe_list_personal_tools", "allow"),
            ("personal_mcpipe_generate_doob_openapi", "ask"),
        ],
        "agentlint" => &[
            ("personal_agentlint_check", "allow"),
            ("personal_agentlint_infer_docs_schema", "allow"),
        ],
        _ => &[],
    }
}
