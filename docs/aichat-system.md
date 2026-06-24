# Godmode AIChat System

This repository ships a repo-local AIChat assistant pack that can be installed into
`$HOME/.config/aichat/`.

## Assets

- `aichat/functions/agents.txt` registers `godmode-assistant`.
- `aichat/functions/agents/godmode-assistant/index.yaml` defines the agent instructions,
  variables, and conversation starters.
- `aichat/agents/godmode-assistant/config.yaml` stores per-agent runtime defaults.
- `aichat/roles/gm-planner.md`, `gm-reviewer.md`, `gm-debugger.md`, and `gm-verifier.md`
  provide focused AIChat roles for common godmode workflows.

AIChat 0.30.0 discovers agents from `functions/agents.txt`, loads agent definitions from
`functions/agents/<name>/index.yaml`, and loads per-agent config from
`agents/<name>/config.yaml`.

## Install

Install into the requested XDG-style target:

```nu
nu scripts/aichat-system-install.nu --target $"($env.HOME)/.config/aichat"
```

The installer:

- creates `config.yaml` when the target has none;
- merges `godmode-assistant` into `functions/agents.txt`;
- copies the agent definition, per-agent config, and support roles;
- refuses to overwrite existing differing files unless `--force` is passed;
- validates the agent with `AICHAT_CONFIG_DIR=<target> aichat --agent godmode-assistant --info`;
- warns when default `aichat --info` points at another config directory.

On macOS, AIChat may default to `/Users/joe/Library/Application Support/aichat/`. Use
`AICHAT_CONFIG_DIR=$HOME/.config/aichat` when you want AIChat to read this installed pack.

To validate without mutating the requested target:

```nu
nu scripts/aichat-system-install.nu --target $"($env.HOME)/.config/aichat" --dry-run
```

To intentionally replace existing installed agent files:

```nu
nu scripts/aichat-system-install.nu --target $"($env.HOME)/.config/aichat" --force
```

## Validate Without A Live Model Call

```nu
with-env { AICHAT_CONFIG_DIR: $"($env.HOME)/.config/aichat" } {
  aichat --agent godmode-assistant --info
}
```

This validates the agent definition and config without spending model tokens.

## Smoke Test Prompt Rendering

```nu
with-env { AICHAT_CONFIG_DIR: $"($env.HOME)/.config/aichat" } {
  aichat --agent godmode-assistant --dry-run "Create a task graph for a Rust feature"
}
```

`--dry-run` still requires the target config to have a valid model/client setup. If the target
only has the minimal config created by the installer, use `--info` until client credentials are
configured there.

## Godmode Template

Create a task graph for installing and validating the system:

```nu
godmode task apply AICHAT-SYSTEM --var target=$"($env.HOME)/.config/aichat"
```

The template depends on `.template.yaml` resolution support in `godmode-core`.
