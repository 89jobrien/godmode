//! Workspace maintenance commands used by local development and CI.
//!
//! Run `cargo xtask help` to list the available quality gates and utilities.

#![deny(missing_docs)]

use std::process::{Command, ExitCode};

use anyhow::{Context, Result, bail};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str());

    match cmd {
        Some("pre-commit") => pre_commit(),
        Some("ci") => ci(),
        Some("dist") => dist(),
        Some("install") => install(),
        Some(other) => bail!("unknown xtask: {other}"),
        None => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: cargo xtask <COMMAND>\n\n\
         Commands:\n  \
           pre-commit   fmt-check + clippy + conformance\n  \
           ci           full gate (fmt + clippy -D warnings + nextest + conformance)\n  \
           dist         release build of godmode-cli\n  \
           install      build release and copy to ~/.cargo/bin/"
    );
}

// ── Pre-commit gate ────────────────────────────────────────────────

fn pre_commit() -> Result<()> {
    header("fmt --check");
    cargo(&["fmt", "--all", "--check"])?;

    header("clippy");
    cargo(&["clippy", "--workspace", "--", "-D", "warnings"])?;

    header("conformance");
    cargo(&[
        "run",
        "-p",
        "godmode-conformance",
        "--bin",
        "run-conformance",
        "--",
        "--verbose",
    ])?;

    eprintln!("\nAll pre-commit checks passed.");
    Ok(())
}

// ── Full CI gate ───────────────────────────────────────────────────

fn ci() -> Result<()> {
    header("fmt --check");
    cargo(&["fmt", "--all", "--check"])?;

    header("clippy --all-targets --all-features");
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])?;

    header("nextest");
    cargo(&["nextest", "run", "--workspace"])?;

    header("conformance");
    cargo(&[
        "run",
        "-p",
        "godmode-conformance",
        "--bin",
        "run-conformance",
        "--",
        "--verbose",
    ])?;

    header("Nushell plugin conformance");
    command("nu", &["tests/conformance/plugin-structure.nu"])?;

    header("skill index --check");
    godmode(&["skill", "index", "--check"])?;

    header("agent index --check");
    godmode(&["agent", "index", "--check"])?;

    header("command generate --target claude --check");
    godmode(&["command", "generate", "--target", "claude", "--check"])?;

    header("release validate");
    godmode(&["release", "validate"])?;

    header("cargo deny check");
    cargo(&["deny", "check"])?;

    eprintln!("\nAll CI checks passed.");
    Ok(())
}

// ── Dist build ─────────────────────────────────────────────────────

fn dist() -> Result<()> {
    header("release build");
    cargo(&["build", "--release", "-p", "godmode-cli"])?;

    let binary = target_dir()?.join("release/godmode");
    eprintln!("Binary: {}", binary.display());
    Ok(())
}

// ── Install ────────────────────────────────────────────────────────

fn install() -> Result<()> {
    dist()?;

    let src = target_dir()?.join("release/godmode");
    let dest_dir = home_dir()?.join(".cargo/bin");
    let dest = dest_dir.join("godmode");

    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("failed to create {}", dest_dir.display()))?;

    std::fs::copy(&src, &dest)
        .with_context(|| format!("failed to copy {} -> {}", src.display(), dest.display()))?;

    eprintln!("Installed: {}", dest.display());
    Ok(())
}

fn target_dir() -> Result<std::path::PathBuf> {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            if path.is_absolute() {
                Ok(path)
            } else {
                Ok(project_root()?.join(path))
            }
        }
        None => Ok(project_root()?.join("target")),
    }
}

fn home_dir() -> Result<std::path::PathBuf> {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .context("HOME not set")
}

// ── Helpers ────────────────────────────────────────────────────────

fn cargo(args: &[&str]) -> Result<()> {
    command("cargo", args)
}

fn godmode(args: &[&str]) -> Result<()> {
    let mut cargo_args = vec!["run", "-q", "-p", "godmode-cli", "--"];
    cargo_args.extend_from_slice(args);
    cargo(&cargo_args)
}

fn command(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(project_root()?)
        .status()
        .with_context(|| format!("failed to run {program} {}", args.join(" ")))?;

    if !status.success() {
        bail!("{program} {} failed (exit {})", args.join(" "), status);
    }
    Ok(())
}

fn project_root() -> Result<std::path::PathBuf> {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    // xtask/Cargo.toml -> workspace root
    manifest_dir
        .parent()
        .context("could not find workspace root")
        .map(std::path::Path::to_path_buf)
}

fn header(label: &str) {
    eprintln!("\n--- {label} ---");
}
