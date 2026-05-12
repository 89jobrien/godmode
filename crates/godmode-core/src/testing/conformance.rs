//! Trait-conformance test runner.
//!
//! Enforces the Liskov Substitution Principle in tests: every adapter that
//! implements a port trait must pass the same behavioural suite.
//!
//! # Design
//!
//! - [`ConformanceCase`] is the port — one trait per behavioural requirement.
//! - [`ConformanceSuite`] is the runner — collects cases and runs them against
//!   a concrete adapter supplied at test time.
//! - Adding a new adapter never touches existing case code (OCP).
//! - Each case depends only on the port trait, not on adapter internals (DIP).
//!
//! # Example
//!
//! ```rust,ignore
//! use godmode_core::testing::conformance::{ConformanceCase, ConformanceSuite};
//!
//! trait GreetPort {
//!     fn greet(&self, name: &str) -> String;
//! }
//!
//! struct GreetsWithName;
//! impl ConformanceCase<dyn GreetPort> for GreetsWithName {
//!     fn name(&self) -> &str { "greets_with_name" }
//!     fn run(&self, subject: &dyn GreetPort) -> Result<(), String> {
//!         let result = subject.greet("World");
//!         if result.contains("World") { Ok(()) }
//!         else { Err(format!("expected 'World' in {:?}", result)) }
//!     }
//! }
//!
//! fn greeter_suite() -> ConformanceSuite<dyn GreetPort> {
//!     ConformanceSuite::new().case(Box::new(GreetsWithName))
//! }
//!
//! #[test]
//! fn hello_adapter_conforms() {
//!     greeter_suite().assert_all(&HelloAdapter);
//! }
//! ```

/// A single behavioural requirement that any conforming adapter must satisfy.
///
/// `Port` is typically a `dyn Trait` — the abstraction boundary being tested.
pub trait ConformanceCase<Port: ?Sized> {
    /// Short identifier shown in failure messages (snake_case recommended).
    fn name(&self) -> &str;

    /// Execute the case against `subject`.
    ///
    /// Return `Ok(())` on pass, `Err(reason)` on failure.
    fn run(&self, subject: &Port) -> Result<(), String>;
}

/// Outcome of a single conformance case run.
#[derive(Debug)]
pub struct CaseResult {
    pub name: String,
    pub outcome: Result<(), String>,
}

/// Collects [`ConformanceCase`]s and runs them against a concrete adapter.
pub struct ConformanceSuite<Port: ?Sized> {
    cases: Vec<Box<dyn ConformanceCase<Port>>>,
}

impl<Port: ?Sized> ConformanceSuite<Port> {
    pub fn new() -> Self {
        Self { cases: Vec::new() }
    }

    /// Append a case to the suite (builder style).
    pub fn case(mut self, case: Box<dyn ConformanceCase<Port>>) -> Self {
        self.cases.push(case);
        self
    }

    /// Run all cases against `subject` and return per-case results.
    pub fn run_all(&self, subject: &Port) -> Vec<CaseResult> {
        self.cases
            .iter()
            .map(|c| CaseResult {
                name: c.name().to_owned(),
                outcome: c.run(subject),
            })
            .collect()
    }

    /// Run all cases and panic if any fail, printing every failure.
    pub fn assert_all(&self, subject: &Port) {
        let results = self.run_all(subject);
        let failures: Vec<&CaseResult> = results.iter().filter(|r| r.outcome.is_err()).collect();

        if !failures.is_empty() {
            let msgs: Vec<String> = failures
                .iter()
                .map(|r| format!("  FAIL [{}]: {}", r.name, r.outcome.as_ref().unwrap_err()))
                .collect();
            panic!(
                "Conformance suite failed ({}/{} cases):\n{}",
                failures.len(),
                results.len(),
                msgs.join("\n")
            );
        }
    }
}

impl<Port: ?Sized> Default for ConformanceSuite<Port> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Echo {
        fn echo(&self, s: &str) -> String;
    }

    struct EchoesSameString;
    impl ConformanceCase<dyn Echo> for EchoesSameString {
        fn name(&self) -> &str {
            "echoes_same_string"
        }
        fn run(&self, subject: &dyn Echo) -> Result<(), String> {
            let out = subject.echo("hello");
            if out == "hello" {
                Ok(())
            } else {
                Err(format!("expected 'hello', got '{out}'"))
            }
        }
    }

    struct EchoesNonEmpty;
    impl ConformanceCase<dyn Echo> for EchoesNonEmpty {
        fn name(&self) -> &str {
            "echoes_non_empty"
        }
        fn run(&self, subject: &dyn Echo) -> Result<(), String> {
            let out = subject.echo("x");
            if !out.is_empty() {
                Ok(())
            } else {
                Err("output was empty".to_owned())
            }
        }
    }

    struct IdentityEcho;
    impl Echo for IdentityEcho {
        fn echo(&self, s: &str) -> String {
            s.to_owned()
        }
    }

    struct UppercaseEcho;
    impl Echo for UppercaseEcho {
        fn echo(&self, s: &str) -> String {
            s.to_uppercase()
        }
    }

    fn echo_suite() -> ConformanceSuite<dyn Echo> {
        ConformanceSuite::new()
            .case(Box::new(EchoesSameString))
            .case(Box::new(EchoesNonEmpty))
    }

    #[test]
    fn identity_echo_passes_all() {
        let results = echo_suite().run_all(&IdentityEcho);
        assert!(results.iter().all(|r| r.outcome.is_ok()));
    }

    #[test]
    fn uppercase_echo_fails_same_string_case() {
        let results = echo_suite().run_all(&UppercaseEcho);
        let same_string = results
            .iter()
            .find(|r| r.name == "echoes_same_string")
            .unwrap();
        assert!(same_string.outcome.is_err());
        let non_empty = results
            .iter()
            .find(|r| r.name == "echoes_non_empty")
            .unwrap();
        assert!(non_empty.outcome.is_ok());
    }

    #[test]
    #[should_panic(expected = "Conformance suite failed")]
    fn assert_all_panics_on_failure() {
        echo_suite().assert_all(&UppercaseEcho);
    }

    #[test]
    fn assert_all_passes_for_conforming_adapter() {
        echo_suite().assert_all(&IdentityEcho);
    }
}
