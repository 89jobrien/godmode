---
version: 1
last_updated: 2026-09-12
next_review: 2026-09-19
---

# Recurring Mistakes Ledger

Portable evidence uses commits or staged/force-added reports. A finite count labeled **Exact** is used only when that evidence enumerates the complete set. Inherited observations without portable row-level evidence are labeled **Estimated** or **Estimated minimum**; they may use `+`, sampled dates, or phrases such as "and others" and must not be read as exact totals.

---

## Process Errors

### Skill index drift — new skills added without updating skill-index.json

- **Occurrences**: Estimated minimum: 12+
- **Sampled dates**: 2026-05-02, 2026-05-04, 2026-05-20, 2026-06-02, 2026-06-14, 2026-06-24,
  2026-06-27, and earlier observations
- **Affected**: `skills/using-godmode/references/skill-index.json`, `agents/INDEX.md`
- **Sample commits**: `996c84d` (add 14 missing skills), `2314b8c` (add 7 missing),
  `a4eca10` (add missing skills + CLI subcommands), `d6708d5` (CI gate + fix 4 entries)
- **Prevention**: CI runs `cargo xtask ci`, which invokes
  `godmode skill index --check`. The gate fails if a skill dir exists but has no entry
  in skill-index.json.
- **Status**: ✅ Prevention in place (CI gate). Monitor for regressions.
- **Notes**: Most common single mistake in the project. Skills are easy to add but the
  index is a separate manual step that was consistently forgotten until the CI gate.

---

### Stale references in skills/agents — CLI subcommands, helper paths, tool names

