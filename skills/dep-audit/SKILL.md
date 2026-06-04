---
name: "godmode:dep-audit"
description: >
  Audit dependencies across cargo outdated, cargo deny check, and cargo audit.
  Produce tiered findings: Blocking (CVEs), Suggestions (major bumps), Nitpicks
  (minor/patch bumps). Can auto-apply patch updates when asked.
requires: []
next: [dep-bump]
---

# Dependency Audit

## When to Use

- Before a release or merge to main
- After a vulnerability disclosure (e.g., Rust security advisory)
- When asked to "check deps", "audit dependencies", or "security audit"
- To align versions across a Rust workspace
- When suspicious or very old dependencies are discovered

## The Three-Tool Audit Process

### Tool 1: cargo outdated

Lists all crates with available updates. Categorizes by version bump type.

```bash
cargo outdated --workspace
```

Output shows:

- Package name
- Current version
- Latest available version
- Kind (normal, dev, build dependency)

Flag each update by severity:

- Major (X.0.0): breaking changes likely — Suggestion level
- Minor (x.Y.0): new features, no breaking changes — Nitpick level
- Patch (x.y.Z): bug fixes only — Nitpick level

### Tool 2: cargo deny check

Scans for:

- **Advisories**: published security vulnerabilities (CVEs)
- **Licenses**: incompatible or restricted licenses (e.g., GPL in MIT project)
- **Sources**: crates from banned sources

```bash
cargo deny check
```

Any advisory or license failure is Blocking or Suggestion level (depending on
license policy).

### Tool 3: cargo audit

Scans the vulnerability database for known CVEs in transitive dependencies.

```bash
cargo audit
```

Reports CVE ID, affected versions, and fix version. Always Blocking.

## Output Format

Report findings in three sections:

### Blocking (Security advisories, CVEs, license violations)

```
- <crate> <version> — <CVE-ID or advisory> — <description>
- <crate> — <license> incompatible with <project license>
```

### Suggestions (Major version bumps, breaking changes available)

```
- <crate> <current> → <latest> — <reason or breaking change summary>
```

### Nitpicks (Minor/patch bumps available)

```
- <crate> <current> → <latest>
```

Omit sections with no findings.

## Workspace Alignment

When a crate appears in multiple `Cargo.toml` files (workspace, sub-crates),
note version mismatches:

```
- <crate> version mismatch: root uses X, crate-foo uses Y
```

## Fix Mode

When asked to "apply patches" or "fix it":

1. For each patch bump: `cargo update -p <crate> --precise <version>`
2. Rerun `cargo audit` to verify no new conflicts introduced
3. Run `cargo check --workspace` to verify all builds succeed
4. Report: updated crates and re-audit status

**Guardrails**:

- Never apply major/minor bumps without explicit consent
- Always verify build succeeds after updates
- Never remove a dependency to resolve an advisory — report and ask instead
- If a crate requires a feature flag to pass audit, note it in findings

## When to Stop

Stop the audit if:

- `cargo outdated` output is malformed (binary missing or broken)
- `cargo deny` or `cargo audit` crash with unrecoverable error

Report the error and ask for manual investigation.
