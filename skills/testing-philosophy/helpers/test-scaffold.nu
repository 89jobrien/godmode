#!/usr/bin/env nu
# test-scaffold.nu — print a test module stub for a given crate and dimension.
# Usage: nu skills/testing-philosophy/helpers/test-scaffold.nu <crate> <dimension>
# Dimensions: unit | property | conformance | integration | regression

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [crate: string, dimension: string] {
    let valid = ["unit" "property" "conformance" "integration" "regression"]
    if not ($valid | any { |d| $d == $dimension }) {
        print $"ERROR: dimension must be one of: ($valid | str join ', ')"
        exit 1
    }

    let tid = (trace-start "testing-philosophy" "test-scaffold.nu" $crate $dimension)

    let stub = match $dimension {
        "unit" => $"// crates/($crate)/src/<module>.rs
#[cfg\(test\)]
mod tests \{
    use super::*;

    #[test]
    fn <thing>_<scenario>_<expected>\(\) \{
        // Arrange
        // Act
        let result = todo!\(\"call the function\"\);
        // Assert — must FAIL before implementation
        assert_eq!\(result, todo!\(\"expected value\"\)\);
    \}

    #[test]
    fn <thing>_invalid_input_returns_err\(\) \{
        let result: Result<_, _> = todo!\(\"call with invalid input\"\);
        assert!\(result.is_err\(\)\);
    \}
\}"

        "property" => $"// crates/($crate)/src/<module>.rs
use proptest::prelude::*;

prop_compose! \{
    fn arb_<type>\(\)\(/* strategy fields */\) -> <Type> \{
        todo!\(\"construct from fields\"\)
    \}
\}

proptest! \{
    #[test]
    fn <invariant>_holds_for_all_inputs\(value in arb_<type>\(\)\) \{
        // assert the invariant — not a specific value
        prop_assert!\(todo!\(\"invariant expression\"\)\);
    \}
\}"

        "conformance" => $"// tests/conformance_<trait>.rs
fn assert_<trait>_contract<T: <Trait>>\(mut impl_under_test: T\) \{
    // assert every invariant the trait doc promises
    todo!\(\"contract assertions\"\);
\}

#[test]
fn <impl_name>_satisfies_<trait>_contract\(\) \{
    assert_<trait>_contract\(<ImplName>::default\(\)\);
\}"

        "integration" => $"// tests/<feature>_integration.rs
#[test]
fn <scenario>_across_<boundary>\(\) \{
    // Arrange — real dependencies, no fakes
    // Act — exercise the wiring
    // Assert — verify the connected behaviour
    todo!\(\);
\}"

        "regression" => $"// crates/($crate)/src/<module>.rs  (or tests/)
// Regression: <brief description of the bug>
// Reproduces: <issue/PR/commit reference>
#[test]
fn regression_<bug_description>\(\) \{
    // Minimal input that previously caused the failure
    todo!\(\"reproduce the failure\"\);
    // must not panic / must return expected value
\}"

        _ => ""
    }

    print $stub
    trace-end $tid
}
