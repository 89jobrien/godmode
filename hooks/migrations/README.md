# Hook Migrations

Numbered migration scripts run by `godmode hook migrate`. Each script is idempotent.

Scripts are named `NNN-description.nu` and run in sorted order. A migration should
check whether its target state already exists before making changes.
