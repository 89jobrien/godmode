# The --json flag

Read this before adding or changing a subcommand output or its --json support.

Every `godmode` subcommand accepts `--json` (global). Human-readable output goes to stdout by
default; `--json` emits machine-readable JSON for skill/agent consumers. Add `--json` support to
any new subcommand.
