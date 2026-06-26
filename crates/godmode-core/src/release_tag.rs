//! Annotated git tag creation and push.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Read the current version from `plugin.json` (first file in `.version-bump.json`).
pub(crate) fn current_version_inner(root: &Path) -> Result<String> {
    use crate::release::release_version::{load_config, read_version_field};
    let cfg = load_config(root)?;
    let target = cfg
        .files
        .first()
        .context("no files in .version-bump.json")?;
    read_version_field(root, target)
}

/// Create an annotated git tag for the current version.
pub fn tag(root: &Path) -> Result<String> {
    let version = current_version_inner(root)?;
    let tag_name = format!("v{version}");
    let message = format!("release {tag_name}");

    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "tag",
            "-a",
            &tag_name,
            "-m",
            &message,
        ])
        .output()
        .context("failed to run git tag")?;

    if !out.status.success() {
        bail!("git tag failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    Ok(tag_name)
}

/// Push the current branch and version tag to origin.
pub fn push(root: &Path) -> Result<()> {
    let version = current_version_inner(root)?;
    let tag_name = format!("v{version}");

    let out = Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."), "push"])
        .output()
        .context("failed to run git push")?;
    if !out.status.success() {
        bail!("git push failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    let out = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "push",
            "origin",
            &tag_name,
        ])
        .output()
        .context("failed to run git push origin <tag>")?;
    if !out.status.success() {
        bail!(
            "git push origin {tag_name} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(())
}
