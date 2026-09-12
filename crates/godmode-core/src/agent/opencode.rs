//! OpenCode catalog validation, rendering, and installation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Component, Path, PathBuf},
};

/// Permission level rendered for an OpenCode tool.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OpenCodePermission {
    /// Permit the tool without prompting.
    Allow,
    /// Ask the user before invoking the tool.
    Ask,
    /// Deny the tool.
    Deny,
}

impl std::fmt::Display for OpenCodePermission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Allow => "allow",
            Self::Ask => "ask",
            Self::Deny => "deny",
        })
    }
}

/// Project ID to tool-name permission mappings.
pub type OpenCodePermissions = BTreeMap<String, BTreeMap<String, OpenCodePermission>>;

/// Declarative source for one OpenCode router and its project specialists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpenCodeAgentCatalog {
    /// Primary router responsible for delegating workspace requests.
    pub router: OpenCodeRouterDef,
    /// Project-specific subagents available to the router.
    #[serde(default)]
    pub projects: Vec<OpenCodeProjectAgentDef>,
    /// Optional project-specific tool permissions loaded from catalog data.
    #[serde(default)]
    pub permissions: OpenCodePermissions,
}

impl OpenCodeAgentCatalog {
    /// Create a catalog with a router and no project specialists.
    pub fn new(router: OpenCodeRouterDef) -> Self {
        Self {
            router,
            projects: Vec::new(),
            permissions: BTreeMap::new(),
        }
    }

    /// Set project-specific tool permissions.
    pub fn with_permissions(mut self, permissions: OpenCodePermissions) -> Self {
        self.permissions = permissions;
        self
    }

    /// Set the project specialists exposed by this catalog.
    pub fn with_projects(mut self, projects: Vec<OpenCodeProjectAgentDef>) -> Self {
        self.projects = projects;
        self
    }
}

/// OpenCode primary-agent metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpenCodeRouterDef {
    /// File-safe name of the primary router agent.
    pub name: String,
    /// Human-readable summary rendered into agent frontmatter.
    pub description: String,
}

impl OpenCodeRouterDef {
    /// Create primary-router metadata.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// OpenCode subagent metadata for one workspace project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpenCodeProjectAgentDef {
    /// File-safe name of the project specialist.
    pub name: String,
    /// Human-readable summary of the specialist's scope.
    pub description: String,
    /// Registry identifier for the governed project tools.
    pub project: String,
    /// Project path relative to the workspace root.
    pub repo_path: String,
    /// Whether the specialist is visible in OpenCode's agent picker.
    #[serde(default)]
    pub visible: bool,
}

impl OpenCodeProjectAgentDef {
    /// Create hidden project-specialist metadata.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        project: impl Into<String>,
        repo_path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            project: project.into(),
            repo_path: repo_path.into(),
            visible: false,
        }
    }

    /// Set whether the specialist is visible in OpenCode's agent picker.
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Load the project-agent catalog from a path or the embedded default.
///
/// # Errors
///
/// Returns an error when a custom catalog cannot be read, parsed, or validated.
///
/// # Examples
///
/// ```
/// use godmode_core::agent::load_opencode_catalog;
///
/// # fn main() -> anyhow::Result<()> {
/// let catalog = load_opencode_catalog(None)?;
/// assert!(!catalog.projects.is_empty());
/// # Ok(())
/// # }
/// ```
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
        content: render_opencode_project_agent(project, catalog.permissions.get(&project.project)),
    }));
    rendered
}

/// Render the router and project specialists as legacy `(name, content)` pairs.
///
/// # Examples
///
/// ```
/// use godmode_core::agent::{
///     OpenCodeAgentCatalog, OpenCodeRouterDef, render_opencode_agents,
/// };
///
/// let catalog = OpenCodeAgentCatalog::new(OpenCodeRouterDef::new("workspace", "Router"));
/// let rendered = render_opencode_agents(&catalog);
/// assert_eq!(rendered[0].0, "workspace.md");
/// ```
pub fn render_opencode_agents(catalog: &OpenCodeAgentCatalog) -> Vec<(String, String)> {
    render_opencode_agent_files(catalog)
        .into_iter()
        .map(|agent| (agent.file_name, agent.content))
        .collect()
}

