# Code Review Checklist

## Correctness

- [ ] Logic matches the stated requirement
- [ ] Edge cases handled: empty input, `None`, zero, overflow, empty iterator
- [ ] Error paths return `Result<_, E>` with meaningful variants — no `.unwrap()` outside tests
- [ ] No silent data loss: lossy `as` casts, truncating conversions, silent `ok()` drops
- [ ] `?` propagation doesn't swallow context — errors carry enough info to diagnose

## Safety & Security

- [ ] No `unsafe` block without a `// SAFETY:` comment justifying the invariant
- [ ] No string interpolation into shell commands — use `Command::arg()` not `format!()`
- [ ] Secrets not in `Debug` output, log lines, or serialised structs
- [ ] No `std::env::set_var` outside `unsafe {}` (Rust 2024 edition)
- [ ] File paths from external input validated before use

## Architecture

- [ ] Change is in the right layer — I/O not mixed into pure logic
- [ ] New external dependency sits behind a trait (port); implementation in an adapter
- [ ] No new circular dependencies between crates
- [ ] `pub` surface is intentional — no accidental leakage via `pub use` or missing `pub(crate)`
- [ ] Hexagonal boundary respected: domain crates have zero imports from infrastructure crates
- [ ] Ports (`trait`) defined in domain; adapters (`impl`) in infrastructure — never reversed
- [ ] Trait objects (`Box<dyn Trait>`) only where `impl Trait` won't work (object safety, erasure)

## Types & Idioms

- [ ] Prefer `impl Trait` over `Box<dyn Trait>` where lifetimes allow
- [ ] `#[must_use]` on functions returning `Result` or values callers must act on
- [ ] `Clone`/`Copy` derives are intentional — large types shouldn't silently impl `Copy`
- [ ] Newtype wrappers used where primitives carry domain meaning (e.g. `UserId(u64)`)
- [ ] Iterator chains preferred over manual index loops
- [ ] `match` is exhaustive — no `_ => unreachable!()` hiding unhandled variants
- [ ] Lifetime annotations minimal and correct — no `'static` used to paper over a borrow issue

## Tests

See **`godmode:testing-philosophy`** for the full five-dimension model and when to apply each.
Quick reference: `Unit → Property → Conformance → Integration → Regression`

- [ ] Every new public function has at least one unit test
- [ ] Happy path and at least one error/edge case covered
- [ ] Unit tests use in-memory fakes, not mock frameworks
- [ ] No `unwrap()` in tests — use `expect("reason")`
- [ ] No test-only methods on production types
- [ ] Integration test present if the change crosses a crate or module boundary
- [ ] Property test present if the function has a non-trivial input space
- [ ] `prop_compose!` used for domain type strategies — not raw primitives
- [ ] `proptest-regressions/` committed and not gitignored
- [ ] Conformance test present for every new `impl Trait`
- [ ] Regression test present for every bug fix

## Style

- [ ] Names consistent with Rust conventions: `snake_case` fns, `CamelCase` types, `SCREAMING_SNAKE` consts
- [ ] No dead code, unused imports, `#[allow(dead_code)]` without explanation
- [ ] Doc comments (`///`) on public items where behaviour is non-obvious
- [ ] Line width ≤ 100 columns
- [ ] `cargo clippy -- -D warnings` clean

## Severity Guide

| Finding                                         | Severity   |
| ----------------------------------------------- | ---------- |
| Logic error, data loss, panic risk              | Blocking   |
| `unsafe` without `// SAFETY:` comment           | Blocking   |
| Missing test for new public function            | Blocking   |
| New public function with no unit test           | Blocking   |
| Crate boundary crossed with no integration test | Suggestion |
| Non-trivial input space with no property test   | Suggestion |
| Bug fix with no regression test                 | Suggestion |
| New trait impl with no conformance test         | Suggestion |
| Accidental `pub` leakage                        | Blocking   |
| Suboptimal idiom (`Box<dyn>` vs `impl Trait`)   | Suggestion |
| Missing `#[must_use]`, missing newtype          | Suggestion |
| Unclear name, minor duplication                 | Suggestion |
| Formatting, minor style inconsistency           | Nitpick    |
| False positive on test fixture                  | Allowlist  |
