// Test stub — copy into the relevant module's #[cfg(test)] block
// Replace <Thing>, <scenario>, and <expected> throughout.

#[cfg(test)]
mod tests {
    use super::*;

    // --- Happy path ---

    #[test]
    fn <thing>_<scenario>_returns_<expected>() {
        // Arrange
        let input = todo!("provide input");

        // Act
        let result = todo!("call the function");

        // Assert — this MUST FAIL before implementation exists
        assert_eq!(result, todo!("expected value"));
    }

    // --- Error path ---

    #[test]
    fn <thing>_invalid_input_returns_err() {
        let result: Result<_, _> = todo!("call with invalid input");
        assert!(result.is_err());
    }

    // --- Edge case ---

    #[test]
    fn <thing>_empty_input_<expected>() {
        todo!("implement edge case test");
    }
}
