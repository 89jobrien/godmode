#!/usr/bin/env nu
# _lib/trace.nu — shared observability primitives for godmode helpers.
#
# Source this module at the top of any helper:
#   use (/path/to/skills/_lib/trace.nu) *
#
# Session identity is lazily initialised on first write and persisted to
# .ctx/GODMODE.session.json so all helpers in a session share one session_id.

# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

def repo-root [] {
    (run-external "git" "rev-parse" "--show-toplevel" | complete).stdout | str trim
}

def trace-file [] {
    $"(repo-root)/.ctx/GODMODE.trace.jsonl"
}

def session-file [] {
    $"(repo-root)/.ctx/GODMODE.session.json"
}

def now-ms [] {
    date now | into int | $in / 1_000_000 | into int
}

# Resolve or create the current session_id.
def session-id [] {
    let sf = (session-file)
    if ($sf | path exists) {
        (open $sf).session_id
    } else {
        let head = (run-external "git" "rev-parse" "--short" "HEAD" | complete).stdout | str trim
        let id = $"($head)-(now-ms)"
        { session_id: $id, started_at: (date now | format date "%+") } | to json | save --force $sf
        $id
    }
}

# Append one JSON line to the trace file.
def append-event [record: record] {
    let root = (repo-root)
    mkdir $"($root)/.ctx"
    $record | to json --raw | $"($in)\n" | save --append (trace-file)
}

# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

# Emit skill.start; returns a trace_id string for use with trace-end / trace-error.
export def trace-start [skill: string, helper: string, ...args: string] {
    let tid = $"($skill).($helper).(now-ms)"
    append-event {
        event:      "skill.start"
        trace_id:   $tid
        skill:      $skill
        helper:     $helper
        args:       $args
        session_id: (session-id)
        ts:         (date now | format date "%+")
    }
    $tid
}

# Emit skill.complete.
export def trace-end [trace_id: string] {
    let parts = ($trace_id | split row ".")
    let started_ms = ($parts | last | into int)
    let duration_ms = ((now-ms) - $started_ms)
    append-event {
        event:      "skill.complete"
        trace_id:   $trace_id
        duration_ms: $duration_ms
        session_id: (session-id)
        ts:         (date now | format date "%+")
    }
}

# Emit skill.error.
export def trace-error [trace_id: string, exit_code: int, stderr_tail: string] {
    let parts = ($trace_id | split row ".")
    let started_ms = ($parts | last | into int)
    let duration_ms = ((now-ms) - $started_ms)
    append-event {
        event:       "skill.error"
        trace_id:    $trace_id
        exit_code:   $exit_code
        stderr_tail: ($stderr_tail | lines | last 10 | str join "\n")
        duration_ms: $duration_ms
        session_id:  (session-id)
        ts:          (date now | format date "%+")
    }
}

# Emit a branching decision (CI classification, BLOCKED.md found, merge skipped, etc.).
export def trace-decision [skill: string, helper: string, kind: string, value: string] {
    append-event {
        event:      "decision"
        skill:      $skill
        helper:     $helper
        kind:       $kind
        value:      $value
        session_id: (session-id)
        ts:         (date now | format date "%+")
    }
}

# Emit agent.start (called by orchestrator before dispatching a subagent).
export def trace-agent-start [agent_id: string, slot: string, crate: string] {
    append-event {
        event:      "agent.start"
        agent_id:   $agent_id
        slot:       $slot
        crate:      $crate
        session_id: (session-id)
        ts:         (date now | format date "%+")
    }
}

# Emit agent.complete.
export def trace-agent-complete [agent_id: string, slot: string, commits: list<string>] {
    append-event {
        event:      "agent.complete"
        agent_id:   $agent_id
        slot:       $slot
        commits:    $commits
        session_id: (session-id)
        ts:         (date now | format date "%+")
    }
}

# Emit agent.blocked.
export def trace-agent-blocked [agent_id: string, slot: string, reason: string] {
    append-event {
        event:      "agent.blocked"
        agent_id:   $agent_id
        slot:       $slot
        reason:     $reason
        session_id: (session-id)
        ts:         (date now | format date "%+")
    }
}