/// Install OpenCode agents or preview paths from a legacy dry-run flag.
///
/// # Errors
///
/// Returns an error when catalog validation or the atomic installation transaction fails.
///
/// # Examples
///
/// ```
/// use godmode_core::agent::{
///     OpenCodeAgentCatalog, OpenCodeRouterDef, install_opencode_agents,
/// };
///
/// # fn main() -> anyhow::Result<()> {
/// let output = tempfile::tempdir()?;
/// let catalog = OpenCodeAgentCatalog::new(OpenCodeRouterDef::new("workspace", "Router"));
/// let paths = install_opencode_agents(&catalog, output.path(), false)?;
/// assert!(paths[0].exists());
/// # Ok(())
/// # }
/// ```
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
    install_rendered_with_fs(&rendered, output_dir, &StdInstallFs)?;
    Ok(paths)
}

trait InstallFsPort {
    fn exists(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn create_dir(&self, path: &Path) -> std::io::Result<()>;
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()>;
}

struct StdInstallFs;
impl InstallFsPort for StdInstallFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)
    }
    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir(path)
    }
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
        std::fs::write(path, content)
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)
    }
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(path)
    }
}

#[cfg(test)]
fn install_opencode_agents_with_fs(
    catalog: &OpenCodeAgentCatalog,
    output_dir: &Path,
    fs: &dyn InstallFsPort,
) -> Result<Vec<PathBuf>> {
    validate_opencode_catalog(catalog)?;
    let rendered = render_opencode_agent_files(catalog);
    let paths = rendered
        .iter()
        .map(|agent| output_dir.join(&agent.file_name))
        .collect();
    install_rendered_with_fs(&rendered, output_dir, fs)?;
    Ok(paths)
}

