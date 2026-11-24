# CLI subcommands

Read this before looking up or changing a godmode subcommand.

```text
godmode handon                                  # session-start triage summary
godmode handoff                                 # session-end validation
godmode context [--json]                        # full session context for hooks/agents
godmode status [--compact]                      # graph counts + next runnable tasks
godmode task list [--priority high|normal|low]
godmode task next [--priority high|normal|low]
godmode task add <title> [--id t5] [--depends-on t1,t2] [--crate-name X]
godmode task start <id>
godmode task done <id> [--commit <sha>] [--notes <text>]
godmode task block <id> <reason>
godmode task unblock <id>
godmode task unblock-all                        # reset all blocked tasks to pending
godmode task remove <id>
godmode task clear --done
godmode task clear --all
godmode task run <id> [--auto-done]             # execute task's run: field via rx
godmode task pull [--project <name>]            # import pending doob todos
godmode task pull --github [--repo owner/repo] [--label <label>]
godmode task apply <name> [--var k=v]           # expand a template into the task graph
godmode task list-templates                     # list available templates (local + global)
godmode task push-done                          # sync completed tasks back to doob
godmode plan ingest <path>                      # parse plan markdown into task graph
godmode dispatch [--max N] [--critical-path]    # emit parallel chains JSON
godmode visualize-graph [--format dot|svg] [--out <path>]  # render task graph
godmode pin [<path>]                            # pin session to a repo root
godmode unpin                                   # remove the pinned root
godmode init                                    # first-time setup: global config + state dirs
godmode doctor                                  # validate environment: tools, 1Password, worktrees
godmode memory-banking inject
godmode memory-banking remind
godmode memory-banking init
godmode memory-banking status
godmode agent list [--filter <kw>]              # list installed agents
godmode agent index                             # regenerate agents/INDEX.md
godmode agent dispatch <path> [--max N]         # plan ingest + dispatch in one shot
godmode agent generate [<name>] [--all]         # generate .md from agent YAML
godmode agent migrate [<name>] [--all]          # migrate agent .md frontmatter to YAML stubs
godmode graph build [--input <tmpl>] [--var k=v]
godmode verify [--crate-name X]                 # nextest + clippy + fmt + commits
godmode wave init --wave N --agents a,b,c
godmode wave status
godmode wave done <agent> [--commits <commits>]
godmode wave block <agent> <reason>
godmode wave check
godmode worktree add <branch> [--issue N]
godmode worktree remove <branch>
godmode ci triage [--run-id <id>]
godmode issue list [--repo owner/repo] [--label <label>]
godmode issue close <number> --commit <sha> [--repo owner/repo]
godmode issue sync-todos [--preview|--apply] [--repo owner/repo]  # TODO markers <-> issues
godmode skill list                              # list registered skills
godmode skill install <path>                    # install skill from a local dir
godmode skill uninstall <name>                  # remove a skill from the registry
godmode release current                         # show current plugin version
godmode release bump [--version X]              # increment patch version everywhere
godmode release tag                             # create annotated git tag
godmode release push                            # push branch + tag to origin
godmode release changelog                       # generate/prepend changelog entry
godmode release validate                        # cross-check plugin.json/Cargo.toml/tag
godmode pipeline list                           # list available pipelines
godmode pipeline show <name>                    # show steps + current position
godmode pipeline start <name> [--from <skill>]  # activate a pipeline
godmode pipeline next                           # mark current step done, advance
godmode pipeline skip                           # advance without marking done
godmode pipeline stop                           # deactivate current pipeline
godmode pipeline status                         # active pipeline + progress
godmode pipeline run <name> [--from <skill>] [--fail-fast]  # run headlessly
godmode policy resolve <agent> [--level L] [--json]   # effective policy for an agent
godmode policy check <agent> <tool> [--input <text>] [--level L]  # check a tool call
godmode policy list [--json]                        # list default/category/level policies
godmode policy audit [--date YYYY-MM-DD] [--json]   # governance audit trail
godmode hook list
godmode hook log [--tail N]
godmode hook test <script>
godmode hook migrate
godmode hook run <name>
godmode hook generate --client <claude|opencode> [--dry-run]
godmode review self
godmode review skills
godmode review agents
godmode trace tail [--n N] [--session ID|--current]
godmode trace failures [--session ID|--current]
godmode trace stats [--session ID|--current]
godmode trace summary [--sessions N] [--previous]
godmode agent generate-opencode --all [--dry-run]
godmode insight add <title> --body <text> [--tags t1,t2]
godmode insight list [--date YYYY-MM-DD] [--json]
godmode insight render [--date YYYY-MM-DD]
godmode session prune --older-than <days> [--dry-run]
godmode workflow run <agent> <workflow>
godmode workflow list [--agent <name>]
godmode workflow status <name>
godmode scaffold <crate> <dimension>                    # test stub generator
godmode test-check <path>                               # check if .rs file has tests
```

`--json` is a global flag accepted by `skill`, `policy`, and `review` output.

`task done` accepts `--commit <sha>` and `--notes <text>` for trace metadata.
