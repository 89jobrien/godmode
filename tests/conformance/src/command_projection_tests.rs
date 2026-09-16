//! Conformance tests for repository-local command projections.

use std::collections::BTreeMap;
#[cfg(test)]
use std::os::unix::fs::PermissionsExt;
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

fn copy_generator_sources(root: &Path, destination: &Path) -> Result<()> {
    copy_tree(&root.join("commands/gm"), &destination.join("commands/gm"))?;
    copy_tree(&root.join("pipelines"), &destination.join("pipelines"))
}

fn markdown_files(directory: &Path) -> Result<BTreeMap<String, String>> {
    let entries = std::fs::read_dir(directory)
        .with_context(|| format!("read projection directory {}", directory.display()))?;
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with("gm-")
                && entry.path().extension().is_some_and(|ext| ext == "md")
        })
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
        let output = Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&path)
            .current_dir(root)
            .output()
            .with_context(|| format!("check tracked command {}", path.display()))?;
        if !output.status.success() {
            bail!("generated command is not tracked: {}", path.display());
        }
    }
    Ok(())
}

fn parse_frontmatter(content: &str) -> Result<serde_yaml::Value> {
    let rest = content
        .strip_prefix("---\n")
        .context("command must start with YAML frontmatter")?;
    let (yaml, _) = rest
        .split_once("\n---\n")
        .context("command frontmatter must have a closing delimiter")?;
    serde_yaml::from_str(yaml).context("parse command YAML frontmatter")
}

fn run_generator(root: &Path) -> Result<std::process::Output> {
    Command::new("nu")
        .arg(root.join("commands/gm/generate.nu"))
        .current_dir(root)
        .output()
        .context("run command projection generator")
}