fn install_rendered_with_fs(
    rendered: &[RenderedAgent],
    output_dir: &Path,
    fs: &dyn InstallFsPort,
) -> Result<()> {
    if fs.is_file(output_dir) {
        anyhow::bail!(
            "OpenCode agent output is not a directory: {}",
            output_dir.display()
        );
    }
    let parent = output_dir
        .parent()
        .context("OpenCode output directory has no parent")?;
    fs.create_dir_all(parent)
        .with_context(|| format!("creating OpenCode agent parent {}", parent.display()))?;
    let name = output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .context("invalid OpenCode output directory name")?;
    let staging = parent.join(format!(".{name}.{}.staging", std::process::id()));
    let backup = parent.join(format!(".{name}.{}.backup", std::process::id()));
    if fs.exists(&staging) || fs.exists(&backup) {
        anyhow::bail!(
            "OpenCode install transaction already exists for {}",
            output_dir.display()
        );
    }
    fs.create_dir(&staging)
        .with_context(|| format!("creating OpenCode staging dir {}", staging.display()))?;
    let staged = (|| -> Result<()> {
        for agent in rendered {
            let path = staging.join(&agent.file_name);
            fs.write(&path, &agent.content)
                .with_context(|| format!("writing staged OpenCode agent {}", path.display()))?;
        }
        if fs.exists(output_dir) {
            fs.rename(output_dir, &backup)?;
        }
        if let Err(error) = fs.rename(&staging, output_dir) {
            if fs.exists(&backup) {
                let _ = fs.rename(&backup, output_dir);
            }
            return Err(error).with_context(|| {
                format!("installing OpenCode agents into {}", output_dir.display())
            });
        }
        if fs.exists(&backup) {
            fs.remove_dir_all(&backup)?;
        }
        Ok(())
    })();
    if staged.is_err() && fs.exists(&staging) {
        let _ = fs.remove_dir_all(&staging);
    }
    staged
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

fn render_opencode_project_agent(
    project: &OpenCodeProjectAgentDef,
    permissions: Option<&BTreeMap<String, OpenCodePermission>>,
) -> String {
    let hidden = if project.visible { "false" } else { "true" };
    let mut rendered = format!(
        r#"---
description: "{}"
mode: subagent
hidden: {hidden}
color: info
permission:
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
    for (tool, permission) in permissions.into_iter().flatten() {
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
    for (project, permissions) in &catalog.permissions {
        validate_project_id(project)?;
        if !catalog
            .projects
            .iter()
            .any(|entry| entry.project == *project)
        {
            anyhow::bail!("OpenCode permissions reference unknown project: {project}");
        }
        for tool in permissions.keys() {
            if !tool.starts_with("personal_")
                || !tool.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                anyhow::bail!("invalid OpenCode permission tool name: {tool}");
            }
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
    use super::super::extract_frontmatter;
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    #[derive(Clone, Copy)]
    enum Failure {
        Staging,
        SecondFile,
        Swap,
    }

    struct FailingFs {
        failure: Failure,
        writes: AtomicUsize,
    }
    impl FailingFs {
        fn new(failure: Failure) -> Self {
            Self {
                failure,
                writes: AtomicUsize::new(0),
            }
        }
    }
    impl InstallFsPort for FailingFs {
        fn exists(&self, path: &Path) -> bool {
            path.exists()
        }
        fn is_file(&self, path: &Path) -> bool {
            path.is_file()
        }
        fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
            std::fs::create_dir_all(path)
        }
        fn create_dir(&self, path: &Path) -> std::io::Result<()> {
            if matches!(self.failure, Failure::Staging) {
                return Err(std::io::Error::other("staging failed"));
            }
            std::fs::create_dir(path)
        }
        fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
            if matches!(self.failure, Failure::SecondFile)
                && self.writes.fetch_add(1, Ordering::SeqCst) == 1
            {
                return Err(std::io::Error::other("second file failed"));
            }
            std::fs::write(path, content)
        }
        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            if matches!(self.failure, Failure::Swap) && from.to_string_lossy().ends_with(".staging")
            {
                return Err(std::io::Error::other("swap failed"));
            }
            std::fs::rename(from, to)
        }
        fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
            std::fs::remove_dir_all(path)
        }
    }

    fn catalog() -> OpenCodeAgentCatalog {
        OpenCodeAgentCatalog {
            router: OpenCodeRouterDef {
                name: "workspace".into(),
                description: "router".into(),
            },
            projects: vec![OpenCodeProjectAgentDef {
                name: "workspace-x".into(),
                description: "x".into(),
                project: "x".into(),
                repo_path: "x".into(),
                visible: false,
            }],
            permissions: Default::default(),
        }
    }

    #[test]
    fn empty_catalog_documents_and_router_only_catalogs_are_distinguished() {
        let dir = tempfile::tempdir().unwrap();
        let empty = dir.path().join("empty.yaml");
        std::fs::write(&empty, "").unwrap();
        assert!(load_opencode_catalog(Some(&empty)).is_err());
        let router_only = dir.path().join("router.yaml");
        std::fs::write(
            &router_only,
            "router: { name: workspace, description: router }\nprojects: []\n",
        )
        .unwrap();
        assert!(
            load_opencode_catalog(Some(&router_only))
                .unwrap()
                .projects
                .is_empty()
        );
    }

    #[test]
    fn install_reports_staging_and_per_file_failures_without_partial_output() {
        for failure in [Failure::Staging, Failure::SecondFile] {
            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("agents");
            assert!(
                install_opencode_agents_with_fs(&catalog(), &output, &FailingFs::new(failure))
                    .is_err()
            );
            assert!(!output.exists());
            assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn install_swap_failure_restores_previous_generation() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("agents");
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("old.md"), "old").unwrap();
        assert!(
            install_opencode_agents_with_fs(&catalog(), &output, &FailingFs::new(Failure::Swap))
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(output.join("old.md")).unwrap(),
            "old"
        );
    }

    #[test]
    fn install_rejects_preexisting_transaction_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("agents");
        let staging = dir
            .path()
            .join(format!(".agents.{}.staging", std::process::id()));
        std::fs::create_dir(&staging).unwrap();
        let error = install_opencode_agents_with_mode(
            &catalog(),
            &output,
            crate::write_mode::WriteMode::Write,
        )
        .unwrap_err();
        assert!(error.to_string().contains("transaction already exists"));
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
        assert!(!router.contains("permission:\n  \"*\":"));
        assert!(router.contains("task:\n    \"*\": deny"));

        let minibox = rendered
            .iter()
            .find(|(name, _)| name == "workspace-minibox.md")
            .map(|(_, content)| content)
            .unwrap();
        assert!(minibox.contains("hidden: false"));
        assert!(!minibox.contains("permission:\n  \"*\":"));
        assert!(minibox.contains("personal_project_describe"));
        assert!(minibox.contains("  read: allow"));
        assert!(minibox.contains("personal_project_run: allow"));
        assert!(!minibox.contains("personal_project_*"));
        assert!(minibox.contains("personal_minibox_list_containers: allow"));
        assert!(minibox.contains("personal_minibox_stop_container: ask"));

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
    fn catalog_rejects_empty_and_invalid_project_data_variants() {
        let dir = TempDir::new().unwrap();
        for (name, body) in [
            ("empty.yaml", ""),
            (
                "empty-router.yaml",
                "router: { name: , description: router }\nprojects: []\n",
            ),
            (
                "empty-project.yaml",
                "router: { name: workspace, description: router }\nprojects: [{ name: workspace-x, project: , repo_path: x }]\n",
            ),
            (
                "absolute-path.yaml",
                "router: { name: workspace, description: router }\nprojects: [{ name: workspace-x, project: x, repo_path: /tmp/x }]\n",
            ),
            (
                "parent-path.yaml",
                "router: { name: workspace, description: router }\nprojects: [{ name: workspace-x, project: x, repo_path: ../x }]\n",
            ),
        ] {
            let path = dir.path().join(name);
            std::fs::write(&path, body).unwrap();
            assert!(load_opencode_catalog(Some(&path)).is_err(), "{name}");
        }
    }

    #[test]
    fn catalog_permissions_are_loaded_from_yaml_and_validated() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("catalog.yaml");
        std::fs::write(&path, "router: { name: workspace, description: router }\nprojects:\n  - name: workspace-x\n    project: x\n    repo_path: x\n    description: x specialist\npermissions:\n  x: { personal_x_read: allow, personal_x_write: ask }\n").unwrap();
        let catalog = load_opencode_catalog(Some(&path)).unwrap();
        let rendered = render_opencode_agents(&catalog);
        assert!(rendered[1].1.contains("personal_x_read: allow"));
        assert!(rendered[1].1.contains("personal_x_write: ask"));

        std::fs::write(&path, "router: { name: workspace, description: router }\nprojects:\n  - { name: workspace-x, description: x, project: x, repo_path: x }\npermissions: { x: { personal_x: execute } }\n").unwrap();
        assert!(load_opencode_catalog(Some(&path)).is_err());
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

    #[test]
    fn project_tools_render_with_declared_permission() {
        let catalog = load_opencode_catalog(None).unwrap();
        let rendered = render_opencode_agents(&catalog);
        let devloop = rendered
            .iter()
            .find(|(name, _)| name == "workspace-devloop.md")
            .map(|(_, content)| content)
            .unwrap();
        assert!(devloop.contains("personal_devloop_get_timeline: allow"));
    }

    #[test]
    fn mutation_tools_render_as_ask() {
        let catalog = load_opencode_catalog(None).unwrap();
        let rendered = render_opencode_agents(&catalog);
        for (agent, tool) in [
            ("workspace-minibox.md", "personal_minibox_stop_container"),
            ("workspace-crux.md", "personal_crux_task_update"),
            ("workspace-taskit.md", "personal_taskit_protocol_drift"),
            ("workspace-mcpipe.md", "personal_mcpipe_scan_catalog"),
            (
                "workspace-mcpipe.md",
                "personal_mcpipe_generate_doob_openapi",
            ),
        ] {
            let content = rendered
                .iter()
                .find(|(name, _)| name == agent)
                .map(|(_, content)| content)
                .unwrap();
            assert!(content.contains(&format!("{tool}: ask")));
        }
    }

    #[test]
    fn rendered_agent_boundary_exposes_named_content() {
        let catalog = load_opencode_catalog(None).unwrap();
        let files = render_opencode_agent_files(&catalog);
        assert_eq!(files[0].file_name, "workspace.md");
        assert!(files[0].content.contains("mode: primary"));
    }

    #[test]
    fn opencode_catalog_read_error_names_source_path() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("catalog.yaml");
        std::fs::create_dir_all(&path).unwrap();
        let error = load_opencode_catalog(Some(&path)).unwrap_err().to_string();
        assert!(error.contains("catalog.yaml"), "{error}");
    }

    #[test]
    fn opencode_install_failure_leaves_no_partial_output() {
        let catalog = load_opencode_catalog(None).unwrap();
        let dir = TempDir::new().unwrap();
        let output = dir.path().join("agents");
        std::fs::write(&output, "not a directory").unwrap();
        assert!(
            install_opencode_agents_with_mode(
                &catalog,
                &output,
                crate::write_mode::WriteMode::Write
            )
            .is_err()
        );
        assert_eq!(std::fs::read_to_string(output).unwrap(), "not a directory");
    }

    #[test]
    fn opencode_catalog_parse_error_names_source_path() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("broken.yaml");
        std::fs::write(&path, "router: [").unwrap();
        let error = load_opencode_catalog(Some(&path)).unwrap_err().to_string();
        assert!(error.contains("broken.yaml"), "{error}");
    }
}
