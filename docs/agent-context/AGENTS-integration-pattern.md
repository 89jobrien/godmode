# Integration pattern

Read this before adding or changing an integration with an external tool.

All integrations (`doob`, `hj`, `rx`, `crux`) invoke external binaries via `std::process::Command`.
They fail gracefully — callers use `.ok()` or `ok().flatten()` so missing tools don't abort the
session. Never add a hard dependency on an external tool; always degrade gracefully.
