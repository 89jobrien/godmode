# Pattern 2: TQ_UNTESTED on CLI dispatch

Read this before silencing a TQ_UNTESTED finding on `main`, `run`, or a command dispatch function in a binary or CLI crate.

## Symptom

```text
TQ_UNTESTED  src/main.rs:12  run
TQ_UNTESTED  src/cli.rs:45  dispatch_command
```

CLI dispatch functions are pure wiring — they call library functions but contain no
testable logic themselves. Integration tests in `tests/` cover them end-to-end, but
rustqual cannot see those tests.

## Correct fix

Add the dispatch functions to `ignore_functions` in `rustqual.toml`:

```toml
ignore_functions = ["main", "run", "dispatch_command", "dispatch_*"]
```

Do not move these into the library crate to make them testable — that inverts the
architecture. CLI dispatch belongs in the binary crate.
