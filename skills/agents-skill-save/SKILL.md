---
name: agents-skill-save
description: Use when creating a new local skill, or when an existing skill was saved to the wrong skills tree. Explains which of ~/.claude/skills, ~/.agents/skills, ~/dev/godmode/skills, and ~/.notfiles/claude/.claude/skills is canonical, and which are projections of it.
---

# Agents Skill Save

## Which tree is which

There is no single "active" tree. Both `~/.claude/skills` and `~/.agents/skills`
are on the harness search path, and `~/.claude/skills` is a **projection** of the
canonical sources below — never a source itself:

| Path                                        | What it actually is                                                                                                                                                                           |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `~/dev/godmode/skills/<name>/`              | **Canonical for godmode skills.** Git repo. Edit here.                                                                                                                                        |
| `~/.notfiles/claude/.claude/skills/<name>/` | **Canonical for personal/devloop skills** (using-conductor, devloop-analyze, herald-sync, devloop-standup, ...). Git-tracked in the notfiles dotfiles repo.                                   |
| `~/.agents/skills/<name>`                   | Untracked. Partly a **drifting snapshot** (frequently stale), partly the live home of a few skills that `~/.claude/skills` symlinks into (depgraph, using-doob, think-consistency, ...).      |
| `~/.claude/skills/<name>`                   | **Projection only.** Godmode skills appear as dir symlinks into `~/dev/godmode/skills`; notfiles skills as a real dir whose `SKILL.md` is a symlink into `~/.notfiles/claude/.claude/skills`. |

Verify before assuming:

```bash
ls -la ~/.claude/skills | head -30      # symlink targets reveal the real source
git -C ~/dev/godmode status --porcelain # godmode tree is version controlled
git -C ~/.notfiles status --porcelain   # notfiles tree is version controlled
```

## How to save a skill

**New skill that belongs to godmode** — write it into the canonical tree and link it:

```bash
mkdir -p ~/dev/godmode/skills/<skill-name>
# write ~/dev/godmode/skills/<skill-name>/SKILL.md
ln -s ~/dev/godmode/skills/<skill-name> ~/.claude/skills/<skill-name>
git -C ~/dev/godmode add skills/<skill-name> && git -C ~/dev/godmode commit
```

**New personal skill for the devloop/dotfiles workflows** — write it into the notfiles
package and let notfiles link it:

```bash
mkdir -p ~/.notfiles/claude/.claude/skills/<skill-name>
# write ~/.notfiles/claude/.claude/skills/<skill-name>/SKILL.md
notfiles --dry-run link claude   # preview: should offer to link the new SKILL.md
notfiles link claude
git -C ~/.notfiles add claude/.claude/skills/<skill-name> && git -C ~/.notfiles commit
```

**Personal skill with no git home** — `~/.agents/skills/<name>/SKILL.md`
is acceptable, but understand you own keeping it in sync manually; nothing
regenerates it.

## Never do this

- **Do not `mv` a skill out of a git-tracked tree (`~/dev/godmode`, `~/.notfiles`)
  into `~/.agents/skills`.** That pulls content out of version control into an
  untracked, drifting copy. It silently loses version history and reintroduces drift.
  Move the content _into_ the canonical tree instead, then remove the `.agents` copy.
- **Do not edit through a `~/.claude/skills` symlink and assume you edited a
  separate file.** You edited the canonical file it points at. That is
  usually what you want, but confirm with `git -C ~/dev/godmode status` or
  `git -C ~/.notfiles status`.

## Keeping the trees consistent

Both trees being loaded means a skill present in both is loaded twice, and
whichever wins depends on search order. When adding a skill, check for an
existing same-named skill everywhere first:

```bash
# Glob tool (path: $HOME): glob '{.agents,.claude,.notfiles/claude/.claude,dev/godmode}/skills/<skill-name>/SKILL.md'
```

If the same skill exists in more than one canonical tree (this has happened —
herald-sync drifted between `~/.agents` and notfiles), pick one canonical home,
copy the newest content there, link the projection, and remove the other copy —
do not leave both and hope.

## Path references inside skill docs

Prefer repo-relative references (`references/foo.md`, `scripts/bar.sh`) over
absolute `~/...` paths. Absolute paths break as soon as a skill moves between
trees. When a skill genuinely must point at a shared location, point at the
canonical source (`~/dev/godmode/skills/...` or `~/.notfiles/claude/.claude/skills/...`),
not at a projection of it.
