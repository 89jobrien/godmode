//! Conformance tests for the git-hook installer (`hooks/install.nu`).
//!
//! Regression cover for a real incident: the installer used to bake the
//! absolute path of the source checkout into the `.git/hooks` wrappers, so
//! deleting the worktree the installer ran from (issue-32) left every
//! installed hook dangling. Installed hooks must be self-contained.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

/// Git pins `GIT_DIR` & co. when invoking hooks, so child processes in these
/// tests must be scrubbed or they would operate on the invoking repository
/// instead of the fixture repos.
fn isolate_from_invoking_repo(mut command: Command) -> Command {
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_PREFIX",
    ] {
        command.env_remove(var);
    }
    command
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate must be nested under the repository root")
        .to_path_buf()
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else {
            std::fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

/// A throwaway target repository plus a private copy of the godmode hooks
/// tree acting as the plugin root the installer is run from.
struct InstallFixture {
    _temporary: tempfile::TempDir,
    plugin_root: PathBuf,
    repo: PathBuf,
}

fn fixture() -> Result<InstallFixture> {
    let temporary = tempfile::tempdir()?;
    let plugin_root = temporary.path().join("godmode-plugin");
    copy_tree(&repo_root().join("hooks"), &plugin_root.join("hooks"))?;

    let repo = temporary.path().join("target-repo");
    std::fs::create_dir_all(&repo)?;
    let output = isolate_from_invoking_repo(Command::new("git"))
        .arg("init")
        .current_dir(&repo)
        .output()
        .context("run git init for the fixture repository")?;
    if !output.status.success() {
        bail!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(InstallFixture {
        _temporary: temporary,
        plugin_root,
        repo,
    })
}

fn run_installer(fixture: &InstallFixture) -> Result<std::process::Output> {
    isolate_from_invoking_repo(Command::new("nu"))
        .arg(fixture.plugin_root.join("hooks/install.nu"))
        .current_dir(&fixture.repo)
        .output()
        .context("run hooks/install.nu")
}

/// The installed wrappers must resolve their script relative to their own
/// location — never through the absolute path of the checkout that ran the
/// installer.
fn check_wrappers_are_self_contained() -> Result<()> {
    let fixture = fixture()?;
    let output = run_installer(&fixture)?;
    if !output.status.success() {
        bail!(
            "install.nu failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let hooks_dir = fixture.repo.join(".git/hooks");
    let plugin = fixture.plugin_root.display().to_string();
    for hook in ["pre-commit", "pre-push", "post-commit"] {
        let wrapper_path = hooks_dir.join(hook);
        let wrapper = std::fs::read_to_string(&wrapper_path)
            .with_context(|| format!("read installed wrapper {}", wrapper_path.display()))?;
        if wrapper.contains(&plugin) {
            bail!("{hook} wrapper embeds the installer checkout path {plugin}");
        }
        if !wrapper.contains("$(dirname \"$0\")/godmode/") {
            bail!("{hook} wrapper does not resolve its script relative to its own location");
        }
        if !hooks_dir
            .join("godmode")
            .join(format!("{hook}.nu"))
            .is_file()
        {
            bail!("{hook} script was not copied into the git hooks payload directory");
        }
    }

    if !hooks_dir.join("godmode/lib/godmode-hook-lib.nu").is_file() {
        bail!("hook library was not copied into the git hooks payload directory");
    }
    Ok(())
}

/// Installing into a bare repository would write wrappers into the git dir
/// itself (for this repo's bare primary checkout, that is the tracked
/// `hooks/` directory). The installer must refuse.
fn check_installer_refuses_bare_repositories() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let plugin_root = temporary.path().join("godmode-plugin");
    copy_tree(&repo_root().join("hooks"), &plugin_root.join("hooks"))?;

    let bare = temporary.path().join("bare.git");
    let output = isolate_from_invoking_repo(Command::new("git"))
        .args(["init", "--bare"])
        .arg(&bare)
        .output()
        .context("run git init --bare for the fixture repository")?;
    if !output.status.success() {
        bail!(
            "git init --bare failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = isolate_from_invoking_repo(Command::new("nu"))
        .arg(plugin_root.join("hooks/install.nu"))
        .current_dir(&bare)
        .output()
        .context("run hooks/install.nu in a bare repository")?;
    if output.status.success() {
        bail!("install.nu must exit non-zero in a bare repository");
    }
    if bare.join("hooks/godmode").exists() {
        bail!("install.nu must not write a payload directory into a bare repository");
    }
    Ok(())
}

/// Deleting the checkout the installer ran from must not break the installed
/// hooks — the exact failure that left this repository's hooks dangling when
/// the issue-32 worktree was removed.
fn check_installed_hooks_survive_source_removal() -> Result<()> {
    let fixture = fixture()?;
    let output = run_installer(&fixture)?;
    if !output.status.success() {
        bail!(
            "install.nu failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    std::fs::remove_dir_all(&fixture.plugin_root)?;

    let hooks_dir = fixture.repo.join(".git/hooks");
    let status = isolate_from_invoking_repo(Command::new("sh"))
        .arg(hooks_dir.join("pre-push"))
        .current_dir(&fixture.repo)
        .status()
        .context("execute installed pre-push hook")?;
    if !status.success() {
        bail!("installed pre-push hook failed after the source checkout was removed");
    }
    Ok(())
}

pub struct GitHookInstallConformance;

impl ConformanceTest for GitHookInstallConformance {
    fn name(&self) -> &str {
        "git_hook_install_conformance"
    }

    fn crate_name(&self) -> &str {
        "hook_install"
    }

    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }

    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_wrappers_are_self_contained() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

pub struct GitHookSurvivalConformance;

impl ConformanceTest for GitHookSurvivalConformance {
    fn name(&self) -> &str {
        "git_hook_source_removal_survival"
    }

    fn crate_name(&self) -> &str {
        "hook_install"
    }

    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }

    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_installed_hooks_survive_source_removal() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

pub struct GitHookBareRepoConformance;

impl ConformanceTest for GitHookBareRepoConformance {
    fn name(&self) -> &str {
        "git_hook_installer_refuses_bare_repositories"
    }

    fn crate_name(&self) -> &str {
        "hook_install"
    }

    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }

    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_installer_refuses_bare_repositories() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

/// Returns the git-hook installer conformance tests.
pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(GitHookInstallConformance),
        Box::new(GitHookSurvivalConformance),
        Box::new(GitHookBareRepoConformance),
    ]
}

#[test]
fn installed_wrappers_resolve_scripts_relative_to_themselves() -> Result<()> {
    check_wrappers_are_self_contained()
}

#[test]
fn installed_hooks_survive_source_checkout_removal() -> Result<()> {
    check_installed_hooks_survive_source_removal()
}

#[test]
fn installer_refuses_bare_repositories() -> Result<()> {
    check_installer_refuses_bare_repositories()
}
