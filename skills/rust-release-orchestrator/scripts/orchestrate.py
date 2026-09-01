#!/usr/bin/env python3
"""
Autonomous Rust workspace release orchestrator.

Usage:
  python orchestrate.py [--dry-run] [--resume] [--workspace <path>] [--skip-gates]

State file: .release-state.json in the workspace root.
Resume: re-run with --resume; already-published crates are skipped.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

# ── ANSI colours ──────────────────────────────────────────────────────────────
GREEN = "\033[92m"
YELLOW = "\033[93m"
RED = "\033[91m"
CYAN = "\033[96m"
BOLD = "\033[1m"
RESET = "\033[0m"


def info(msg):
    print(f"{CYAN}[INFO]{RESET}  {msg}")


def ok(msg):
    print(f"{GREEN}[OK]{RESET}    {msg}")


def warn(msg):
    print(f"{YELLOW}[WARN]{RESET}  {msg}")


def fail(msg):
    print(f"{RED}[FAIL]{RESET}  {msg}", file=sys.stderr)


def step(msg):
    print(f"\n{BOLD}{CYAN}══ {msg} ══{RESET}")


# ── Shell helpers ──────────────────────────────────────────────────────────────


def run(cmd, cwd=None, capture=False, check=True):
    """Run a shell command; return (stdout, returncode)."""
    result = subprocess.run(
        cmd,
        shell=True,
        cwd=cwd,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
        text=True,
    )
    if check and result.returncode != 0:
        stdout = result.stdout or ""
        stderr = result.stderr or ""
        raise RuntimeError(f"Command failed: {cmd}\n{stdout}\n{stderr}")
    return (result.stdout or ""), result.returncode


# ── Dependency graph ───────────────────────────────────────────────────────────


def compute_publish_order(workspace_root: Path) -> list[dict]:
    """
    Parse `cargo metadata` and return publishable crates in topological order
    (dependencies before dependents).
    """
    raw, _ = run(
        "cargo metadata --no-deps --format-version 1", cwd=workspace_root, capture=True
    )
    meta = json.loads(raw)

    # Map name -> package info for workspace members
    members = {
        p["name"]: p for p in meta["packages"] if p["id"] in meta["workspace_members"]
    }
    publishable = {
        name: pkg for name, pkg in members.items() if pkg.get("publish") != []
    }

    # Build adjacency: dep_name -> set of pkgs that depend on it
    rdeps: dict[str, set] = {n: set() for n in publishable}
    for name, pkg in publishable.items():
        for dep in pkg.get("dependencies", []):
            dep_name = dep["name"]
            if dep_name in publishable:
                rdeps[dep_name].add(name)

    # Kahn's algorithm for topological sort
    in_degree = {n: 0 for n in publishable}
    for name in publishable:
        for dependent in rdeps[name]:
            in_degree[dependent] += 1

    queue = [n for n, d in in_degree.items() if d == 0]
    order = []
    while queue:
        queue.sort()  # deterministic
        n = queue.pop(0)
        order.append(publishable[n])
        for dependent in sorted(rdeps[n]):
            in_degree[dependent] -= 1
            if in_degree[dependent] == 0:
                queue.append(dependent)

    if len(order) != len(publishable):
        raise RuntimeError("Cycle detected in workspace dependency graph!")

    return order


# ── Quality gates ──────────────────────────────────────────────────────────────


def run_gates(workspace_root: Path, dry_run: bool) -> bool:
    """Run fmt / clippy / nextest. Auto-fix what can be fixed. Return True on success."""
    step("Quality gates")

    # fmt
    info("cargo fmt --all --check …")
    _, rc = run(
        "cargo fmt --all --check", cwd=workspace_root, capture=True, check=False
    )
    if rc != 0:
        warn("fmt check failed — auto-formatting …")
        run("cargo fmt --all", cwd=workspace_root)
        ok("Auto-formatted")
    else:
        ok("fmt clean")

    # clippy
    info("cargo clippy --all-targets …")
    out, rc = run(
        "cargo clippy --all-targets -- -D warnings 2>&1",
        cwd=workspace_root,
        capture=True,
        check=False,
    )
    if rc != 0:
        warn("Clippy warnings found — attempting auto-fix …")
        _, fix_rc = run(
            "cargo clippy --fix --allow-dirty --allow-staged --all-targets -- -D warnings 2>&1",
            cwd=workspace_root,
            capture=True,
            check=False,
        )
        # Re-check
        _, rc2 = run(
            "cargo clippy --all-targets -- -D warnings 2>&1",
            cwd=workspace_root,
            capture=True,
            check=False,
        )
        if rc2 != 0:
            fail("Clippy still failing after auto-fix. Fix manually and re-run.")
            return False
        ok("Clippy clean after auto-fix")
    else:
        ok("Clippy clean")

    # tests
    info("cargo nextest run …")
    _, rc = run("cargo nextest run 2>&1", cwd=workspace_root, capture=True, check=False)
    if rc != 0:
        fail("Tests failed. Fix and re-run.")
        return False
    ok("All tests passed")

    return True


# ── Publish with retry ─────────────────────────────────────────────────────────

ALREADY_PUBLISHED_MARKERS = [
    "already exists",
    "crate version `",  # "crate version `X.Y.Z` is already uploaded"
    "already uploaded",
]
RATE_LIMIT_MARKERS = [
    "too many requests",
    "rate limit",
    "429",
]


def _is_already_published(output: str) -> bool:
    lo = output.lower()
    return any(m in lo for m in ALREADY_PUBLISHED_MARKERS)


def _is_rate_limited(output: str) -> bool:
    lo = output.lower()
    return any(m in lo for m in RATE_LIMIT_MARKERS)


def publish_crate(
    pkg: dict, workspace_root: Path, dry_run: bool, max_retries: int = 5
) -> bool:
    """Publish one crate with exponential-backoff retry. Return True on success."""
    name = pkg["name"]
    version = pkg["version"]
    crate_path = Path(pkg["manifest_path"]).parent

    flag = "--dry-run" if dry_run else ""
    cmd = f"cargo publish {flag} 2>&1"

    delay = 5  # seconds
    for attempt in range(1, max_retries + 1):
        info(f"Publishing {name} v{version} (attempt {attempt}/{max_retries}) …")
        out, rc = run(cmd, cwd=crate_path, capture=True, check=False)

        if rc == 0:
            ok(f"{name} v{version} published {'(dry-run)' if dry_run else ''}")
            return True

        if _is_already_published(out):
            ok(f"{name} v{version} already published — treating as success")
            return True

        if _is_rate_limited(out):
            wait = delay * (2 ** (attempt - 1))
            warn(f"Rate limited. Waiting {wait}s before retry …")
            time.sleep(wait)
            continue

        # Hard failure
        fail(f"{name} publish failed:\n{out}")
        if attempt < max_retries:
            wait = delay * (2 ** (attempt - 1))
            warn(f"Retrying in {wait}s …")
            time.sleep(wait)
        else:
            return False

    return False


# ── State management ───────────────────────────────────────────────────────────


def load_state(state_path: Path) -> dict:
    if state_path.exists():
        return json.loads(state_path.read_text())
    return {"published": [], "gates_passed": False}


def save_state(state_path: Path, state: dict):
    state_path.write_text(json.dumps(state, indent=2))


def clear_state(state_path: Path):
    if state_path.exists():
        state_path.unlink()


# ── Report ─────────────────────────────────────────────────────────────────────


def emit_report(order: list[dict], results: dict, dry_run: bool, elapsed: float):
    width = 60
    print("\n" + "═" * width)
    print(f"  {'DRY-RUN ' if dry_run else ''}RELEASE REPORT")
    print("═" * width)
    published = [r for r in results.values() if r == "published"]
    skipped = [r for r in results.values() if r == "skipped"]
    failed = [r for r in results.values() if r == "failed"]

    for pkg in order:
        name = pkg["name"]
        ver = pkg["version"]
        status = results.get(name, "?")
        icon = {"published": "✓", "skipped": "↩", "failed": "✗"}.get(status, "?")
        colour = {"published": GREEN, "skipped": YELLOW, "failed": RED}.get(
            status, RESET
        )
        print(f"  {colour}{icon}{RESET}  {name:<30} v{ver}  [{status}]")

    print("─" * width)
    print(f"  Published : {len(published)}")
    print(f"  Skipped   : {len(skipped)}  (already on crates.io)")
    print(f"  Failed    : {len(failed)}")
    print(f"  Elapsed   : {elapsed:.1f}s")
    if dry_run:
        print(
            f"\n  {YELLOW}This was a DRY RUN — nothing was actually published.{RESET}"
        )
        print(f"  Re-run without --dry-run to publish for real.")
    print("═" * width + "\n")
    return len(failed) == 0


# ── Main ───────────────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(description="Rust workspace release orchestrator")
    parser.add_argument(
        "--dry-run", action="store_true", help="Skip actual cargo publish"
    )
    parser.add_argument(
        "--resume", action="store_true", help="Resume from previous state"
    )
    parser.add_argument("--workspace", default=".", help="Path to workspace root")
    parser.add_argument(
        "--skip-gates", action="store_true", help="Skip fmt/clippy/test"
    )
    args = parser.parse_args()

    workspace = Path(args.workspace).resolve()
    state_path = workspace / ".release-state.json"
    t0 = time.time()

    step("Rust Workspace Release Orchestrator")
    info(f"Workspace : {workspace}")
    info(f"Dry-run   : {args.dry_run}")
    info(f"Resume    : {args.resume}")

    # State
    state = (
        load_state(state_path)
        if args.resume
        else {"published": [], "gates_passed": False}
    )

    # 1. Compute publish order
    step("Computing publish order")
    order = compute_publish_order(workspace)
    info(f"Publish order ({len(order)} crates):")
    for i, pkg in enumerate(order, 1):
        print(f"  {i}. {pkg['name']} v{pkg['version']}")

    # 2. Quality gates (skip if already passed in this session and resuming)
    if not args.skip_gates and not state.get("gates_passed"):
        if not run_gates(workspace, args.dry_run):
            fail(
                "Quality gates failed. Fix issues and re-run (add --resume to continue from here)."
            )
            state["gates_passed"] = False
            save_state(state_path, state)
            sys.exit(1)
        state["gates_passed"] = True
        save_state(state_path, state)
    elif state.get("gates_passed"):
        ok("Quality gates already passed (resuming) — skipping")

    # 3. Publish each crate in order
    step("Publishing crates")
    results = {}
    for pkg in order:
        name = pkg["name"]
        if name in state["published"]:
            ok(f"{name} already published in previous run — skipping")
            results[name] = "skipped"
            continue

        success = publish_crate(pkg, workspace, args.dry_run)
        if success:
            results[name] = "published"
            if not args.dry_run:
                state["published"].append(name)
                save_state(state_path, state)
        else:
            results[name] = "failed"
            fail(f"{name} failed. State saved. Re-run with --resume to continue.")
            save_state(state_path, state)
            # Emit partial report and exit
            emit_report(order, results, args.dry_run, time.time() - t0)
            sys.exit(1)

    # 4. Report
    elapsed = time.time() - t0
    success = emit_report(order, results, args.dry_run, elapsed)

    # Clean up state on full success
    if success and not args.dry_run:
        clear_state(state_path)
        info("State file cleaned up")

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
