# Doc Sync — Drift Detection Checklist

## CLI Surface

- [ ] `<binary> --help` — list all subcommands; compare against documented subcommand table
- [ ] `<binary> <cmd> --help` for each subcommand — list all flags; compare against documented flags
- [ ] Documented flags that no longer appear in `--help` → **Blocking**
- [ ] Flags in `--help` not documented → **Suggestion**

## Crate / Module Surface

- [ ] `ls crates/` → compare against crate list in README or CLAUDE.md
- [ ] `grep -r '^pub mod' src/lib.rs` → compare against module table
- [ ] New crates not in docs → **Suggestion**
- [ ] Docs mention removed crates → **Blocking**

## Skills

- [ ] `ls skills/*/SKILL.md` → compare against skill table in README and CLAUDE.md
- [ ] Skill `name:` frontmatter matches table entry
- [ ] New skills not in tables → **Suggestion**
- [ ] Table references skills that no longer exist → **Blocking**

## Agents

- [ ] `ls agents/*.md` → compare against `agents/INDEX.md`
- [ ] Agent `name:` frontmatter matches INDEX entry
- [ ] Run `nu scripts/gen-index.nu --dry-run` to detect INDEX drift

## File Paths

- [ ] Extract all literal paths from docs (grep for `/`, `./`, `~/`)
- [ ] For each path: verify it exists on disk
- [ ] Missing paths → **Blocking**

## Cross-Doc Consistency

- [ ] Same feature described in README and CLAUDE.md → descriptions must agree
- [ ] Skill description in SKILL.md frontmatter matches README skills table
- [ ] Agent description in agent `.md` matches INDEX.md entry
- [ ] Version numbers consistent across all docs

## Severity Quick Reference

| Severity   | Examples                                             |
| ---------- | ---------------------------------------------------- |
| Blocking   | Removed flag documented, path does not exist         |
| Suggestion | New feature undocumented, description mismatch       |
| Nitpick    | Wording inconsistency, stale version number in prose |
