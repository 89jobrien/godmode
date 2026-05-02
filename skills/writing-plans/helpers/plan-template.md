# Plan: <Feature Name>

## Goal

One sentence. What does this implement and why.

## Architecture

- Crates affected: `<crate-a>`, `<crate-b>`
- New traits/types: `TraitName` in `crates/<crate>/src/<module>.rs`
- Data flow: source → transform → sink

## Tech Stack

- Rust edition 2024
- Key crates: list any new deps and why

## Tasks

### Task 1: <name>

**Crate**: `<crate-name>`
**Run**: `cargo nextest run -p <crate>`

1. Write failing test:

   ```rust
   #[test]
   fn <test_name>() {
       // Arrange
       // Act
       // Assert — MUST FAIL before implementation
   }
   ```

   Run: `cargo nextest run -p <crate> -- <test_name>`
   Expected: FAIL

2. Implement:

   ```rust
   // exact code — no placeholders
   ```

3. Verify:

   ```
   cargo nextest run -p <crate>              → all green
   cargo clippy -p <crate> -- -D warnings   → zero warnings
   ```

4. Commit: `git commit -m "feat(<crate>): <summary>"`

### Task 2: <name>

**Crate**: `<crate-name>`
**Run**: `cargo nextest run -p <crate>`

<!-- repeat structure from Task 1 -->

## introspection Checklist

- [ ] Every requirement maps to at least one task
- [ ] No placeholders or vague directives anywhere
- [ ] Method names and types consistent across all tasks
- [ ] Each task is 2–5 minutes of focused work (split if longer)
- [ ] Each task ends with a commit
- [ ] `**Run**:` annotation present where useful
