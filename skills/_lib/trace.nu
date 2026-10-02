#!/usr/bin/env nu
# _lib/trace.nu — thin Nushell facade over `godmode trace emit`.
#
# Source this module at the top of any helper:
#   use (/path/to/skills/_lib/trace.nu) *
#
# The event vocabulary, the session-id rotation, and the trace-id format all
# live in `godmode-core` (`hooks::trace_log`). This file exists only so the 18
# helpers that call `trace-start` and friends keep working unchanged; it holds
# no logic of its own and must not grow any.
#
# Session identity is resolved by `godmode` and persisted to
# .ctx/godmode/session.json, so every helper in a session shares one session_id
# regardless of which process wrote an event.

# ---------------------------------------------------------------------------
# Internal
# ---------------------------------------------------------------------------

# Locate the godmode binary, honouring GODMODE_BIN for tests and for repos
# that vendored a build rather than installing it.
def godmode-bin [] {
    if ($env.GODMODE_BIN? | default "") != "" {
        $env.GODMODE_BIN
    } else {
        which godmode | get -o path | last
    }
}

# Run `godmode trace emit`, swallowing failure. A trace write must never be
# able to fail the helper that made the call, so the exit code is discarded.
#
# Takes an explicit list rather than rest args: a rest-arg call still goes
# through flag parsing, so a passthrough `--trace-id` would be claimed by this
# command instead of being forwarded.
def emit-event [argv: list<string>] {
    let bin = (try { godmode-bin } catch { return })
    if ($bin == null) { return }
    try { ^$bin ...$argv | complete | ignore } catch { null }
}

# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

# Emit skill.start; returns a trace_id for use with trace-end / trace-error.
export def trace-start [skill: string, helper: string, ...args: string] {
    let bin = (try { godmode-bin } catch { return null })
    # Without the binary there is no vocabulary to emit, so hand back a sentinel
    # the closing calls will reject rather than invent an id godmode cannot read.
    if ($bin == null) { return "untraced" }
    let flags = ($args | each {|a| ["--args" $a] } | flatten)
    let argv = (["trace" "emit" "skill-start" "--skill" $skill "--helper" $helper] | append $flags)
    let out = (^$bin ...$argv | complete)
    if ($out.exit_code != 0) {
        if ($out.stderr | str contains "unrecognized subcommand") {
            warn-unsupported
        }
        return "untraced"
    }
    $out.stdout | str trim
}

# Warn that this godmode predates `trace emit`.
#
# `trace emit` is newer than the released CLI, so a skills checkout paired with
# an older godmode would otherwise emit nothing and say nothing. Silence here
# reads as "nothing happened", which is indistinguishable from a quiet day.
#
# Not rate-limited: an env var set inside a `def` does not outlive the call in
# Nushell, so there is no cheap once-per-process guard. A helper calls
# trace-start once, so this is one line per helper run, and the situation
# disappears once the CLI and the plugin ship together.
def warn-unsupported [] {
    # Keep the literal free of parentheses, backticks, and quotes: inside an
    # interpolated string Nushell treats those as command substitution.
    let which = ($env.GODMODE_BIN? | default "godmode")
    let msg = $"warning: ($which) has no trace-emit subcommand, so observability events are being dropped. Install a godmode build that has it."
    print --stderr $msg
}

# Emit skill.complete.
export def trace-end [trace_id: string] {
    emit-event ["trace" "emit" "skill-complete" "--trace-id" $trace_id]
}

# Emit skill.error.
export def trace-error [trace_id: string, exit_code: int, stderr_tail: string] {
    let tail = ($stderr_tail | lines | last 10 | str join "\n")
    emit-event ["trace" "emit" "skill-error" "--trace-id" $trace_id "--exit-code" $"($exit_code)" "--stderr-tail" $tail]
}

# Emit a branching decision (CI classification, BLOCKED.md found, merge skipped, etc.).
export def trace-decision [skill: string, helper: string, kind: string, value: string] {
    emit-event ["trace" "emit" "decision" "--skill" $skill "--helper" $helper "--kind" $kind "--value" $value]
}

# Emit agent.start (called by orchestrator before dispatching a subagent).
export def trace-agent-start [agent_id: string, slot: string, crate: string] {
    emit-event ["trace" "emit" "agent-start" "--agent-id" $agent_id "--slot" $slot "--crate" $crate]
}

# Emit agent.complete.
export def trace-agent-complete [agent_id: string, slot: string, commits: list<string>] {
    let flags = ($commits | each {|c| ["--commits" $c] } | flatten)
    let argv = (["trace" "emit" "agent-complete" "--agent-id" $agent_id "--slot" $slot] | append $flags)
    emit-event $argv
}

# Emit agent.blocked.
export def trace-agent-blocked [agent_id: string, slot: string, reason: string] {
    emit-event ["trace" "emit" "agent-blocked" "--agent-id" $agent_id "--slot" $slot "--reason" $reason]
}
