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

    header("clippy");
    cargo(&["clippy", "--workspace", "--", "-D", "warnings"])?;

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

    eprintln!("\nAll CI checks passed.");
    Ok(())
}

// ── Dist build ─────────────────────────────────────────────────────

fn dist() -> Result<()> {
    header("release build");
    cargo(&["build", "--release", "-p", "godmode-cli"])?;

    let binary = project_root()?.join("target/release/godmode");
    eprintln!("Binary: {}", binary.display());
    Ok(())
}

// ── Install ────────────────────────────────────────────────────────

fn install() -> Result<()> {
    dist()?;

    let src = project_root()?.join("target/release/godmode");
    let dest_dir = home_dir()?.join(".cargo/bin");
    let dest = dest_dir.join("godmode");

    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("failed to create {}", dest_dir.display()))?;

    std::fs::copy(&src, &dest)
        .with_context(|| format!("failed to copy {} -> {}", src.display(), dest.display()))?;

    eprintln!("Installed: {}", dest.display());
    Ok(())
}

fn home_dir() -> Result<std::path::PathBuf> {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .context("HOME not set")
}

// ── Helpers ────────────────────────────────────────────────────────

fn cargo(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(project_root()?)
        .status()
        .with_context(|| format!("failed to run cargo {}", args.join(" ")))?;

    if !status.success() {
        bail!("cargo {} failed (exit {})", args.join(" "), status);
    }
    Ok(())
}

fn project_root() -> Result<std::path::PathBuf> {
    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    // xtask/Cargo.toml -> workspace root
    Ok(dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join("crates").exists())
        .context("could not find workspace root")?
        .to_path_buf())
}

fn header(label: &str) {
    eprintln!("\n--- {label} ---");
}
