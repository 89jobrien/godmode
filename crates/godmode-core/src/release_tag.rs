//! Annotated git tag creation and push.

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::integrations::subprocess::command_in;

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

    let out = command_in("git", &["tag", "-a", &tag_name, "-m", &message], root)
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

    let out = command_in("git", &["push"], root)
        .output()
        .context("failed to run git push")?;
    if !out.status.success() {
        bail!("git push failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    let out = command_in("git", &["push", "origin", &tag_name], root)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git_harness::{BareRepo, GitRepo, PinnedGitDir};

    /// A fixture with a version file and one commit, wired to `origin`.
    fn fixture_with_origin(version: &str) -> (GitRepo, BareRepo) {
        let origin = BareRepo::init();
        let repo = GitRepo::init();
        repo.commit("initial commit", None);
        repo.remote(origin.path().to_str().expect("bare path is UTF-8"));
        repo.git_ok(&["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
        repo.write(
            ".version-bump.json",
            r#"{"files":[{"path":".claude-plugin/plugin.json","field":"version"}]}"#,
        );
        repo.write(
            ".claude-plugin/plugin.json",
            &format!(r#"{{"version":"{version}"}}"#),
        );
        (repo, origin)
    }

    /// Regression: `git tag` must tag `root`, not the repository an ambient
    /// `GIT_DIR` points at.
    #[test]
    fn tag_is_created_in_the_given_root_under_a_pinned_git_dir() {
        let (fixture, _origin) = fixture_with_origin("0.2.0");

        let decoy = GitRepo::init();
        decoy.commit("decoy commit", None);

        let _pin = PinnedGitDir::pointing_at(&decoy);

        assert_eq!(tag(fixture.path()).expect("create tag"), "v0.2.0");
        assert_eq!(
            fixture.git_ok(&["tag", "--list"]),
            "v0.2.0",
            "the tag belongs in the given root"
        );
        assert!(
            decoy.git_ok(&["tag", "--list"]).is_empty(),
            "the pinned repository must be left untouched"
        );
    }

    /// Regression: `git push` must publish `root`, not the repository an
    /// ambient `GIT_DIR` points at. Both remotes are local bare repositories,
    /// so this never touches the network.
    #[test]
    fn push_publishes_the_given_root_under_a_pinned_git_dir() {
        let (fixture, fixture_origin) = fixture_with_origin("0.3.0");
        fixture.git_ok(&["tag", "v0.3.0"]);

        let (decoy, decoy_origin) = fixture_with_origin("9.9.9");

        let _pin = PinnedGitDir::pointing_at(&decoy);

        push(fixture.path()).expect("push the given root");
        assert_eq!(
            fixture_origin.git_ok(&["tag", "--list"]),
            "v0.3.0",
            "the tag belongs in the given root's origin"
        );
        assert!(
            decoy_origin.git_ok(&["tag", "--list"]).is_empty(),
            "the pinned repository's origin must be left untouched"
        );
    }
}
