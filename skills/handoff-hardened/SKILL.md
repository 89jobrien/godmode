---
name: handoff-hardened
description: Resumable, self-verifying end-of-day handoff. Enumerates dirty/unpushed repos, dispatches per-repo handoff subagents with a hard timeout and one retry (falling back to a partial-handoff marker instead of dropping the repo), persists a checkpoint so a rerun resumes rather than restarts, then runs a dedicated verification subagent that independently confirms every factual claim (commit counts, test results, versions, task status) against git log / cargo test / gh issue state before gating the memory-bank commit on that verification passing. Use for "harden my eod", "resumable handoff", "verify my handoff claims", or when a prior handoff run stalled or was interrupted.
allowed_tools:
  - Bash
  - Read
  - Glob
  - Agent
  - Write
max_turns: 40
---

## State machine

DISCOVER -> DISPATCHED -> RUNNING -> {COMPLETE | RETRY -> PARTIAL} -> VERIFYING -> {VERIFIED | FLAGGED} -> COMMIT-GATE -> DONE

Checkpoint file: `~/dev/.ctx/godmode/handoff-checkpoint.json`, keyed by repo name.
Statuses persisted per repo: PENDING, RUNNING, RETRY, PARTIAL, COMPLETE, VERIFIED, FLAGGED, COMMITTED.

## Rules

- This skill does NOT depend on `skills/observability-as-infrastructure/helpers/trace-stats.nu`.
  If it exists, call it opportunistically for extra stats in the final report — never fail
  the run if it's missing or errors.
- Never silently drop a repo. Any repo that fails twice becomes a PARTIAL marker in the
  checkpoint and in the final report — never omitted.
- Never delete a claim flagged as unverifiable by the verification subagent. Flag it inline
  in the handoff doc; leave the claim text intact.
- Only commit memory-bank/HANDOFF updates after the verification subagent has run and
  produced a report (even if it flags issues). If verification itself crashes/times out,
  do NOT commit — report that verification did not complete and stop before COMMIT-GATE.
- Per-repo handoff subagent timeout: 5 minutes. On timeout or error: retry exactly once.
  Second failure -> PARTIAL, do not retry again.
- On rerun, load the checkpoint first. Skip any repo already COMPLETE/PARTIAL/VERIFIED/
  COMMITTED. Only re-enter DISPATCHED for repos still PENDING/RUNNING/RETRY (interrupted).

## Procedure

1. **DISCOVER**: run `skills/handoff-hardened/helpers/discover-dirty-repos.nu` to enumerate
   repos with uncommitted or unpushed work. Load the checkpoint via
   `skills/handoff-hardened/helpers/checkpoint.nu` (`load-checkpoint`). Compute the pending
   set with `pending-repos`. If the pending set is empty and the checkpoint has entries for
   all discovered repos, skip straight to step 4 (resume case).

2. **DISPATCHED / RUNNING / RETRY / PARTIAL** (per pending repo, dispatched via the Agent
   tool with `atelier:minion` or `atelier:forge`, `run_in_background: true` is NOT used here
   — dispatch synchronously per repo or in small batches, since each needs a hard timeout you
   enforce yourself):
   - Mark checkpoint entry RUNNING (`update-repo`, `save-checkpoint`) before dispatch.
   - Spawn a handoff subagent for the repo: "Generate a HANDOFF update for <repo> at <path>.
     Report factual claims explicitly: commit count since last handoff, test pass/fail
     counts if you ran tests, version numbers touched, and status of each task item.
     Write the HANDOFF.<repo>.\*.yaml update but do not commit or push."
   - Track wall-clock time yourself; if the agent call is still unresolved past 5 minutes
     (in practice: the Agent tool call itself will return — if it does not return in a
     reasonable turn, treat that as a stall), or if it errors, mark checkpoint RETRY and
     dispatch exactly once more with the same prompt.
   - On second failure: mark checkpoint PARTIAL with `error` set to the failure reason and
     `claims` set to whatever partial info is known (e.g., raw git log even if the subagent
     never produced prose). Continue to the next repo — do not block the run.
   - On success: mark checkpoint COMPLETE, store the subagent's claimed facts under `claims`.

3. After all repos reach a terminal per-repo state (COMPLETE or PARTIAL), proceed.

4. **VERIFYING**: dispatch one verification subagent (Agent tool, `general-purpose` or
   `atelier:sentinel`) with the full set of COMPLETE repos' claims plus their paths. Instruct
   it explicitly: "For each factual claim (commit counts, test results, version numbers, task
   statuses) independently confirm against `git log`, `cargo test` output (rerun if needed),
   and `gh issue` state. Do not trust the claim at face value. For each claim, return
   confirmed or unverifiable-with-reason. Do not delete or omit any claim — flag it inline."
   Parse its structured findings.

5. **VERIFIED / FLAGGED**: update each repo's checkpoint entry to VERIFIED, storing the
   verification subagent's per-claim confirmed/flagged results alongside the original claims
   (flagged claims stay in the record, tagged, not removed).

6. **COMMIT-GATE**: only if step 4 produced a report (regardless of flagged claims), commit
   the memory-bank/HANDOFF file updates for repos in COMPLETE/VERIFIED state. Use `git add`
   scoped to the specific HANDOFF/memory files touched, not `-A`. Mark checkpoint COMMITTED
   for each repo actually committed. If verification did not complete, skip this step and
   say so explicitly in the report.

7. **DONE**: report per repo: final status, attempts used, whether PARTIAL, any FLAGGED
   claims (verbatim), and commit sha if committed. Report the checkpoint path so the user can
   inspect or rerun.

## Resuming

Rerunning this skill is always safe: it re-reads the checkpoint and only touches repos not
already terminal. To force a full restart, delete
`~/dev/.ctx/godmode/handoff-checkpoint.json` first (ask before doing this — it's a
reversible-but-explicit action).
