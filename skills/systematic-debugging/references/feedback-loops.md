# Feedback Loop Recipes

Concrete loops per bug class, roughly cheapest first. All examples assume a Rust workspace at
the repo root. Adapt the paths; the structure is the point.

## Tightening an existing loop

| Symptom                            | Fix                                                                 |
| ---------------------------------- | ------------------------------------------------------------------- |
| Loop takes more than 10s           | Narrow the nextest filter, skip unrelated init, cache fixtures      |
| Asserts only that it did not panic | Assert the exact symptom — wrong value, wrong row count, a diff     |
| Fails roughly 1 run in 5           | Pin the clock, seed the RNG, isolate the tempdir, block the network |
| Fails only in a full-workspace run | Confirm it is order or parallelism, then isolate the crate          |
| Fails only under load              | Add parallelism and stress; do not serialise                        |

## Test loop

```bash
# Narrowest possible signal: one test in one crate
RUST_BACKTRACE=1 cargo nextest run -p <crate> --test <file> -E 'test(<name>)'

# Catch test pollution that a single-test run hides
cargo nextest run -p <crate> --retries 0

# Is it parallelism or logic?
cargo nextest run -p <crate> --test-threads 1
```

`--retries 0` is the important one. Without it nextest hides flakes by retrying, so a green
run may have masked three failures.

## HTTP / service loop

```bash
# Start only what the bug needs, then issue one deterministic request
cargo run -p <bin> -- --port 8080 &
until curl -sf http://127.0.0.1:8080/health; do sleep 0.2; done
curl -sS -X POST http://127.0.0.1:8080/<route> \
  -H 'content-type: application/json' \
  -d @fixtures/<case>.json | jq -S . > /tmp/actual.json
diff -u fixtures/<case>.expected.json /tmp/actual.json
```

Snapshot the normalised response, not the raw bytes. Field order and timestamps are noise that
will mask the real signal.

## CLI fixture loop

```bash
cargo run -q -p <bin> -- <args> > /tmp/actual.txt 2>&1
diff -u fixtures/<case>.expected.txt /tmp/actual.txt
```

Record the exact invocation in `helpers/repro-template.md` so the loop is copy-paste ready.

## Trace replay

For bugs that only appear against a real upstream:

1. Capture the failing request, payload, or event log to a fixture file.
2. Extract the minimal field set that still triggers it — halve it repeatedly.
3. Feed the minimised fixture through the code path in isolation, with no network.

The minimised fixture is the regression test payload. Do not throw it away.

## Throwaway harness

Add a temporary example that calls the suspect path directly, with the real types but mocked
dependencies:

```rust
// examples/repro.rs — temporary, delete in Phase 6
fn main() {
    let ctx = test_context();
    let out = suspect::handle(&ctx, fixture());
    assert_eq!(out, expected());
}
```

Run it with `cargo run --example repro`. It compiles in seconds and isolates the bug from
everything else in the system.

## Fuzz loop

For intermittently wrong output:

```bash
cargo test -p <crate> --lib proptest
# or, if the repo has a fuzz target:
cargo +nightly fuzz run <target>
```

Start with a small case count and a fixed seed (`ProptestConfig { cases: 256, .. }`) so the
failing input is reproducible. Shrink before you fix.

## Bisection harness

When the bug appeared between two known states:

```bash
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  'cargo run -q -p <bin> -- <fixture-args> > /tmp/actual.txt 2>&1' \
  'diff -q /tmp/actual.txt fixtures/expected.txt' > /tmp/check.sh
chmod +x /tmp/check.sh
git bisect run /tmp/check.sh
```

The script must exit 0 for good, non-zero for bad, and must never require manual input.

## Differential loop

```bash
# Build both sides, then diff the same input
git worktree add /tmp/old HEAD~5
(cd /tmp/old && cargo build -q -p <bin> --release)
cargo build -q -p <bin> --release
/tmp/old/target/release/<bin> <args> > /tmp/old.txt 2>&1
./target/release/<bin> <args> > /tmp/new.txt 2>&1
diff -u /tmp/old.txt /tmp/new.txt
```

## Performance regression

Measure before touching anything.

```bash
# 1. Baseline on the known-good state
git stash
cargo bench -p <crate> -- --save-baseline before
git stash pop

# 2. Current state, compared against that baseline
cargo bench -p <crate> -- --baseline before

# 3. Bisect by hand with the bench as the check

# 4. Once localised, find where the time went
cargo flamegraph --bin <bin> -o /tmp/flame.svg
```

Without an explicit save and compare, criterion numbers are not comparable across states.

## Raising a reproduction rate

For non-deterministic bugs, do not wait for luck:

```bash
# Hammer it
for i in $(seq 1 100); do ./target/debug/<bin> <args> > /tmp/run.txt 2>&1 || echo "FAIL $i"; done

# Hammer it in parallel
seq 1 50 | xargs -P 8 -I{} ./target/debug/<bin> <args> > /dev/null 2>&1 || echo "FAIL {}"

# Shrink the timing window
RUST_LOG=trace ./target/debug/<bin> <args>
```

Record the observed rate in `helpers/repro-template.md` so you can tell whether your changes
actually raised it.
