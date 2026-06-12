# rustqual CLI Reference

## Usage

```
rustqual [OPTIONS] [PATH]
```

**PATH**: File or directory to analyze. Defaults to current directory.

## Output Options

| Flag                | Description                                      |
| ------------------- | ------------------------------------------------ |
| `-v, --verbose`     | Show all functions, not just findings            |
| `--findings`        | Show only findings with file:line (one per line) |
| `--json`            | Output as JSON (for CI integration)              |
| `--format <FORMAT>` | Output format (see below)                        |

### Output Formats (`--format`)

| Format    | Use case                                          |
| --------- | ------------------------------------------------- |
| `text`    | Default human-readable output                     |
| `json`    | Machine-readable, same as `--json`                |
| `github`  | GitHub Actions annotations (`::warning file=...`) |
| `dot`     | Graphviz DOT for coupling/dependency graphs       |
| `sarif`   | SARIF for IDE integration (VS Code, etc.)         |
| `html`    | Standalone HTML report                            |
| `ai`      | Optimized for LLM consumption (compact text)      |
| `ai-json` | Structured JSON optimized for LLM tools           |

## Analysis Tuning

| Flag                         | Description                                         |
| ---------------------------- | --------------------------------------------------- |
| `--strict-closures`          | Treat closures as logic (stricter IOSP)             |
| `--strict-iterators`         | Treat `.map`, `.filter`, etc. as logic              |
| `--allow-recursion`          | Don't count recursive calls as violations           |
| `--strict-error-propagation` | Count `?` operator as logic (implicit control flow) |

These flags override the corresponding `rustqual.toml` settings for a
single run. Useful for exploring what a stricter analysis would look like
before committing to it in config.

## Configuration

| Flag                    | Description                                                   |
| ----------------------- | ------------------------------------------------------------- |
| `-c, --config <CONFIG>` | Path to config file (default: `rustqual.toml` in target dir)  |
| `--init`                | Generate a tailored `rustqual.toml` based on current codebase |

`--init` analyzes the codebase first and sets thresholds at current
maximums + 20% headroom. Run this once, then tighten thresholds over time.

## CI / Baseline

| Flag                          | Description                                        |
| ----------------------------- | -------------------------------------------------- |
| `--no-fail`                   | Don't exit 1 on findings (for local exploration)   |
| `--fail-on-warnings`          | Exit 1 on warnings too (e.g., suppression ratio)   |
| `--min-quality-score <SCORE>` | Exit 1 if quality score is below threshold (0-100) |
| `--save-baseline <FILE>`      | Save current results as a baseline JSON file       |
| `--compare <FILE>`            | Compare current results against a saved baseline   |
| `--fail-on-regression`        | Exit 1 only if score dropped vs baseline           |

### CI Workflow Example

```bash
# In CI — fail only if quality regressed
rustqual . --compare baseline.json --fail-on-regression

# Update baseline after intentional changes
rustqual . --save-baseline baseline.json
```

### Quality Gate Example

```bash
# Require minimum 80% quality score
rustqual . --min-quality-score 80
```

## Focused Analysis

| Flag               | Description                                             |
| ------------------ | ------------------------------------------------------- |
| `--diff [<REF>]`   | Only analyze files changed vs a git ref (default: HEAD) |
| `--watch`          | Re-analyze continuously on file changes                 |
| `--explain <FILE>` | Show how a file is classified for architecture rules    |
| `--suggestions`    | Show refactoring suggestions for IOSP violations        |
| `--sort-by-effort` | Sort IOSP violations by effort (highest first)          |

### Diff Mode

```bash
# Analyze only files changed since last commit
rustqual . --diff

# Analyze files changed vs a specific branch
rustqual . --diff main

# Analyze files changed vs a tag
rustqual . --diff v1.0.0
```

`--diff` conflicts with `--watch`.

## Test Quality

| Flag                     | Description                                           |
| ------------------------ | ----------------------------------------------------- |
| `--coverage <LCOV_FILE>` | Path to LCOV coverage file for TQ-004/TQ-005 analysis |

