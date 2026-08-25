# Plan: OpenCode Project Agents

## Goal

Install a global OpenCode router and repo specialists backed by safe personal-mcp project tools.

## Context Map

### Files to Modify

| File                                       | Purpose          | Changes Needed                                |
| ------------------------------------------ | ---------------- | --------------------------------------------- |
| `personal-mcp/src/project_registry.rs`     | Registry domain  | Parse, describe, and execute safe actions     |
| `personal-mcp/src/main.rs`                 | MCP router       | Register project tools                        |
| `personal-mcp/config/project-tools.toml`   | Command policy   | Declare all indexed projects and safe actions |
| `godmode/crates/godmode-core/src/agent.rs` | Agent generation | Render and install OpenCode Markdown agents   |
| `godmode/crates/godmode-cli/src/main.rs`   | CLI              | Add `agent install-opencode`                  |
| `godmode/agents/opencode-projects.yaml`    | Agent catalog    | Declare router and 23 project specialists     |

### Dependencies

| File                                     | Relationship                             |
| ---------------------------------------- | ---------------------------------------- |
| `personal-mcp/Cargo.toml`                | Adds subprocess timeout support          |
| `godmode/crates/godmode-core/src/lib.rs` | Existing `agent` module already exported |
| `$HOME/.config/opencode/agents/*.md`     | Generated installation output            |

### Test Coverage

- Inline personal-mcp tests cover catalog parsing, unknown names, path containment, timeout, and
  successful execution.
- Inline godmode-core tests cover catalog parsing, router rendering, hidden specialists, and
  dry-run installation.
- Existing workspace tests cover CLI parsing and agent generation regressions.

## Architecture

- Crates affected: `personal-mcp`, `godmode-core`, `godmode-cli`.
- Data flow: YAML/TOML catalogs -> validated domain types -> Markdown agents or subprocess output.
- The MCP registry exposes only exact read-only command vectors and accepts no free-form arguments.

## Tasks

### Task 1: Add the project registry domain

**Crate**: `personal-mcp`
**Files**: `src/project_registry.rs`, `config/project-tools.toml`, `Cargo.toml`
**Run**: `cargo nextest run project_registry`

Write failing registry tests, implement catalog parsing and guarded execution, then run clippy.

### Task 2: Register project MCP tools

**Crate**: `personal-mcp`
**Files**: `src/main.rs`, `AGENTS.md`
**Run**: `cargo nextest run`

Register list, describe, and run handlers and document their safety contract.

### Task 3: Add OpenCode catalog rendering

**Crate**: `godmode-core`
**Files**: `crates/godmode-core/src/agent.rs`, `agents/opencode-projects.yaml`
**Run**: `cargo nextest run -p godmode-core agent::tests`

Write failing rendering tests, parse the catalog, and render router and specialist Markdown.

### Task 4: Add installation CLI

**Crate**: `godmode-cli`
**Files**: `crates/godmode-cli/src/main.rs`
**Run**: `cargo nextest run -p godmode-cli`

Add dry-run and output-directory options, then install agents into the global OpenCode directory.

### Task 5: Verify integration

Generate the agents, inspect the installed files, validate OpenCode configuration, and run format,
clippy, and test gates in both repositories.
