//! Shared helpers for the `godmode-core` integration tests.
//!
//! Each integration test binary is its own crate, so it compiles this module
//! whole and may use only part of it. The `dead_code` allowances below keep an
//! unused helper in one binary from failing `-D warnings` in another.

use std::path::PathBuf;
use std::process::Command;

/// Git's repository-pinning variables, matching `godmode_core`'s
/// `GIT_PINNED_ENV`.
///
/// A `git` hook run from a linked worktree exports an absolute `GIT_DIR` to its
/// children, and this crate is developed from a linked worktree. A scratch
/// repository created or driven while that pin is inherited would read and
/// write the *hook's* repository instead of its own directory, so every child
/// spawned by these tests strips the pins first.
#[allow(dead_code)]
pub const GIT_PINNED_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
];

/// `bin` in `dir` with the repository pins stripped from the child environment.
#[allow(dead_code)]
pub fn scrubbed(bin: impl AsRef<std::path::Path>, dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(bin.as_ref());
    cmd.current_dir(dir);
    for var in GIT_PINNED_ENV {
        cmd.env_remove(var);
    }
    cmd
}

/// Path to the compiled `godmode` binary.
///
/// Walks from `CARGO_MANIFEST_DIR` up to the workspace root, then checks
/// `target/debug/godmode` and `target/release/godmode`.
#[allow(dead_code)]
pub fn godmode_bin() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/godmode-core; workspace root is two levels up.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    // Honour CARGO_TARGET_DIR. Hardcoding `<root>/target` points at a stale
    // binary whenever the target dir is redirected (a worktree, a shared cache)
    // — or finds nothing and fails for a reason that reads like a build step
    // was forgotten.
    let target_root = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => {
            let dir = PathBuf::from(dir);
            if dir.is_absolute() {
                dir
            } else {
                workspace_root.join(dir)
            }
        }
        None => workspace_root.join("target"),
    };

    // Prefer the profile that matches current build (debug first, then release).
    for profile in &["debug", "release"] {
        let candidate = target_root.join(profile).join("godmode");
        if candidate.exists() {
            return candidate;
        }
    }

    panic!(
        "godmode binary not found under {}/{{debug,release}}. \
         Run `cargo build -p godmode-cli` first.",
        target_root.display()
    )
}
