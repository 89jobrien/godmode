# Documentation Types Reference

## README

**Purpose**: Entry point for anyone encountering the project for the first time.

**Required sections**:

- What it is (one paragraph)
- Install / quickstart
- Key concepts or mental model
- Common usage examples (real commands, real output)
- Links to deeper docs

**Avoid**: Implementation details, internal architecture, exhaustive flag lists (link to reference instead).

## CLAUDE.md

**Purpose**: Agent instructions — what Claude Code needs to work effectively in this repo.

**Required sections**:

- Build / test / lint commands
- High-level architecture (component ownership, not file lists)
- Conventions and constraints that aren't obvious from the code
- Anything that would surprise a developer new to the repo

**Avoid**: Generic best practices, repeating what's in README, invented state.

## Architecture Doc

**Purpose**: Capture component ownership, data flow, and key design decisions.

**Required sections**:

- Component diagram or ownership table
- Data flow narrative (how a request moves through the system)
- Key decisions and their rationale (especially non-obvious ones)
- Known tradeoffs and constraints

**Avoid**: Implementation minutiae, code snippets that will rot quickly.

## API Reference

**Purpose**: Every public export documented with signature, purpose, and example.

**Format per export**:

```
### `fn name(params) -> ReturnType`

One-sentence purpose.

**Parameters**: table of name / type / description
**Returns**: what it returns and when
**Errors**: error variants returned and when
**Example**: real usage snippet (not pseudocode)
```

**Avoid**: Restating the type signature in prose. Use examples, not descriptions.

## Skill / Agent Doc (SKILL.md)

**Required frontmatter**: `name`, `description`, `requires`, `next`
**Required sections**: When to Use, Process (numbered steps), Output Format
**Optional**: Rules, Guardrails, Handoff

**description** field: one sentence, include trigger phrases ("Triggers on X, Y, Z").
