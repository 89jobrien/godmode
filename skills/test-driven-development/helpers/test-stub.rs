// Copy-paste starting point for a new test module.
// Place inside the file under test (unit) or in tests/ (integration).

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // Helpers / fixtures
    // ---------------------------------------------------------------------------

    fn make_subject() -> () {
        // TODO: construct the type under test
        ()
    }

    // ---------------------------------------------------------------------------
    // Happy-path tests
    // ---------------------------------------------------------------------------

    #[test]
    fn it_does_the_thing() {
        // Arrange
        let subject = make_subject();

        // Act
        // let result = subject.do_thing();

        // Assert
        // assert_eq!(result, expected);
        let _ = subject;
    }

    // ---------------------------------------------------------------------------
    // Error / edge-case tests
    // ---------------------------------------------------------------------------

    #[test]
    fn it_returns_error_on_empty_input() {
        // Arrange

        // Act

        // Assert
        // assert!(result.is_err());
    }

    // ---------------------------------------------------------------------------
    // Trait-double example (in-memory fake — no mocked HTTP/DB)
    // ---------------------------------------------------------------------------

    // struct FakeRepo {
    //     items: Vec<String>,
    // }
    //
    // impl MyPort for FakeRepo {
    //     fn fetch(&self, id: &str) -> Option<&str> {
    //         self.items.iter().find(|s| s.starts_with(id)).map(|s| s.as_str())
    //     }
    // }
}
