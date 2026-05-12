//! Environment variable isolation for tests.
//!
//! [`TestContext`] is an RAII guard that sets env vars for the duration of a
//! test and restores their previous values on drop.
//!
//! # Thread safety
//!
//! Env var mutation is process-global. Tests using this guard must not run in
//! parallel within the same process if they touch the same keys. Use
//! `#[serial_test::serial]` or restrict each test to unique key names.

struct SavedVar {
    key: String,
    prior: Option<String>,
}

/// RAII guard that sets env vars for the duration of a test and restores
/// their previous values on drop.
///
/// ```rust,ignore
/// use godmode_core::testing::env::TestContext;
///
/// let _ctx = TestContext::builder()
///     .env("MY_API_URL", "http://test:9999")
///     .env("MY_TEST_MODE", "1")
///     .build();
///
/// assert_eq!(std::env::var("MY_API_URL").unwrap(), "http://test:9999");
/// // restored on drop
/// ```
pub struct TestContext {
    saved: Vec<SavedVar>,
}

impl TestContext {
    pub fn builder() -> TestContextBuilder {
        TestContextBuilder::default()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        for saved in self.saved.drain(..) {
            // SAFETY: single-threaded test context — caller is responsible
            // for serialising parallel tests that touch the same keys.
            match &saved.prior {
                Some(v) => unsafe { std::env::set_var(&saved.key, v) },
                None => unsafe { std::env::remove_var(&saved.key) },
            }
        }
    }
}

/// Builder for [`TestContext`].
#[derive(Default)]
pub struct TestContextBuilder {
    overrides: std::collections::HashMap<String, String>,
}

impl TestContextBuilder {
    /// Override an environment variable for the test scope.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    /// Apply all overrides and return the active [`TestContext`] guard.
    pub fn build(self) -> TestContext {
        let mut saved = Vec::with_capacity(self.overrides.len());
        for (key, value) in self.overrides {
            let prior = std::env::var(&key).ok();
            // SAFETY: same as Drop impl — single-threaded test context.
            unsafe { std::env::set_var(&key, &value) };
            saved.push(SavedVar { key, prior });
        }
        TestContext { saved }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_sets_and_restores() {
        let key = "GODMODE_TEST_CTX_PROBE";
        unsafe { std::env::remove_var(key) };

        {
            let _ctx = TestContext::builder().env(key, "hello").build();
            assert_eq!(std::env::var(key).unwrap(), "hello");
        }

        assert!(std::env::var(key).is_err());
    }

    #[test]
    fn test_context_restores_previous_value() {
        let key = "GODMODE_TEST_CTX_RESTORE";
        unsafe { std::env::set_var(key, "original") };

        {
            let _ctx = TestContext::builder().env(key, "override").build();
            assert_eq!(std::env::var(key).unwrap(), "override");
        }

        assert_eq!(std::env::var(key).unwrap(), "original");
        unsafe { std::env::remove_var(key) };
    }
}
