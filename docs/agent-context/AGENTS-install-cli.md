# Install the CLI

Read this before rebuilding or installing the godmode binary.

```bash
taskit dev build --release && cp target/release/godmode ~/.cargo/bin/godmode
```

Note: `which godmode` resolves to `~/.cargo/bin/`, not `~/.local/bin/`. Always copy to
`~/.cargo/bin/` when rebuilding.
