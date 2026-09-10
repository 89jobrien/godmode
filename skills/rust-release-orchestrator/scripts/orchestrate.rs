#!/usr/bin/env rust-script
//! Rust workspace release orchestrator.
//!
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! clap = { version = "4", features = ["derive"] }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Parser)]
#[command(about = "Publish a Rust workspace in dependency order")]
struct Args {
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    resume: bool,
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
    #[arg(long)]
    skip_gates: bool,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Package {
    id: String,
    name: String,
    version: String,
    manifest_path: PathBuf,
    publish: Option<Vec<String>>,
    #[serde(default)]
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Deserialize)]
struct Dependency {
    name: String,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
struct ReleaseState {
    published: Vec<String>,
    gates_passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultStatus {
    Published,
    Skipped,
    Failed,
}

fn info(message: impl std::fmt::Display) {
    println!("[INFO]  {message}");
}
fn ok(message: impl std::fmt::Display) {
    println!("[OK]    {message}");
}
fn warn(message: impl std::fmt::Display) {
    println!("[WARN]  {message}");
}
fn fail(message: impl std::fmt::Display) {
    eprintln!("[FAIL]  {message}");
}
fn step(message: impl std::fmt::Display) {
    println!("\n══ {message} ══");
}

fn run(cwd: &Path, program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("running {program} {}", args.join(" ")))
}

fn require_success(output: Output, description: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    bail!(
        "{description} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn compute_publish_order(workspace: &Path) -> Result<Vec<Package>> {
    let output = require_success(
        run(
            workspace,
            "cargo",
            &["metadata", "--no-deps", "--format-version", "1"],
        )?,
        "cargo metadata",
    )?;
    let metadata = serde_json::from_slice(&output.stdout).context("parsing cargo metadata")?;
    publish_order(metadata)
}

fn publish_order(metadata: Metadata) -> Result<Vec<Package>> {
    let members = metadata
        .workspace_members
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut packages = metadata
        .packages
        .into_iter()
        .filter(|package| members.contains(&package.id))
        .filter(|package| {
            package
                .publish
                .as_ref()
                .is_none_or(|registries| !registries.is_empty())
        })
        .map(|package| (package.name.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let names = packages.keys().cloned().collect::<BTreeSet<_>>();
    let mut dependents = names
        .iter()
        .map(|name| (name.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut in_degree = names
        .iter()
        .map(|name| (name.clone(), 0_usize))
        .collect::<BTreeMap<_, _>>();

    for package in packages.values() {
        for dependency in &package.dependencies {
            if names.contains(&dependency.name) {
                dependents
                    .get_mut(&dependency.name)
                    .unwrap()
                    .insert(package.name.clone());
                *in_degree.get_mut(&package.name).unwrap() += 1;
            }
        }
    }

    let mut ready = in_degree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(name, _)| name.clone())
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(packages.len());
    while let Some(name) = ready.pop_front() {
        order.push(packages.remove(&name).unwrap());
        for dependent in dependents.get(&name).unwrap() {
            let degree = in_degree.get_mut(dependent).unwrap();
            *degree -= 1;
            if *degree == 0 {
                let position = ready
                    .iter()
                    .position(|queued| queued > dependent)
                    .unwrap_or(ready.len());
                ready.insert(position, dependent.clone());
            }
        }
    }
    if !packages.is_empty() {
        bail!("cycle detected in workspace dependency graph");
    }
    Ok(order)
}

fn allow_remediation(dry_run: bool) -> bool {
    !dry_run
}

fn run_gates(workspace: &Path, dry_run: bool) -> Result<bool> {
    step("Quality gates");
    let fmt = run(workspace, "cargo", &["fmt", "--all", "--check"])?;
    if !fmt.status.success() {
        if !allow_remediation(dry_run) {
            fail("fmt check failed; dry-run will not modify files");
            return Ok(false);
        }
        warn("fmt check failed — auto-formatting");
        require_success(run(workspace, "cargo", &["fmt", "--all"])?, "cargo fmt")?;
    }

    let clippy = run(
        workspace,
        "cargo",
        &["clippy", "--all-targets", "--", "-D", "warnings"],
    )?;
    if !clippy.status.success() {
        if !allow_remediation(dry_run) {
            fail("Clippy failed; dry-run will not apply fixes");
            return Ok(false);
        }
        warn("Clippy warnings found — attempting auto-fix");
        let _ = run(
            workspace,
            "cargo",
            &[
                "clippy",
                "--fix",
                "--allow-dirty",
                "--allow-staged",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        )?;
        if !run(
            workspace,
            "cargo",
            &["clippy", "--all-targets", "--", "-D", "warnings"],
        )?
        .status
        .success()
        {
            fail("Clippy still failing after auto-fix");
            return Ok(false);
        }
    }

    if !run(workspace, "cargo", &["nextest", "run"])?
        .status
        .success()
    {
        fail("Tests failed");
        return Ok(false);
    }
    ok("Quality gates passed");
    Ok(true)
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn is_already_published(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    ["already exists", "crate version `", "already uploaded"]
        .iter()
        .any(|marker| output.contains(marker))
}

fn is_rate_limited(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    ["too many requests", "rate limit", "429"]
        .iter()
        .any(|marker| output.contains(marker))
}

fn retry_delay(attempt: u32, base_ms: Option<u64>) -> Duration {
    let factor = 1_u64
        .checked_shl(attempt.saturating_sub(1))
        .unwrap_or(u64::MAX);
    match base_ms {
        Some(base) => Duration::from_millis(base.saturating_mul(factor)),
        None => Duration::from_secs(5_u64.saturating_mul(factor)),
    }
}

fn publish_crate(package: &Package, dry_run: bool, max_retries: u32) -> Result<bool> {
    let cwd = package
        .manifest_path
        .parent()
        .context("package manifest has no parent")?;
    let args = if dry_run {
        vec!["publish", "--dry-run"]
    } else {
        vec!["publish"]
    };
    let retry_base_ms = std::env::var("GODMODE_RELEASE_RETRY_BASE_MS")
        .ok()
        .and_then(|value| value.parse().ok());
    for attempt in 1..=max_retries {
        info(format!(
            "Publishing {} v{} (attempt {attempt}/{max_retries})",
            package.name, package.version
        ));
        let output = run(cwd, "cargo", &args)?;
        let text = output_text(&output);
        if output.status.success() || is_already_published(&text) {
            return Ok(true);
        }
        if attempt == max_retries {
            fail(format!("{} publish failed:\n{text}", package.name));
            return Ok(false);
        }
        let wait = retry_delay(attempt, retry_base_ms);
        if is_rate_limited(&text) {
            warn(format!(
                "Rate limited; retrying in {:.3}s",
                wait.as_secs_f64()
            ));
        } else {
            fail(format!("{} publish failed:\n{text}", package.name));
            warn(format!("Retrying in {:.3}s", wait.as_secs_f64()));
        }
        thread::sleep(wait);
    }
    Ok(false)
}

fn load_state(path: &Path) -> Result<ReleaseState> {
    if !path.exists() {
        return Ok(ReleaseState::default());
    }
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

fn save_state(path: &Path, state: &ReleaseState) -> Result<()> {
    std::fs::write(path, format!("{}\n", serde_json::to_string_pretty(state)?))
        .with_context(|| format!("writing {}", path.display()))
}

fn emit_report(
    order: &[Package],
    results: &BTreeMap<String, ResultStatus>,
    dry_run: bool,
    elapsed: Duration,
) -> bool {
    println!(
        "\n{}\n  {}RELEASE REPORT\n{}",
        "═".repeat(60),
        if dry_run { "DRY-RUN " } else { "" },
        "═".repeat(60)
    );
    for package in order {
        let (icon, label) = match results.get(&package.name) {
            Some(ResultStatus::Published) => ("✓", "published"),
            Some(ResultStatus::Skipped) => ("↩", "skipped"),
            Some(ResultStatus::Failed) => ("✗", "failed"),
            None => ("?", "?"),
        };
        println!(
            "  {icon}  {:<30} v{}  [{label}]",
            package.name, package.version
        );
    }
    let count = |status| results.values().filter(|value| **value == status).count();
    println!("{}", "─".repeat(60));
    println!("  Published : {}", count(ResultStatus::Published));
    println!("  Skipped   : {}", count(ResultStatus::Skipped));
    println!("  Failed    : {}", count(ResultStatus::Failed));
    println!("  Elapsed   : {:.1}s", elapsed.as_secs_f64());
    if dry_run {
        println!("  This was a DRY RUN — nothing was published or written.");
    }
    count(ResultStatus::Failed) == 0
}

fn main() -> Result<()> {
    let args = Args::parse();
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("resolving {}", args.workspace.display()))?;
    let state_path = workspace.join(".release-state.json");
    let started = Instant::now();
    step("Rust Workspace Release Orchestrator");
    info(format!("Workspace: {}", workspace.display()));
    let mut state = if args.resume {
        load_state(&state_path)?
    } else {
        ReleaseState::default()
    };
    let order = compute_publish_order(&workspace)?;
    for (index, package) in order.iter().enumerate() {
        println!("  {}. {} v{}", index + 1, package.name, package.version);
    }

    if !args.skip_gates && !state.gates_passed {
        if !run_gates(&workspace, args.dry_run)? {
            if !args.dry_run {
                save_state(&state_path, &state)?;
            }
            bail!("quality gates failed; fix issues and re-run with --resume");
        }
        state.gates_passed = true;
        if !args.dry_run {
            save_state(&state_path, &state)?;
        }
    }

    let max_retries = std::env::var("GODMODE_RELEASE_MAX_RETRIES")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(5);
    let mut results = BTreeMap::new();
    for package in &order {
        if state.published.contains(&package.name) {
            results.insert(package.name.clone(), ResultStatus::Skipped);
            continue;
        }
        if publish_crate(package, args.dry_run, max_retries)? {
            results.insert(package.name.clone(), ResultStatus::Published);
            if !args.dry_run {
                state.published.push(package.name.clone());
                save_state(&state_path, &state)?;
            }
        } else {
            results.insert(package.name.clone(), ResultStatus::Failed);
            if !args.dry_run {
                save_state(&state_path, &state)?;
            }
            emit_report(&order, &results, args.dry_run, started.elapsed());
            bail!("{} failed; re-run with --resume", package.name);
        }
    }
    let success = emit_report(&order, &results, args.dry_run, started.elapsed());
    if success && !args.dry_run && state_path.exists() {
        std::fs::remove_file(&state_path)?;
    }
    if !success {
        bail!("release failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, dependencies: &[&str], publish: Option<Vec<String>>) -> Package {
        Package {
            id: format!("{name} 0.1.0"),
            name: name.into(),
            version: "0.1.0".into(),
            manifest_path: PathBuf::from(format!("/{name}/Cargo.toml")),
            publish,
            dependencies: dependencies
                .iter()
                .map(|name| Dependency {
                    name: (*name).into(),
                })
                .collect(),
        }
    }

    #[test]
    fn dependencies_publish_first_and_private_crates_are_skipped() {
        let packages = vec![
            package("cli", &["core", "adapters"], None),
            package("private", &[], Some(vec![])),
            package("adapters", &["core"], None),
            package("core", &[], None),
        ];
        let workspace_members = packages.iter().map(|package| package.id.clone()).collect();
        let order = publish_order(Metadata {
            packages,
            workspace_members,
        })
        .unwrap();
        assert_eq!(
            order
                .iter()
                .map(|package| package.name.as_str())
                .collect::<Vec<_>>(),
            ["core", "adapters", "cli"]
        );
    }

    #[test]
    fn cycles_are_rejected() {
        let packages = vec![package("a", &["b"], None), package("b", &["a"], None)];
        let workspace_members = packages.iter().map(|package| package.id.clone()).collect();
        assert!(publish_order(Metadata {
            packages,
            workspace_members
        })
        .is_err());
    }

    #[test]
    fn output_classification_is_case_insensitive() {
        assert!(is_already_published(
            "Crate version `1.0.0` is already uploaded"
        ));
        assert!(is_rate_limited("HTTP 429 Too Many Requests"));
    }

    #[test]
    fn dry_run_disables_mutating_gate_remediation() {
        assert!(!allow_remediation(true));
        assert!(allow_remediation(false));
    }

    #[test]
    fn retry_delay_can_be_disabled_for_fake_cargo_integration_tests() {
        assert_eq!(retry_delay(3, Some(0)), Duration::ZERO);
        assert_eq!(retry_delay(3, Some(2)), Duration::from_millis(8));
    }

    #[test]
    fn state_roundtrips() {
        let state = ReleaseState {
            published: vec!["core".into()],
            gates_passed: true,
        };
        assert_eq!(
            serde_json::from_str::<ReleaseState>(&serde_json::to_string(&state).unwrap()).unwrap(),
            state
        );
    }
}
