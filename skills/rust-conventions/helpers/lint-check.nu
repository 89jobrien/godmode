#!/usr/bin/env nu
# rust-conventions/helpers/lint-check.nu
# Run the full Rust quality gate locally. Use before committing.

print "[rust-conventions] Running quality gate..."

let fmt = do { cargo fmt --all --check } | complete
if $fmt.exit_code != 0 {
    print "[FAIL] cargo fmt — run `cargo fmt --all` to fix"
    exit 1
}
print "[PASS] cargo fmt"

let clippy = do { cargo clippy --workspace -- -D warnings } | complete
if $clippy.exit_code != 0 {
    print $"[FAIL] cargo clippy:\n($clippy.stderr)"
    exit 1
}
print "[PASS] cargo clippy"

let test = do { cargo nextest run --workspace } | complete
if $test.exit_code != 0 {
    print $"[FAIL] cargo nextest:\n($test.stdout)"
    exit 1
}
print "[PASS] cargo nextest"

print "[rust-conventions] All checks passed."
