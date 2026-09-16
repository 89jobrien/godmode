//! Conformance tests for repository-local command projections.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

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

fn markdown_files(directory: &Path) -> Result<BTreeMap<String, String>> {
    let entries = std::fs::read_dir(directory)
        .with_context(|| format!("read projection directory {}", directory.display()))?;
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(entry.path())?;
            Ok((name, content))
        })
        .collect()
}

fn ensure_tracked(root: &Path, directory: &Path, files: &BTreeMap<String, String>) -> Result<()> {
    for name in files.keys() {
        let path = directory.join(name).strip_prefix(root)?.to_path_buf();
        let status = Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&path)
            .current_dir(root)
            .status()
            .with_context(|| format!("check tracked command {}", path.display()))?;
        if !status.success() {
            bail!("generated command is not tracked: {}", path.display());
        }
    }
    Ok(())
}

fn check_command_projections() -> Result<()> {
    let root = repo_root();
    let temporary = tempfile::tempdir()?;
    let generated_root = temporary.path();
    let generated_source = generated_root.join("commands/gm");
    copy_tree(&root.join("commands/gm"), &generated_source)?;

    let output = Command::new("nu")
        .arg(generated_source.join("generate.nu"))
        .current_dir(generated_root)
        .output()
        .context("run command projection generator")?;
    if !output.status.success() {
        bail!(
            "generator failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let generated_claude = markdown_files(&generated_root.join("commands"))?;
    let generated_opencode = markdown_files(&generated_root.join(".opencode/commands"))?;
    let tracked_claude = markdown_files(&root.join("commands"))?;
    let tracked_opencode = markdown_files(&root.join(".opencode/commands"))?;

    ensure_tracked(&root, &root.join("commands"), &tracked_claude)?;
    ensure_tracked(&root, &root.join(".opencode/commands"), &tracked_opencode)?;

    if generated_claude != tracked_claude {
        bail!("tracked Claude command projection is stale");
    }
    if generated_opencode != tracked_opencode {
        bail!("tracked OpenCode command projection is stale");
    }
    if generated_claude.keys().ne(generated_opencode.keys()) {
        bail!("Claude and OpenCode command-name sets differ");
    }

    for (name, content) in &generated_claude {
        if !content.contains("\nallowed-tools:\n") || content.contains("\nsubtask:") {
            bail!("Claude command {name} has incorrect frontmatter");
        }
    }
    for (name, content) in &generated_opencode {
        if !content.contains("\nsubtask: false\n") || content.contains("\nallowed-tools:") {
            bail!("OpenCode command {name} has incorrect frontmatter");
        }
    }

    let claude_content = generated_claude.values().cloned().collect::<String>();
    let opencode_content = generated_opencode.values().cloned().collect::<String>();
    if !claude_content.contains("/gm:") || claude_content.contains("/gm-") {
        bail!("Claude command references must use /gm:<name>");
    }
    if !opencode_content.contains("/gm-") || opencode_content.contains("/gm:") {
        bail!("OpenCode command references must use /gm-<name>");
    }

    Ok(())
}

pub struct DualCommandProjection;

impl ConformanceTest for DualCommandProjection {
    fn name(&self) -> &str {
        "dual_command_projection"
    }

    fn crate_name(&self) -> &str {
        "command_projection"
    }

    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }

    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_command_projections() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![Box::new(DualCommandProjection)]
}

#[test]
fn one_invocation_maintains_dual_command_projections() -> Result<()> {
    check_command_projections()
}
