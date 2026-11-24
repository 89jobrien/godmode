# OpenCode gates

Read this before committing a change to .opencode/ or the hook registry.

`taskit check ci` only dispatches Rust built-ins, so the OpenCode projection is gated in
GitHub Actions (`conformance-opencode` job) rather than taskit. Run both locally before
committing a change to `.opencode/` or `crates/godmode-core/src/hooks/registry.rs`:

```bash
bun install --cwd .opencode                # audit reads the installed type definitions
bun run scripts/opencode/audit-plugin.ts   # contract audit
bun x tsc -p .opencode/tsconfig.json       # typecheck
```

`audit-plugin.ts` derives the hook contract by parsing the installed
`@opencode-ai/plugin/dist/index.d.ts` rather than hardcoding hook names, so a hook OpenCode
never declared is caught. It is not redundant with `tsc`: TypeScript does **not** enforce
handler arity, and does **not** flag an unknown property when a valid key is also present.
A 1-arg handler satisfying a 2-arity hook, or a typo'd hook name beside a real one, both
compile clean.

`.opencode/package.json` and `bun.lock` are tracked on purpose. The audit reads the pinned
`@opencode-ai/plugin` version, so leaving them gitignored would make the gate
unreproducible and unable to run in CI.
