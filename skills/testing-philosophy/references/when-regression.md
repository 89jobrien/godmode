# Regression tests

Read this before fixing a bug, so the fix lands with a test that stops it recurring.

### Regression — after every bug fix

Every bug that reaches production or a failing test represents a gap in the test suite.
Close that gap permanently.

1. Reproduce the bug with a minimal failing test (unit or property)
2. Fix the bug
3. Verify the test now passes
4. If it was a property test failure, commit the regression file

Never fix a bug without a regression test. "It's fixed" without a test means it will
recur.
