IOSP (Integration/Operation Separation Principle) states that a function should either orchestrate
calls to other functions (integration) or contain logic (operations), but not both. Here's how to
handle each finding:

---

**`open` — refactor**

Checking if a file exists before reading it is a logic branch mixed with I/O calls. Extract the
existence check into a predicate or just attempt the read and handle the error:

```rust
// Before: checks existence (logic) then reads (call)
fn open(path: &Path) -> Result<String> {
    if path.exists() {
        fs::read_to_string(path)
    } else {
        Err(...)
    }
}

// After: pure operation — let the OS error propagate naturally
fn open(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(Into::into)
}
```

If distinct behavior on "not found" vs other errors is required, handle it at the call site or in
a thin integrator above `open`.

---

**`load_config` — refactor**

Parsing TOML (logic) combined with fallback defaults (more logic) mixed with the call to read the
file is the classic IOSP violation. Split into three parts:

```rust
fn read_config_file(path: &Path) -> Result<String> { ... }          // operation: I/O only
fn parse_config(raw: &str) -> Result<Config> { ... }                // operation: parse only
fn apply_defaults(cfg: Config) -> Config { ... }                    // operation: logic only

fn load_config(path: &Path) -> Result<Config> {                     // integrator
    let raw = read_config_file(path)?;
    let cfg = parse_config(&raw)?;
    Ok(apply_defaults(cfg))
}
```

This is worth refactoring — the split is clean and the operations become independently testable.

---

**`handle_request` — suppress**

A CLI dispatch function is inherently an integrator — its entire job is to look at a variant and
call the right handler. The "logic" rustqual sees is the `match` arm selection, which is routing,
not business logic. This is a well-known false positive category for IOSP checkers.

Suppress with a comment explaining why:

```rust
#[allow(clippy::iosp_violation)] // dispatch/router — intentional integrator
fn handle_request(req: Request) -> Response { ... }
```

Or whatever suppression mechanism rustqual uses (inline annotation or config entry).

---

**`process_batch` — suppress or minor refactor**

Iterating over items and calling `process_one` on each is a loop integrator — the loop body
contains no logic, it's just repeated delegation. This is another common false positive.

If the iterator chain is simple (`items.iter().map(process_one).collect()`), suppress it. If the
loop contains any inline logic (filtering, accumulating errors, conditional skips), extract that
logic into a predicate and then suppress the outer loop.

---

**Summary**

| Function         | Action   | Reason                                                      |
| ---------------- | -------- | ----------------------------------------------------------- |
| `open`           | Refactor | Remove the `exists()` guard; let I/O errors propagate       |
| `load_config`    | Refactor | Split into read / parse / defaults — three clean operations |
| `handle_request` | Suppress | Dispatch/router is a valid integrator                       |
| `process_batch`  | Suppress | Loop-over-delegating is a valid integrator pattern          |
