# Design: OpenCode Project Agents

## Goal

Create global OpenCode agents that use Joe's projects as governed tools while retaining an
explicit path for source maintenance.

## Approved Approach

Combine a global router with repo specialists and a phased MCP surface: a safe registry first,
then dedicated typed tools for frequently used operations.

## Crate Ownership

- **godmode-core**: owns the OpenCode agent catalog, rendering, and installation logic.
- **godmode-cli**: exposes the `godmode agent install-opencode` command.
- **personal-mcp**: owns project discovery and execution of exact allowlisted read-only actions.

## Public API

### Godmode Types

```rust
pub struct OpenCodeAgentCatalog {
    pub router: OpenCodeRouterDef,
    pub projects: Vec<OpenCodeProjectAgentDef>,
}

pub struct OpenCodeRouterDef {
    pub name: String,
    pub description: String,
}

pub struct OpenCodeProjectAgentDef {
    pub name: String,
    pub description: String,
    pub project: String,
    pub repo_path: String,
    pub visible: bool,
}

pub fn load_opencode_catalog(path: Option<&Path>) -> Result<OpenCodeAgentCatalog>;
pub fn render_opencode_agents(catalog: &OpenCodeAgentCatalog) -> Vec<(String, String)>;
pub fn install_opencode_agents(
    catalog: &OpenCodeAgentCatalog,
    output_dir: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>>;
```

### Personal MCP Types

```rust
pub struct ProjectDescribeRequest {
    pub project: String,
}

pub struct ProjectRunRequest {
    pub project: String,
    pub action: String,
}

pub fn project_list() -> Result<String, ErrorData>;
pub fn project_describe(req: ProjectDescribeRequest) -> Result<String, ErrorData>;
pub fn project_run(req: ProjectRunRequest) -> Result<String, ErrorData>;
```

## Data Flow

1. A user asks the global `workspace` primary agent for project-specific work.
2. The router delegates to the matching `workspace-*` specialist.
3. The specialist discovers safe actions through `personal_project_describe`.
4. Read-only operations run through `personal_project_run` after registry validation.
5. Explicit source-maintenance requests use OpenCode file and Bash tools after reading the repo's
   `AGENTS.md`.

## Hexagonal Boundaries

- The OpenCode catalog is declarative input rendered by godmode-core.
- The personal-mcp project registry is the policy boundary around subprocess execution.
- Exact command vectors, workspace-root containment, timeouts, and output limits prevent the
  registry from becoming an unrestricted shell.

## Out of Scope

- Arbitrary command execution through the project registry.
- Mutating or destructive registry actions.
- Replacing project-local OPAVS agents.
- Dedicated typed tools for every project operation in the first phase.

## Risk

- [ ] Breaking API changes: no; all APIs and commands are additive.
- [x] New external dependency: `wait-timeout` in personal-mcp for subprocess deadlines.
- [ ] Feature flag required: no.
- [x] Generic execution risk: mitigated by an embedded reviewed allowlist and no caller arguments.
