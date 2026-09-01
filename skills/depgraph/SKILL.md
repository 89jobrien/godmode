---
name: "godmode:depgraph"
description: >
  Generate a hexagonal architecture report for a Rust workspace. Visualizes
  crate dependencies as concentric rings, color-codes good (adjacent-ring) vs
  bad (ring-skipping) deps, computes an arch health score, lists actionable
  violations with remediation, and shows a branch evolution timeline. Use when
  the user says "depgraph", "architecture report", "dependency diagram",
  "hexagonal map", or "arch health".
requires: ["cargo-depgraph"]
next: [health-score, dep-audit, code-review]
---

# Depgraph -- Hexagonal Architecture Report

Generate a self-contained dark-mode HTML report showing workspace crate
dependencies as a hexagonal architecture diagram with actionable violation
tracking.

## Prerequisites

- `cargo-depgraph` (`cargo install cargo-depgraph`)
- Python 3 (stdlib only, no pip deps)

## When to Run

- Before a release or large merge to audit architectural drift
- After adding a new workspace crate
- When reviewing a branch that touches cross-crate boundaries
- On demand to check arch health score trend

## Usage

The script lives at `xtask/scripts/depgraph-report.py` in any repo that
has it, or can be copied from the skill helpers directory.

```bash
# Current branch vs main, open in browser
python3 xtask/scripts/depgraph-report.py --open

# Specific branch and base
python3 xtask/scripts/depgraph-report.py --branch feat/foo --base develop

# Custom output
python3 xtask/scripts/depgraph-report.py --output /tmp/report.html

# Different repo
python3 xtask/scripts/depgraph-report.py --repo ~/dev/other-workspace --open
```

## Process

### Step 1: Run the report

```bash
python3 xtask/scripts/depgraph-report.py --repo <workspace> --open
```

If the script is not present in the target repo, copy it from the skill
helpers directory:

```bash
mkdir -p <repo>/xtask/scripts
cp "$CLAUDE_PLUGIN_ROOT/skills/depgraph/helpers/depgraph-report.py" <repo>/xtask/scripts/
cp "$CLAUDE_PLUGIN_ROOT/skills/depgraph/helpers/depgraph_layout.py" <repo>/xtask/scripts/
```

### Step 2: Read the health score

The top-level health score is the percentage of deps that follow ring
adjacency (green). Thresholds:

| Score  | Color | Meaning                                      |
| ------ | ----- | -------------------------------------------- |
| >= 80% | Green | Clean layering, minor violations at most     |
| 60-79% | Amber | Structural debt accumulating, review actions |
| < 60%  | Red   | Significant architectural violations present |

### Step 3: Triage action items

Action items are sorted by severity:

- **critical** -- inverted dependency (inner ring depends on outer). Fix
  immediately by extracting a trait into the inner ring.
- **high** -- skips 2+ rings. Introduce a facade or re-export through
  the adjacent ring.
- **medium** -- skips 1 ring. Re-export needed types through the
  intermediate layer.
- **low** -- same-ring dependency. Consider whether one crate belongs
  one ring lower.

### Step 4: Plan fixes

For each action item:

1. Identify what types/traits the source crate actually uses from the
   target
2. Determine the correct intermediate crate (one ring inward from
   source)
3. Add a re-export, facade module, or trait extraction in that
   intermediate crate
4. Update the source crate to depend on the intermediate instead

### Step 5: Re-run and verify

After fixes, re-run the report and confirm the health score improved
and the specific violation is gone.

## Report Sections

| Section            | Content                                          |
| ------------------ | ------------------------------------------------ |
| Health score       | % of good deps, color-coded bar                  |
| Hexagonal map      | SVG diagram with green/red edge coloring         |
| Action items       | Severity-sorted violations with remediation text |
| Architecture rings | Ring descriptions with member crates             |
| Fan-in / fan-out   | Per-crate dependency counts                      |
| Evolution timeline | Branch commit activity by day and category       |
| Insights           | Auto-generated observations about the graph      |

## Ring Classification

Rings are assigned automatically by topological sort depth:

- **Core** (layer 0) -- leaf crates with no workspace deps (traits,
  contracts)
- **Domain** (layer 1) -- crates that depend only on Core
- **Adapters** (layer 2) -- crates that implement Core traits against
  real backends
- **Applications** (layer 3 and deeper) -- outer orchestration crates. Layers
  deeper than three are capped at this ring.

Shallow workspaces retain their actual depth, so a root at layer 1 remains in
the Domain ring instead of being stretched to Applications.

## Edge Classification

- **Good** (green) -- source ring is exactly 1 higher than target ring.
  Clean adjacent-layer dependency.
- **Bad** (red) -- any other case: same ring, ring-skipping, or
  inverted. Each gets a severity and remediation.