### Using Coverage Data

```bash
# Generate LCOV coverage (requires cargo-llvm-cov)
cargo llvm-cov --lcov --output-path lcov.info

# Run rustqual with coverage data
rustqual . --coverage lcov.info
```

Coverage data enables additional findings:

- **TQ-004**: Functions with 0% line coverage
- **TQ-005**: Functions below coverage threshold

## Shell Completions

```bash
# Generate completions for your shell
rustqual --completions bash > ~/.bash_completions/rustqual
rustqual --completions zsh > ~/.zfunc/_rustqual
rustqual --completions fish > ~/.config/fish/completions/rustqual.fish
rustqual --completions nu > ~/.config/nushell/completions/rustqual.nu
```

## Exit Codes

| Code | Meaning                                                                      |
| ---- | ---------------------------------------------------------------------------- |
| 0    | No findings (or `--no-fail` used)                                            |
| 1    | Findings exist, or score below `--min-quality-score`, or regression detected |
| 2    | Configuration error, parse failure, or invalid arguments                     |

## Inline Suppression Syntax

```rust
// qual:allow(iosp) reason: "integration root"
fn main() { ... }

// qual:allow(complexity) reason: "unavoidable state machine"
fn parse_token(...) { ... }

// qual:allow — suppresses ALL dimensions
fn legacy_code() { ... }
```

Dimensions: `iosp`, `complexity`, `dry`, `srp`, `coupling`, `test`,
`architecture`.

**Not suppressible inline**: DEAD_CODE (use `detect_dead_code = false`
in config).

## Config File Reference (rustqual.toml)

```toml
# ── Function Classification ─────────────────────────────────
ignore_functions = ["main", "test_*"]  # Glob patterns
exclude_files = []                      # File path patterns
strict_closures = false
strict_iterator_chains = false
allow_recursion = false
strict_error_propagation = false

# ── Suppression Health ───────────────────────────────────────
max_suppression_ratio = 0.05  # Warn if >5% of functions suppressed
fail_on_warnings = false

# ── Complexity ───────────────────────────────────────────────
[complexity]
enabled = true
max_cognitive = 15          # Max cognitive complexity per function
max_cyclomatic = 10         # Max cyclomatic complexity
max_nesting_depth = 4       # Max nesting depth
max_function_lines = 50     # Max lines per function
include_nesting_penalty = true
detect_magic_numbers = true
detect_unsafe = true
detect_error_handling = true
allowed_magic_numbers = ["0", "1", "-1", "2"]

# ── DRY / Duplicates ────────────────────────────────────────
[duplicates]
enabled = true
similarity_threshold = 0.85  # 0.0-1.0, higher = stricter
min_tokens = 30
min_lines = 5
min_statements = 3
ignore_tests = true          # Don't flag test code duplication
ignore_trait_impls = true
detect_dead_code = true      # Set false for workspace crates
detect_wildcard_imports = true
detect_repeated_matches = true

# ── Boilerplate ──────────────────────────────────────────────
[boilerplate]
enabled = true
suggest_crates = true  # Suggest thiserror, derive_more, etc.

# ── SRP ──────────────────────────────────────────────────────
[srp]
enabled = true
smell_threshold = 0.6
max_fields = 12
max_methods = 20
max_fan_out = 10
lcom4_threshold = 2        # Max independent method clusters
weights = [0.4, 0.25, 0.15, 0.2]
file_length_baseline = 300  # Ideal max file length
file_length_ceiling = 800   # Hard max file length
max_independent_clusters = 2
min_cluster_statements = 5
max_parameters = 5          # Max function parameters

# ── Coupling ─────────────────────────────────────────────────
[coupling]
enabled = true
max_instability = 0.8
max_fan_in = 15
max_fan_out = 12
check_sdp = true  # Stable Dependencies Principle

# ── Test Quality ─────────────────────────────────────────────
[test_quality]
enabled = true
# coverage_file = "lcov.info"

# ── Quality Score Weights ────────────────────────────────────
# Must sum to ~1.0
[weights]
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```
