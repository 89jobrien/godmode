//! Shared helpers for the `godmode-cli` integration tests.
//!
//! Each integration test binary is its own crate, so it compiles this module
//! whole and may use only part of it. The `dead_code` allowances below keep an
//! unused helper in one binary from failing `-D warnings` in another.

use std::path::{Path, PathBuf};
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
pub fn scrubbed(bin: impl AsRef<Path>, dir: &Path) -> Command {
    let mut cmd = Command::new(bin.as_ref());
    cmd.current_dir(dir);
    for var in GIT_PINNED_ENV {
        cmd.env_remove(var);
    }
    cmd
}

/// Path to the compiled `godmode` binary.
///
/// `cargo nextest`/`cargo test` set `CARGO_BIN_EXE_godmode`; the fallback walks
/// to the workspace `target` for the case where it is not exported.
#[allow(dead_code)]
pub fn godmode_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_godmode") {
        return PathBuf::from(path);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("target/debug/godmode")
}

/// Run the `godmode` binary in `dir` with `args`, pins stripped.
#[allow(dead_code)]
pub fn run_godmode(dir: &Path, args: &[&str]) -> std::process::Output {
    scrubbed(godmode_bin(), dir)
        .args(args)
        .output()
        .expect("run godmode")
}
