# Fuzz tests

Read this before adding a fuzz target for code that parses bytes, handles untrusted input, or contains `unsafe`.

### Fuzz — for unsafe, parsers, and protocol boundaries

If a function touches raw bytes, parses external input, handles untrusted data, or
contains `unsafe`, fuzz it. Property tests generate structured inputs; fuzz tests
generate arbitrary byte sequences. They find panics, integer overflows, and logic
errors that structured generation misses.

- Use `cargo-fuzz` (libFuzzer); targets live in `fuzz/fuzz_targets/`
- The `fuzz/` crate must be excluded from the workspace (`workspace.exclude = ["fuzz"]`)
- Seed corpus in `fuzz/corpus/<target>/` — commit representative inputs
- Requires nightly: `cargo +nightly fuzz run <target> -- -max_total_time=60`
- Good candidates: markdown/YAML/TOML parsers, template substitution, any function
  that deserialises external strings, any function that calls `unsafe`
- A fuzz target that finds no crash in 60s on first run is table stakes, not done —
  add it to CI with a short time budget (`-max_total_time=30`)
- Assert lightweight invariants inside fuzz targets, not just crash-freedom — e.g.
  verify summary counts equal task count after deserialising arbitrary YAML

```rust
// fuzz/fuzz_targets/fuzz_plan_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // plan::parse must never panic on arbitrary markdown input.
        let _ = godmode_core::plan::parse(s);
    }
});
```
