//! Conformance checks for generated hook manifests.

use anyhow::{Result, bail};
use godmode_core::hooks::registry::{HookClient, REGISTRY, diagnose_coverage, generate_manifest};

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

pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![Box::new(HookManifestConformance)]
}

#[test]
fn generated_hook_manifest_is_conformant() -> Result<()> {
    check_hook_manifest()
}
