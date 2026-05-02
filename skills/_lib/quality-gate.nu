# Quality gate: nextest + clippy + fmt runner and command-block generator.

# Run nextest + clippy + fmt for a crate or workspace.
# Exits non-zero on any failure.
export def run-quality-gate [crate?: string] {
    let scope = if ($crate == null or ($crate | is-empty)) { ["--workspace"] } else { ["-p" $crate] }
    let r1 = (do { cargo nextest run ...$scope } | complete)
    if $r1.exit_code != 0 { exit $r1.exit_code }
    let r2 = (do { cargo clippy ...$scope -- -D warnings } | complete)
    if $r2.exit_code != 0 { exit $r2.exit_code }
    let r3 = (do { cargo fmt --all --check } | complete)
    if $r3.exit_code != 0 { exit $r3.exit_code }
}

# Return the canonical gate command block as a markdown code fence string.
export def quality-gate-cmds [crate?: string] {
    let scope = if ($crate == null or ($crate | is-empty)) { "--workspace" } else { $"-p ($crate)" }
    $"```bash
cargo nextest run ($scope)
cargo clippy ($scope) -- -D warnings
cargo fmt --all --check
```"
}
