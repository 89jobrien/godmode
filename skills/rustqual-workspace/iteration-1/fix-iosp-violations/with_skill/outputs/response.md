# IOSP Violation Analysis

Four functions flagged. Two should be suppressed; two should be refactored.

---

## Summary table

| Location           | Function         | Action   | Reason                                   |
| ------------------ | ---------------- | -------- | ---------------------------------------- |
| `src/store.rs:45`  | `open`           | Suppress | I/O boundary — existence check + read    |
| `src/store.rs:112` | `load_config`    | Refactor | Logic (defaults/fallback) is extractable |
| `src/api.rs:30`    | `handle_request` | Suppress | CLI dispatch root — inherently mixed     |
| `src/api.rs:88`    | `process_batch`  | Refactor | Iteration + delegation is separable      |

---

## `open` — suppress

`open` checks file existence then reads: this is a canonical I/O boundary. The check and the
read are inseparable at this level. Refactoring would just push the VIOLATION into a helper
(`read_data`) while leaving `open` clean — but the helper still has the violation. The net
finding count stays the same.

Suppress with a reason comment:

```rust
// qual:allow(iosp) reason: "I/O boundary — file existence check + read are inseparable"
pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
    ...
}
```

If you want a cleaner `open` for other reasons (testability, reuse), the extract-helper
pattern is valid — but do it for code quality, not to chase the score.

---

## `load_config` — refactor

TOML parsing with fallback defaults mixes two concerns: I/O (reading the TOML bytes) and
logic (applying defaults when fields are absent or the file is missing). The logic part is
independently testable and worth extracting.

**Before:**

```rust
pub fn load_config(path: &Path) -> Result<Config> {
    let raw = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut config: Config = toml::from_str(&raw)?;
    if config.timeout == 0 {          // logic
        config.timeout = 30;
    }
    if config.retries == 0 {          // logic
        config.retries = 3;
    }
    Ok(config)
}
```

**After:**

```rust
// qual:allow(iosp) reason: "I/O boundary — file may or may not exist"
fn read_config_raw(path: &Path) -> Result<String> {
    if path.exists() {
        Ok(fs::read_to_string(path)?)
    } else {
        Ok(String::new())
    }
}

fn apply_defaults(mut config: Config) -> Config {
    if config.timeout == 0 { config.timeout = 30; }
    if config.retries == 0 { config.retries = 3; }
    config
}

pub fn load_config(path: &Path) -> Result<Config> {
    let raw = read_config_raw(path)?;
    let config: Config = toml::from_str(&raw)?;
    Ok(apply_defaults(config))
}
```

`load_config` becomes a pure orchestrator (calls only). `apply_defaults` is pure logic with
no I/O — unit-testable without a filesystem. `read_config_raw` still has a VIOLATION (it's
the I/O boundary), so suppress it inline.

Net result: `load_config` is clean, `apply_defaults` is fully testable, one suppression on
`read_config_raw`.

---

## `handle_request` — suppress

CLI dispatch functions inherently match on input and call different handlers — that is their
job. A match arm that calls `handle_add()` or `handle_delete()` is not a logic/call mix in
any meaningful sense; the match IS the dispatch logic. rustqual cannot distinguish dispatch
from embedded business logic.

Suppress unconditionally:

```rust
// qual:allow(iosp) reason: "CLI dispatch root — match dispatch is not embeddable logic"
pub fn handle_request(req: &Request) -> Result<Response> {
    ...
}
```

Do not refactor this function to satisfy IOSP. Extracting a `dispatch()` helper just moves
the violation one level down.

---

## `process_batch` — refactor

`process_batch` iterates over items and calls `process_one` on each. If the iteration
contains any conditional logic (filtering, early exit, accumulation decisions), that logic is
extractable. If it is a pure `for item in items { process_one(item) }` loop, the violation
is borderline — but the refactor is still worthwhile because it separates iteration policy
from item processing.

**Before:**

```rust
pub fn process_batch(items: &[Item]) -> Vec<Result<Output>> {
    let mut results = Vec::new();
    for item in items {
        if item.is_active() {           // logic
            results.push(process_one(item));
        }
    }
    results
}
```

**After:**

```rust
fn active_items(items: &[Item]) -> impl Iterator<Item = &Item> {
    items.iter().filter(|i| i.is_active())
}

pub fn process_batch(items: &[Item]) -> Vec<Result<Output>> {
    active_items(items).map(process_one).collect()
}
```

`process_batch` becomes calls-only. `active_items` is pure logic — testable without touching
`process_one`. If there is no filtering logic and the loop body is literally just
`process_one(item)`, the function is marginal; suppress it:

```rust
// qual:allow(iosp) reason: "pure delegation loop — no embeddable logic"
pub fn process_batch(items: &[Item]) -> Vec<Result<Output>> {
    items.iter().map(process_one).collect()
}
```

---

## Decision guide

| Pattern                               | Action                                |
| ------------------------------------- | ------------------------------------- |
| I/O boundary (exist check + read)     | Suppress                              |
| CLI dispatch (match on command/route) | Suppress                              |
| Logic mixed with I/O calls            | Refactor — extract logic into pure fn |
| Iteration with filtering/accumulation | Refactor — extract predicate/reducer  |
| Pure delegation loop (no conditions)  | Suppress if trivial                   |

The key test: can the logic be independently unit-tested if extracted? If yes, extract it.
If the function is inherently a coordination point (dispatch, I/O boundary), suppress it.

---

## Watch out: splitting can increase TQ_UNTESTED

Each extracted helper is a new function rustqual will check for test coverage. Only split
when the resulting function is independently testable and worth a unit test. `apply_defaults`
and the `active_items` filter above both qualify. A trivial one-liner helper does not.
