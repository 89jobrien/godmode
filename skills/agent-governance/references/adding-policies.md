# Adding a category or custom policy

Read this before adding a new agent category, or before creating a one-off policy for an agent that does not fit any category.

## Adding a New Category

1. Add the category field to your agent's `agents/cfg/<name>.cfg.yaml`
2. Copy `helpers/policy-template.yaml` to `policies/by-category/<category>.yaml`
3. Fill in allowed/blocked tools based on the agent's responsibilities
4. The hook picks it up automatically — no registration needed

## Adding a Custom Policy

For one-off agents that don't fit a category:

1. Create `policies/custom/<agent-name>.yaml`
2. Update `resolve-policy.nu` to check `custom/` before `by-category/`
3. The custom policy composes on top of default, overriding category

(Not yet implemented — use category policies for now.)