fn check_command_projections() -> Result<()> {
    let root = repo_root();
    let temporary = tempfile::tempdir()?;
    let generated_root = temporary.path();
    copy_generator_sources(&root, generated_root)?;

    let output = run_generator(generated_root)?;
    if !output.status.success() {
        bail!(
            "generator failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    if generated_root.join(".command-projection-stage").exists()
        || generated_root.join(".command-projection-backup").exists()
    {
        bail!("generator left projection scratch directories after success");
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
        let frontmatter = parse_frontmatter(content)?;
        let mapping = frontmatter
            .as_mapping()
            .with_context(|| format!("Claude command {name} frontmatter must be a mapping"))?;
        if mapping.len() != 3
            || mapping
                .get("name")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || mapping
                .get("allowed-tools")
                .and_then(serde_yaml::Value::as_sequence)
                .is_none()
            || mapping
                .get("max-turns")
                .and_then(serde_yaml::Value::as_i64)
                .is_none()
        {
            bail!("Claude command {name} has incorrect frontmatter");
        }
    }
    for (name, content) in &generated_opencode {
        let frontmatter = parse_frontmatter(content)?;
        let mapping = frontmatter
            .as_mapping()
            .with_context(|| format!("OpenCode command {name} frontmatter must be a mapping"))?;
        let description = mapping
            .get("description")
            .and_then(serde_yaml::Value::as_str)
            .with_context(|| format!("OpenCode command {name} needs a description"))?;
        let stem = name.trim_start_matches("gm-").trim_end_matches(".md");
        let canonical: serde_yaml::Value = serde_yaml::from_str(&std::fs::read_to_string(
            root.join("commands/gm").join(format!("{stem}.yaml")),
        )?)?;
        let expected_description = canonical
            .get("prompt")
            .and_then(serde_yaml::Value::as_str)
            .and_then(|prompt| prompt.lines().find(|line| !line.trim().is_empty()))
            .map(str::trim)
            .context("canonical prompt needs a description line")?;
        if mapping.len() != 2
            || mapping.get("subtask").and_then(serde_yaml::Value::as_bool) != Some(false)
            || description != expected_description
        {
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

    let claude_doc_sync = generated_claude
        .get("gm-doc-sync.md")
        .context("gm-doc-sync.md must be generated")?;
    let claude_self_heal = generated_claude
        .get("gm-self-heal.md")
        .context("gm-self-heal.md must be generated")?;
    let claude_moa = generated_claude
        .get("gm-moa-review.md")
        .context("gm-moa-review.md must be generated")?;
    if !claude_doc_sync.contains("skills/*/SKILL.md")
        || !claude_self_heal.contains("$ARGUMENTS")
        || !claude_moa.contains("`git diff $ARGUMENTS...HEAD`")
    {
        bail!("Claude projection changed literal prompt content");
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
fn pipeline_sequence_drives_generated_workflow_commands() -> Result<()> {
    let root = repo_root();
    let temporary = tempfile::tempdir()?;
    let generated_root = temporary.path();
    copy_generator_sources(&root, generated_root)?;

    for (pipeline_name, sentinel) in [
        ("feature", "feature-semantic-drift-sentinel"),
        ("release", "release-semantic-drift-sentinel"),
    ] {
        let pipeline_path = generated_root.join(format!("pipelines/{pipeline_name}.yaml"));
        let mut pipeline: serde_yaml::Value =
            serde_yaml::from_str(&std::fs::read_to_string(&pipeline_path)?)?;
        pipeline["steps"]
            .as_sequence_mut()
            .with_context(|| format!("{pipeline_name} pipeline steps must be a sequence"))?
            .insert(1, serde_yaml::from_str(&format!("skill: {sentinel}"))?);
        std::fs::write(&pipeline_path, serde_yaml::to_string(&pipeline)?)?;
    }

    let output = run_generator(generated_root)?;
    assert!(
        output.status.success(),
        "generator failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (command, before, sentinel, after) in [
        (
            "feature",
            "brainstorm",
            "feature-semantic-drift-sentinel",
            "context-map",
        ),
        (
            "release",
            "health-score",
            "release-semantic-drift-sentinel",
            "dep-audit",
        ),
    ] {
        let projection =
            std::fs::read_to_string(generated_root.join(format!("commands/gm-{command}.md")))?;
        let before = projection
            .find(&format!("godmode:{before}"))
            .with_context(|| format!("{command} preceding step"))?;
        let sentinel = projection
            .find(&format!("godmode:{sentinel}"))
            .with_context(|| format!("{command} pipeline-injected sentinel step"))?;
        let after = projection
            .find(&format!("godmode:{after}"))
            .with_context(|| format!("{command} following step"))?;
        assert!(before < sentinel && sentinel < after);
    }
    Ok(())
}

#[test]
fn one_invocation_maintains_dual_command_projections() -> Result<()> {
    check_command_projections()
}

#[test]
fn discovery_ignores_unrelated_markdown_commands() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    std::fs::write(temporary.path().join("gm-example.md"), "generated")?;
    std::fs::write(temporary.path().join("custom.md"), "custom")?;
    let files = markdown_files(temporary.path())?;
    assert_eq!(files.keys().collect::<Vec<_>>(), ["gm-example.md"]);
    Ok(())
}

#[test]
fn failed_second_target_setup_leaves_both_projections_unchanged() -> Result<()> {
    let root = repo_root();
    let temporary = tempfile::tempdir()?;
    let generated_root = temporary.path();
    copy_generator_sources(&root, generated_root)?;
    let sentinel = generated_root.join("commands/gm-sentinel.md");
    std::fs::write(&sentinel, "unchanged")?;
    std::fs::write(generated_root.join(".opencode"), "blocks output directory")?;

    let output = run_generator(generated_root)?;
    assert!(!output.status.success());
    assert_eq!(std::fs::read_to_string(sentinel)?, "unchanged");
    assert!(!generated_root.join(".command-projection-stage").exists());
    assert!(!generated_root.join(".command-projection-backup").exists());
    Ok(())
}

#[test]
fn second_target_swap_failure_rolls_back_both_and_cleans_scratch() -> Result<()> {
    let root = repo_root();
    let temporary = tempfile::tempdir()?;
    let generated_root = temporary.path();
    copy_generator_sources(&root, generated_root)?;
    let claude_sentinel = generated_root.join("commands/gm-sentinel.md");
    std::fs::write(&claude_sentinel, "claude original")?;

    let opencode_parent = generated_root.join(".opencode");
    let opencode_commands = opencode_parent.join("commands");
    std::fs::create_dir_all(&opencode_commands)?;
    let opencode_sentinel = opencode_commands.join("gm-sentinel.md");
    std::fs::write(&opencode_sentinel, "opencode original")?;
    std::fs::set_permissions(&opencode_parent, std::fs::Permissions::from_mode(0o555))?;

    let output = run_generator(generated_root)?;
    std::fs::set_permissions(&opencode_parent, std::fs::Permissions::from_mode(0o755))?;

    assert!(!output.status.success());
    assert_eq!(std::fs::read_to_string(claude_sentinel)?, "claude original");
    assert_eq!(
        std::fs::read_to_string(opencode_sentinel)?,
        "opencode original"
    );
    assert!(!generated_root.join(".command-projection-stage").exists());
    assert!(!generated_root.join(".command-projection-backup").exists());
    Ok(())
}

#[test]
fn github_actions_runs_projection_conformance() -> Result<()> {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/conformance.yml"))?;
    assert!(workflow.contains("cargo test -p godmode-conformance"));
    Ok(())
}

#[test]
fn github_actions_audits_linux_and_macos_dependency_graphs() -> Result<()> {
    let root = repo_root();
    let deny = std::fs::read_to_string(root.join("deny.toml"))?;
    let workflow = std::fs::read_to_string(root.join(".github/workflows/conformance.yml"))?;

    assert!(deny.contains("x86_64-unknown-linux-gnu"));
    assert!(deny.contains("aarch64-apple-darwin"));
    assert!(workflow.contains("cargo deny check"));
    Ok(())
}

#[test]
fn github_actions_release_artifacts_follow_full_gates() -> Result<()> {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/release.yml"))?;

    assert!(workflow.contains("tags:"));
    assert!(workflow.contains("cargo xtask ci"));
    assert!(workflow.contains("cargo deny check"));
    assert!(workflow.contains("needs: gates"));
    assert!(workflow.contains("cargo xtask dist"));
    assert!(workflow.contains("SHA256SUMS"));
    assert!(workflow.contains("actions/upload-artifact"));
    assert!(workflow.contains("softprops/action-gh-release"));
    Ok(())
}

#[test]
fn github_actions_runs_every_fuzz_target_with_bounds_and_failure_artifacts() -> Result<()> {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/fuzz.yml"))?;

    assert!(workflow.contains("schedule:"));
    assert!(workflow.contains("workflow_dispatch:"));
    for target in [
        "fuzz_plan_parse",
        "fuzz_template_substitute",
        "fuzz_yaml_roundtrip",
        "fuzz_config_toml",
        "fuzz_pipeline_parse",
        "fuzz_pipeline_state",
    ] {
        assert!(workflow.contains(target), "missing fuzz target {target}");
    }
    assert!(workflow.contains("-max_total_time="));
    assert!(workflow.contains("if: failure()"));
    assert!(workflow.contains("fuzz/artifacts/"));
    assert!(workflow.contains("fuzz-logs/"));
    Ok(())
}

#[test]
fn generated_projections_are_excluded_from_markdown_reformatting() -> Result<()> {
    let ignored = std::fs::read_to_string(repo_root().join(".prettierignore"))?;
    assert!(ignored.lines().any(|line| line == "commands/gm-*.md"));
    assert!(
        ignored
            .lines()
            .any(|line| line == ".opencode/commands/gm-*.md")
    );
    Ok(())
}
