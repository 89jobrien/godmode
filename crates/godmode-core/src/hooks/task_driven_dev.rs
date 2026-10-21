//! task-driven-development — PreToolUse/Bash hook.
//! Warns if running cargo build/run without test changes in working tree.

use std::path::Path;

use crate::integrations::subprocess::command_in;

/// Run the task-driven-development hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str) -> String {
    let is_build = command.contains("cargo build") || command.contains("cargo run");
    if !is_build {
        return String::new();
    }

    // Exclude test/check/clippy variants
    if command.contains("cargo test")
        || command.contains("cargo nextest")
        || command.contains("cargo check")
        || command.contains("cargo clippy")
    {
        return String::new();
    }

    // Check for test-related changes in working tree
    let output = command_in("git", &["diff", "HEAD", "--name-only"], root).output();

    let has_test_changes = match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .any(|f| f.contains("test") || f.ends_with("_test.rs"))
        }
        _ => return String::new(),
    };

    if has_test_changes {
        String::new()
    } else {
        "[godmode:tdd] Building without tests — write a failing test first (see tdd-tasks.yaml)"
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git_harness::{GitRepo, PinnedGitDir};

    /// Regression: the hook must inspect `root`'s working tree, not the one
    /// belonging to the repository an ambient `GIT_DIR` points at.
    #[test]
    fn build_warning_uses_the_given_roots_working_tree_under_a_pinned_git_dir() {
        let fixture = GitRepo::init();
        fixture.write("src/lib.rs", "pub fn fixture() {}\n");
        fixture.git_ok(&["add", "--all"]);
        fixture.commit("fixture commit", None);

        // The decoy has an uncommitted change to a test file, so a hook that
        // reads it believes tests were already written and stays silent.
        let decoy = GitRepo::init();
        decoy.write("src/decoy_test.rs", "// committed\n");
        decoy.git_ok(&["add", "--all"]);
        decoy.commit("decoy commit", None);
        decoy.write("src/decoy_test.rs", "// uncommitted test change\n");

        let _pin = PinnedGitDir::pointing_at(&decoy);

        assert!(
            !run(fixture.path(), "cargo build").is_empty(),
            "a build with no test changes in the given root must warn"
        );
    }
}
