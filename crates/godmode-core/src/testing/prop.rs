//! Property-based testing helpers.
//!
//! Thin wrappers around [`proptest`]:
//!
//! - [`PropConfig`] — named presets (`ci`, `dev`, `exhaustive`).
//! - [`assert_round_trip`] — verifies `deserialize(serialize(x)) == x`.

pub use proptest::prelude::*;

/// Named proptest configuration presets.
pub struct PropConfig;

impl PropConfig {
    /// 64 cases — fast, catches obvious regressions. Use in CI.
    pub fn ci() -> ProptestConfig {
        ProptestConfig {
            cases: 64,
            ..ProptestConfig::default()
        }
    }

    /// 256 cases — proptest default. Use in local dev.
    pub fn dev() -> ProptestConfig {
        ProptestConfig::default()
    }

    /// 1024 cases — thorough local exploration.
    pub fn exhaustive() -> ProptestConfig {
        ProptestConfig {
            cases: 1024,
            ..ProptestConfig::default()
        }
    }
}

/// Assert that `deserialize(serialize(value)) == value` for every value
/// produced by `strategy`. Serialization via `serde_json`.
pub fn assert_round_trip<T>(strategy: impl Strategy<Value = T>)
where
    T: std::fmt::Debug + PartialEq + serde::Serialize + serde::de::DeserializeOwned,
{
    let config = PropConfig::ci();
    let mut runner = proptest::test_runner::TestRunner::new(config);
    runner
        .run(&strategy, |original| {
            let json = serde_json::to_string(&original)
                .map_err(|e| proptest::test_runner::TestCaseError::fail(e.to_string()))?;
            let restored: T = serde_json::from_str(&json)
                .map_err(|e| proptest::test_runner::TestCaseError::fail(e.to_string()))?;
            prop_assert_eq!(&original, &restored);
            Ok(())
        })
        .unwrap_or_else(|e| panic!("round-trip property failed: {e}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_config_has_reduced_cases() {
        assert_eq!(PropConfig::ci().cases, 64);
    }

    #[test]
    fn exhaustive_has_more_than_dev() {
        assert!(PropConfig::exhaustive().cases > PropConfig::dev().cases);
    }

    proptest! {
        #![proptest_config(PropConfig::ci())]

        #[test]
        fn u32_addition_is_commutative(a in 0u32..500, b in 0u32..500) {
            prop_assert_eq!(a + b, b + a);
        }
    }

    #[test]
    fn u32_round_trips() {
        assert_round_trip(any::<u32>());
    }

    #[test]
    fn string_round_trips() {
        assert_round_trip("[a-zA-Z0-9 ]{0,64}".prop_map(|s| s));
    }
}
