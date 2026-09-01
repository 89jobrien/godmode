//! Test scaffold generation — produces test module stubs for each testing dimension.
//!
//! Replaces `skills/testing-philosophy/helpers/test-scaffold.nu`.

use std::fmt;

/// The seven testing dimensions (minus "model check" which has no scaffold template).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    /// Focused tests of one function or module in isolation.
    Unit,
    /// Generated-input tests of invariants.
    Property,
    /// Coverage-guided tests over arbitrary input bytes.
    Fuzz,
    /// Shared contract tests for multiple implementations.
    Conformance,
    /// Tests spanning real component boundaries.
    Integration,
    /// Tests that reproduce a previously observed defect.
    Regression,
}

impl Dimension {
    /// All dimensions supported by scaffold generation.
    pub const ALL: &[Dimension] = &[
        Dimension::Unit,
        Dimension::Property,
        Dimension::Fuzz,
        Dimension::Conformance,
        Dimension::Integration,
        Dimension::Regression,
    ];
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dimension::Unit => write!(f, "unit"),
            Dimension::Property => write!(f, "property"),
            Dimension::Fuzz => write!(f, "fuzz"),
            Dimension::Conformance => write!(f, "conformance"),
            Dimension::Integration => write!(f, "integration"),
            Dimension::Regression => write!(f, "regression"),
        }
    }
}

impl std::str::FromStr for Dimension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unit" => Ok(Dimension::Unit),
            "property" => Ok(Dimension::Property),
            "fuzz" => Ok(Dimension::Fuzz),
            "conformance" => Ok(Dimension::Conformance),
            "integration" => Ok(Dimension::Integration),
            "regression" => Ok(Dimension::Regression),
            other => Err(format!(
                "unknown dimension '{other}'; valid: unit, property, fuzz, conformance, integration, regression"
            )),
        }
    }
}

/// Generate a test stub for the given crate and dimension.
pub fn generate(crate_name: &str, dimension: Dimension) -> String {
    match dimension {
        Dimension::Unit => format!(
            r#"// crates/{crate_name}/src/<module>.rs
#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn <thing>_<scenario>_<expected>() {{
        // Arrange
        // Act
        let result = todo!("call the function");
        // Assert — must FAIL before implementation
        assert_eq!(result, todo!("expected value"));
    }}

    #[test]
    fn <thing>_invalid_input_returns_err() {{
        let result: Result<_, _> = todo!("call with invalid input");
        assert!(result.is_err());
    }}
}}"#
        ),

        Dimension::Fuzz => format!(
            r#"// fuzz/fuzz_targets/fuzz_<target>.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {{
    if let Ok(s) = std::str::from_utf8(data) {{
        // Must never panic on arbitrary input.
        let _ = {crate_name}::todo!("call the parser");
    }}
}});

// Seed corpus: fuzz/corpus/fuzz_<target>/
// Run: cargo +nightly fuzz run fuzz_<target> -- -max_total_time=60"#
        ),

        Dimension::Property => format!(
            r#"// crates/{crate_name}/src/<module>.rs
use proptest::prelude::*;

prop_compose! {{
    fn arb_<type>()(/* strategy fields */) -> <Type> {{
        todo!("construct from fields")
    }}
}}

proptest! {{
    #[test]
    fn <invariant>_holds_for_all_inputs(value in arb_<type>()) {{
        // assert the invariant — not a specific value
        prop_assert!(todo!("invariant expression"));
    }}
}}"#
        ),

        Dimension::Conformance => r#"// tests/conformance_<trait>.rs
fn assert_<trait>_contract<T: <Trait>>(mut impl_under_test: T) {
    // assert every invariant the trait doc promises
    todo!("contract assertions");
}

#[test]
fn <impl_name>_satisfies_<trait>_contract() {
    assert_<trait>_contract(<ImplName>::default());
}"#
        .to_string(),

        Dimension::Integration => r#"// tests/<feature>_integration.rs
#[test]
fn <scenario>_across_<boundary>() {
    // Arrange — real dependencies, no fakes
    // Act — exercise the wiring
    // Assert — verify the connected behaviour
    todo!();
}"#
        .to_string(),

        Dimension::Regression => format!(
            r#"// crates/{crate_name}/src/<module>.rs  (or tests/)
// Regression: <brief description of the bug>
// Reproduces: <issue/PR/commit reference>
#[test]
fn regression_<bug_description>() {{
    // Minimal input that previously caused the failure
    todo!("reproduce the failure");
    // must not panic / must return expected value
}}"#
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_dimensions_parse_roundtrip() {
        for d in Dimension::ALL {
            let s = d.to_string();
            let parsed: Dimension = s.parse().expect("should parse");
            assert_eq!(&parsed, d);
        }
    }

    #[test]
    fn invalid_dimension_returns_error() {
        let result: Result<Dimension, _> = "snapshot".parse();
        assert!(result.is_err());
    }

    #[test]
    fn unit_stub_contains_cfg_test() {
        let out = generate("my-crate", Dimension::Unit);
        assert!(out.contains("#[cfg(test)]"));
        assert!(out.contains("my-crate"));
    }

    #[test]
    fn fuzz_stub_contains_fuzz_target() {
        let out = generate("my-crate", Dimension::Fuzz);
        assert!(out.contains("fuzz_target!"));
        assert!(out.contains("my-crate"));
    }

    #[test]
    fn conformance_stub_is_crate_independent() {
        let out = generate("any-crate", Dimension::Conformance);
        assert!(out.contains("assert_<trait>_contract"));
        assert!(!out.contains("any-crate"));
    }
}
