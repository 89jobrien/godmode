---
name: "godmode:release-notes"
description: >
  Write user-facing release notes from git history. Use when preparing
  announcements, GitHub Releases, or user documentation. Differs from
  changelog — focuses on user impact, not implementation or crate structure.
requires: []
next: []
---

# Release Notes

## When to Use

- Before publishing a GitHub Release
- When writing an announcement for end users
- To create a companion to the developer-facing changelog
- When asked to "write release notes", "announce release", or "GitHub release"

## Key Difference from Changelog

| Aspect           | Changelog               | Release Notes                |
| ---------------- | ----------------------- | ---------------------------- |
| **Audience**     | Developers, maintainers | End users                    |
| **Grouping**     | By crate, then type     | By user-visible feature area |
| **Language**     | Technical               | Plain, user-centric          |
| **Detail level** | Commit granularity      | User impact only             |
| **Scope**        | All changes             | User-visible changes only    |

**Changelog**: "refactor(dispatch): use Arc for thread-safe graph mutations"

**Release Notes**: "Task execution is now thread-safe, allowing larger
projects to run without race conditions."

## Feature Area Grouping

Group changes by how users perceive them, not by implementation structure.

**User-visible areas**:

- Command-line interface (new flags, new commands)
- Performance (faster builds, lower memory use)
- Error messages (clearer diagnostics)
- Documentation (new guides, updated reference)
- Compatibility (support for new platforms, runtimes)

**Skip entirely**:

- Internal refactoring with no user impact
- Dependency updates (unless fixing a user-facing bug)
- Test improvements
- Build system changes
- Internal module restructuring

## Content Guidelines

### What to include

1. **New user-facing features**: "You can now use `--parallel N` to run
   multiple tasks concurrently"
2. **User-visible improvements**: "Error messages are now more concise and
   actionable"
3. **Critical bug fixes**: "Fixed crash when using paths with spaces"
4. **Performance gains**: "Builds are 30% faster on macOS"
5. **Breaking changes**: Always describe migration path

### What to exclude

- Internal refactoring (no user-visible change)
- Crate or module names (unless the crate is a user-facing tool)
- Implementation details
- Internal test improvements
- Dependency version bumps (mention only if it fixes a user issue)

## Prose Style

- **Verb tense**: Present passive ("Tasks are now cached") or simple present
  ("The CLI now supports...")
- **Audience level**: Plain English, no jargon. Define terms if first mention.
- **Length**: One to two sentences per feature
- **Examples**: Include concrete examples for complex features

**Example**:

- Good: "You can now filter tasks by priority using `godmode task list
--priority high`, making it easier to focus on critical work."
- Avoid: "Implemented priority filtering in task list subcommand with O(n)
  complexity."

## Breaking Changes

Always include a "Breaking Changes" section if applicable. For each breaking
change:

1. State what changed (what users must update)
2. Explain why it changed
3. Provide migration steps (concrete, copy-paste if possible)

**Example**:

```markdown
## Breaking Changes

### `godmode task add` flag rename

The `--depends` flag has been renamed to `--depends-on` for consistency.

**Migration**: Update scripts that use `godmode task add` from
`--depends t1,t2` to `--depends-on t1,t2`.
```

## Output Format

Produce markdown with these sections (omit if empty):

```markdown
# Release v<version>

## What's New

(brand-new user-facing features)

## Improvements

(enhancements to existing features, performance, documentation)

## Bug Fixes

(critical bugs, correctness issues)

## Breaking Changes

(specify migration path for each)

## Deprecations

(features that still work but will be removed)

## Contributors

(if applicable)
```

## Integration with Release Workflow

Write final notes to `.ctx/` or `docs/` as appropriate, then commit.
When bumping versions, pass the notes manually to your release script or commit
alongside the version bump (`godmode release bump` / `tag` / `push` handle the version side).

## Guardrails

- Never describe internal refactors as user features
- Do not mention crate names unless they are user-facing tools
- Always verify feature claims in `git log` before writing
- Distinguish breaking from non-breaking changes
- Do not commit release notes without explicit instruction
- Never fabricate changes not present in git history
- If a change is ambiguous, err on the side of exclusion — release notes
  should only cover clear, obvious user impact
