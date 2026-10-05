//! Conformance checks for generated hook manifests.

use anyhow::{Context, Result, bail};
use godmode_core::hooks::registry::{
    HookClient, REGISTRY, UNSUPPORTED_OPENCODE_EVENTS, diagnose_coverage, generate_manifest,
    generate_opencode_plugin, opencode_hooks, opencode_unmapped,
};
use std::collections::BTreeSet;

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn check_hook_manifest() -> Result<()> {
    let root = repo_root();
    let tracked: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("hooks/hooks.json"))?)?;
    if tracked != generate_manifest(HookClient::Claude) {
        bail!("hooks/hooks.json has drifted from the hook registry");
    }
    let issues = diagnose_coverage(&root, HookClient::Claude, REGISTRY, &tracked);
    if !issues.is_empty() {
        bail!("hook coverage issues: {}", serde_json::to_string(&issues)?);
    }
    Ok(())
}

/// Verifies `.opencode/plugins/godmode.ts` matches the registry and that every
/// OpenCode hook script referenced actually exists.
fn check_opencode_plugin() -> Result<()> {
    let root = repo_root();
    let tracked_path = root.join(".opencode/plugins/godmode.ts");
    if !tracked_path.is_file() {
        bail!(
            ".opencode/plugins/godmode.ts is missing; run `godmode hook generate --client opencode`"
        );
    }
    let tracked = std::fs::read_to_string(&tracked_path)?;
    let expected = format!("{}\n", generate_opencode_plugin());
    if tracked != expected {
        bail!(
            ".opencode/plugins/godmode.ts has drifted from the hook registry; \
             run `godmode hook generate --client opencode`"
        );
    }

    // Every script an OpenCode hook invokes must exist, or the hook silently
    // fails at runtime. Claude-side coverage is checked separately.
    for event in ["tool.execute.before", "tool.execute.after"] {
        for entry in opencode_hooks(event) {
            if let Some(script) = entry.command.split("$CLAUDE_PLUGIN_ROOT/").nth(1)
                && !root.join(script).is_file()
            {
                bail!(
                    "OpenCode hook '{}' references missing script: {script}",
                    entry.id
                );
            }
        }
    }

    // Hooks with no OpenCode counterpart must be reported, not dropped.
    for (id, event, _) in opencode_unmapped() {
        if !UNSUPPORTED_OPENCODE_EVENTS.contains(&event) {
            bail!("hook {id} reported unmapped but sits on supported event {event}");
        }
        if !tracked.contains(id) {
            bail!("unmapped hook {id} is not reported in the generated plugin");
        }
    }
    Ok(())
}

pub struct HookManifestConformance;
impl ConformanceTest for HookManifestConformance {
    fn name(&self) -> &str {
        "hook_manifest_conformance"
    }
    fn crate_name(&self) -> &str {
        "hook_registry"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_hook_manifest() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

pub struct OpenCodePluginConformance;
impl ConformanceTest for OpenCodePluginConformance {
    fn name(&self) -> &str {
        "opencode_plugin_conformance"
    }
    fn crate_name(&self) -> &str {
        "hook_registry"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_opencode_plugin() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

/// Verifies every `.opencode/agents/*.md` carries the frontmatter OpenCode
/// requires, that the permission map is deny-by-default, and that the set
/// matches `agents/cfg/`.
fn check_opencode_agents() -> Result<()> {
    let root = repo_root();
    let agents_dir = root.join(".opencode").join("agents");
    if !agents_dir.is_dir() {
        bail!(".opencode/agents/ is missing; run `godmode agent generate-opencode --all`");
    }

    let cfg_names: BTreeSet<String> = godmode_core::agent::list_cfg_agents(&root.join("agents"))?
        .into_iter()
        .map(|n| n.strip_suffix("-agent").unwrap_or(&n).to_string())
        .collect();

    let mut seen = BTreeSet::new();
    for entry in std::fs::read_dir(&agents_dir)?.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("non-UTF-8 agent filename: {}", path.display()))?
            .to_string();

        let content = std::fs::read_to_string(&path)?;
        let rest = content
            .strip_prefix("---\n")
            .ok_or_else(|| anyhow::anyhow!("{} has no frontmatter", path.display()))?;
        let (frontmatter, _) = rest
            .split_once("\n---\n")
            .ok_or_else(|| anyhow::anyhow!("{} has unterminated frontmatter", path.display()))?;
        let parsed: serde_yaml::Value = serde_yaml::from_str(frontmatter)
            .with_context(|| format!("parsing {}", path.display()))?;

        match parsed.get("mode").and_then(serde_yaml::Value::as_str) {
            Some("subagent") => {}
            other => bail!("{} has mode {other:?}, expected subagent", path.display()),
        }
        if parsed
            .get("description")
            .and_then(serde_yaml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            bail!("{} is missing a description", path.display());
        }

        // Deny-by-default: the catch-all must be the final rule, because
        // OpenCode applies the last matching permission.
        let permissions = parsed
            .get("permission")
            .and_then(serde_yaml::Value::as_mapping)
            .ok_or_else(|| anyhow::anyhow!("{} has no permission map", path.display()))?;
        let keys: Vec<&str> = permissions.keys().filter_map(|k| k.as_str()).collect();
        let deny = keys
            .iter()
            .position(|k| *k == "*")
            .ok_or_else(|| anyhow::anyhow!("{} has no catch-all deny", path.display()))?;
        if deny + 1 != keys.len() {
            bail!(
                "{} places the catch-all deny before an explicit rule",
                path.display()
            );
        }

        seen.insert(stem);
    }

    let missing: Vec<&String> = cfg_names.difference(&seen).collect();
    if !missing.is_empty() {
        bail!("agents in cfg/ have no OpenCode projection: {missing:?}");
    }
    Ok(())
}

pub struct OpenCodeAgentConformance;
impl ConformanceTest for OpenCodeAgentConformance {
    fn name(&self) -> &str {
        "opencode_agent_conformance"
    }
    fn crate_name(&self) -> &str {
        "hook_registry"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        if let Err(error) = check_opencode_agents() {
            ctx.fail(&error.to_string());
        }
        ctx.result()
    }
}

/// Returns the generated hook-manifest conformance tests.
pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(HookManifestConformance),
        Box::new(OpenCodePluginConformance),
        Box::new(OpenCodeAgentConformance),
    ]
}

#[test]
fn generated_hook_manifest_is_conformant() -> Result<()> {
    check_hook_manifest()
}

#[test]
fn generated_opencode_plugin_is_conformant() -> Result<()> {
    check_opencode_plugin()
}

#[test]
fn generated_opencode_agents_are_conformant() -> Result<()> {
    check_opencode_agents()
}
