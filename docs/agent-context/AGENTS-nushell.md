# Nushell

Read this before writing or editing hook scripts.

Hook scripts in this repo are Nushell. Key syntax rules:

- `const` cannot reference `$env` — use `let` or read at runtime with `$env.VAR`
- `&&` is not valid — use `;` to chain commands
- `open --raw /dev/stdin | from json` to read stdin in hook scripts
- `do { ... } | complete` captures stdout + exit code for fallible commands
- Never use bash-isms: no `$()`, no `export VAR=val`, no `if [ ... ]`
- Test syntax with `nu -c '<snippet>'` before writing to a file
