#!/usr/bin/env nu
# rust-conventions/helpers/lint-check.nu
# Run the full Rust quality gate locally. Use before committing.

print "[rust-conventions] Running quality gate..."

let fmt = do { taskit check fmt --check } | complete
if $fmt.exit_code != 0 {
    print "[FAIL] taskit check fmt --check — run `taskit check fmt` to fix"
    exit 1
}
print "[PASS] taskit check fmt"

let clippy = do { taskit check lint } | complete
if $clippy.exit_code != 0 {
    print $"[FAIL] taskit check lint:\n($clippy.stderr)"
    exit 1
}
print "[PASS] taskit check lint"

let test = do { taskit test run } | complete
if $test.exit_code != 0 {
    print $"[FAIL] taskit test run:\n($test.stdout)"
    exit 1
}
print "[PASS] taskit test run"

print "[rust-conventions] All checks passed."