- **Occurrences**: Estimated minimum: 9
- **Dates**: 2026-05-02, 2026-05-04, 2026-06-02, 2026-06-14, 2026-06-24, 2026-06-27
- **Affected**: Various SKILL.md files, agents/*.md, introspection audit output
- **Sample commits**: `0ce0b6e` (branch guards, index sync, stale refs),
  `b2a03f8` (remove nonexistent `godmode report` subcommand, fix stale tdd-agent ref),
  `74e679d` (replace cat/grep with Read/Grep tools, fix stale tdd-agent ref),
  `8475906` (remove stale helpers/fuzz/ reference from decompose)
- **Prevention**: Run `/godmode:introspection` before any release; add CLI subcommand
  list to a machine-readable registry that skills can reference rather than inline strings.
- **Status**: ⚠️ Partially mitigated (introspection skill exists, CI conformance suite).
  Stale references still surface on nearly every introspection run.
- **Notes**: Pattern: a skill is written referencing a CLI subcommand or helper path that
  either never existed or was removed. Introspection catches these but they accumulate
  between introspection runs.

---

### Uncommitted session work — changes staged or modified but not committed at session end

- **Occurrences**: Estimated minimum: 5
- **Sampled dates**: 2026-06-08, 2026-06-03, 2026-05-13, 2026-06-28, and other observations
- **Affected**: Various files across sessions
- **Evidence**: Inherited ledger observations record one session with 7 uncommitted files and another with 10+ untracked or modified files. These values are retained as estimates, not exact totals.
- **Prevention**: Run `/gm-cap` or `/gm-handoff` as last action of every session.
  The `Stop` hook runs `hj handoff` automatically, but does not commit.
- **Status**: ⚠️ Structural — session end workflow doesn't enforce commit. Add
  "stage and commit session work" as explicit last step in lifecycle pipeline.
- **Notes**: The untracked items are often from prior sessions that accumulated. Each
  session that doesn't commit its output makes the next session's hygiene harder.

---

### Stash pop conflict from TODO annotations committed after worktree creation

- **Occurrences**: Estimated minimum: 3 (1 inherited prior conflict; at least 2 preservation/reapplication failures in the closeout session)
- **Dates**: 2026-06-28; 2026-09-09; 2026-09-12
- **Affected**: `crates/godmode-core/src/release.rs`; MoA integration branches;
  restored dirty source state including `crates/godmode-core/src/agent/opencode.rs`
- **Evidence**: The inherited ledger records one prior `release.rs` stash-pop conflict. The portable closeout reflection records the later branch-movement/merge collision, repeated conflicts, and formatting rerun after local restoration; the resolution audit isolates the fixed baseline from preserved local state. [Reflection: `.ctx/godmode/reports/reflect/reflect-2026-09-12.md:21-31`; Audit: `.ctx/godmode/review/99-resolution.md:1-6`]
- **Prevention**: Freeze and record the integration base, check for branch movement before
  every merge, run all fixed-baseline gates before restoring local state, restore local state
  only as the final step, and immediately rerun formatting after source restoration. Commit
  task-owned changes before worktree dispatch when possible rather than carrying them in a stash.
- **Status**: ⚠️ Recurring process friction — preservation is necessary, but restoration
  timing and moving integration bases remain uncontrolled.
- **Notes**: The ledger already contained the uncommitted-work/stash-conflict pattern; these
  two observed failures increment that pattern rather than creating a duplicate.

---

### Fix-agent completion overclaims before independent row audit

- **Occurrences**: Exact: 2
- **Dates**: 2026-09-12 (two successive remediation passes)
- **Affected**: MoA remediation of the 58 rows synthesized in
  `.ctx/godmode/review/00-synthesis.md`
- **Evidence**: Fix agents twice reported all assigned findings resolved, but independent
  row audits found 13 PARTIAL rows after the initial integration (45/58 resolved) and then
  2 PARTIAL rows after the follow-up integration (56/58 resolved). Git history records the
  initial integration through `6226044`, follow-up fixes/integration through `1321d24`, and
  third-pass fixes `adfc159` and `8e6e865` merged through `b50f753`. The final
  `.ctx/godmode/review/99-resolution.md` independently records 58 RESOLVED, 0 PARTIAL, and
  0 BLOCKED.
- **Prevention**: Require a row-by-row independent audit against every synthesis finding
  before declaring integration complete; do not treat fix-agent assignment summaries or
  successful merges as resolution evidence. Explicitly reassign every PARTIAL row and repeat
  the independent audit before integration completion.
- **Status**: ⚠️ Recurring within the remediation cycle — closed on the third pass at 58/58.
- **Notes**: These are two overclaim occurrences, not 15 finding occurrences; 13 and 2 are
  the exact partial-row counts exposed by the two audits.

---

## Nu/Shell Syntax Errors

### Nu `2>&1` vs `out+err>` confusion — POSIX redirect used in Nu context

- **Occurrences**: Estimated: 4 (session observations; exact total unclear)
- **Dates**: 2026-06-28 (twice in one session), prior sessions
- **Affected**: Any Bash tool call piping combined stdout+stderr
- **Evidence**: Inherited ledger observation: the redirect was attempted twice in one session and blocked by the pre-tool hook. The total remains an estimate.
- **Prevention**: Mental shortlist: always use `out+err>|` for combined redirect in Nu.
  The coursers pre-hook blocks `2>&1` with a clear error, but the fix still requires
  a retry. Add a personal snippet or alias.
- **Status**: ⚠️ Known but unautomated. Hooks catch it, but the cognitive overhead
  remains.

---

### Nu backtick/parenthesis quoting in `gh issue create` — parse errors on complex strings

- **Occurrences**: Estimated: 2 observed in one inherited session
- **Dates**: 2026-06-28
- **Affected**: `gh issue create --body "..."` calls with file paths and Rust type sigs
- **Evidence**: Inherited ledger observation: backtick-heavy issue bodies caused repeated parse errors and required a body file. The count is not treated as an exact project total.
- **Prevention**: Generate a temporary path, for example
  `let issue_body = ($env.TMPDIR | path join $"issue-body-(random uuid).txt")`, then use
  `save --force $issue_body` + `--body-file $issue_body` for multi-line/complex gh issue
  bodies in Nu. Never inline the body string directly.
- **Status**: ⚠️ Pattern identified. Document in session preflight mental model.

---

### Nu deprecated `get -i` → `get -o` — scripts silently broke across Nu version

- **Occurrences**: Exact: 1 batch fix
- **Dates**: 2026-05-31
- **Affected**: Multiple Nu hook scripts
- **Evidence**: commit `9c8b427` "replace deprecated `get -i` with `get -o` in remaining scripts"
- **Prevention**: After Nu version upgrades, run `nu --check` on all hook scripts. Track
  Nu changelog for deprecation notices in `CLAUDE.md` notes.
- **Status**: ✅ Resolved for this deprecation. General risk remains on future Nu upgrades.

---

### Nu dynamic `use` at parse time — fails nu-check for scripts using git rev-parse

- **Occurrences**: Estimated: 1 documented inherited observation
- **Dates**: 2026-06-03
- **Affected**: Nu scripts using `use (git rev-parse ...)` style dynamic imports
- **Evidence**: Inherited ledger observation: dynamic parse-time `use` paths failed `nu-check` and motivated migration to Rust.
- **Prevention**: Do not use dynamic paths in `use` statements in Nu. Use static
  absolute paths or migrate the logic to Rust.
- **Status**: ✅ Motivates nu-to-Rust migration (completed 2026-05-31 for hook scripts).

---

## Test Failures / Flakes

### `hj_integration::handoff_passes_args` — intermittent failure in full suite

- **Occurrences**: Estimated minimum: 2 observed; additional occurrences may exist
- **Dates**: 2026-06-03; prior observations
- **Affected**: `godmode-core` integration tests
- **Evidence**: Inherited ledger observation: the test passed in isolation and failed in a full suite. The count is an estimated minimum.
- **Prevention**: Investigate test isolation — likely a timing or environment-variable
  leak. Consider adding `#[serial]` or a test mutex guard.
- **Status**: ⚠️ Pre-existing, unresolved. Track occurrences.

---

## Dependency / API Breakage

### crux-runtime breaking change undocumented — Step struct fields added without CHANGELOG

- **Occurrences**: Estimated: 1 inherited observation
- **Dates**: 2026-06-03
- **Affected**: `crates/godmode-core/src/session_trace.rs`, `crates/godmode-core/src/integrations/crux.rs`
- **Evidence**: Inherited ledger observation: added `Step` fields (`findings` and `metadata`) were discovered through compile errors after new tests were added.
- **Prevention**: When bumping `crux-runtime` dependency, always read its CHANGELOG or
  run `cargo check --workspace` before any other work in the session.
- **Status**: ✅ Resolved (fields added). Prevention: add `cargo check` to session
  pre-flight.
- **Notes**: The fix was mechanical once found, but took time to diagnose since the
  error appeared in test files, not in the code that changed.
