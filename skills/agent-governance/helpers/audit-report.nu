#!/usr/bin/env nu
# audit-report.nu — Generate a governance audit report from trace data.
#
# Reads .ctx/godmode/traces/governance-audit.jsonl and trace.jsonl,
# correlates governance events with agent lifecycle, and produces a
# summary report.
#
# Usage:
#   nu audit-report.nu [--date 2026-06-04] [--json]

use ../../_lib/trace.nu *

def main [
    --date: string = ""   # filter to specific date (YYYY-MM-DD), default today
    --json                # output as JSON instead of markdown
] {
    let _tid = (trace-start "agent-governance" "audit-report.nu")

    let git_root = (do { git rev-parse --show-toplevel } | complete)
    if $git_root.exit_code != 0 {
        print --stderr "[audit-report] Not in a git repo"
        exit 1
    }
    let root = $git_root.stdout | str trim
    let traces_dir = $"($root)/.ctx/godmode/traces"

    let target_date = if ($date != "") { $date } else {
        date now | format date "%Y-%m-%d"
    }

    # Read governance audit log
    let gov_path = $"($traces_dir)/governance-audit.jsonl"
    let gov_events = if ($gov_path | path exists) {
        open --raw $gov_path
        | lines
        | where { |l| not ($l | is-empty) }
        | each { |l| try { $l | from json } catch { null } }
        | where { |e| $e != null }
        | where { |e|
            let ts = $e | get ts? | default ""
            $ts | str starts-with $target_date
        }
    } else {
        []
    }

    # Read agent lifecycle events from main trace
    let trace_path = $"($traces_dir)/trace.jsonl"
    let agent_events = if ($trace_path | path exists) {
        open --raw $trace_path
        | lines
        | where { |l| not ($l | is-empty) }
        | each { |l| try { $l | from json } catch { null } }
        | where { |e| $e != null }
        | where { |e|
            let event = $e | get event? | default ""
            let ts = $e | get ts? | default ""
            ($event | str starts-with "agent.") and ($ts | str starts-with $target_date)
        }
    } else {
        []
    }

    # Build report
    let denied = $gov_events | where { |e|
        ($e | get action? | default "") == "denied"
    }
    let reviews = $gov_events | where { |e|
        ($e | get action? | default "") == "review"
    }
    let allowed = $gov_events | where { |e|
        ($e | get action? | default "") == "allowed"
    }

    let agent_starts = $agent_events | where { |e|
        ($e | get event? | default "") == "agent.start"
    }
    let agent_completes = $agent_events | where { |e|
        ($e | get event? | default "") == "agent.complete"
    }
    let agent_blocked = $agent_events | where { |e|
        ($e | get event? | default "") == "agent.blocked"
    }

    let report = {
        date: $target_date
        governance_events: {
            total: ($gov_events | length)
            denied: ($denied | length)
            reviews: ($reviews | length)
            allowed: ($allowed | length)
        }
        agent_lifecycle: {
            started: ($agent_starts | length)
            completed: ($agent_completes | length)
            blocked: ($agent_blocked | length)
        }
        denials: ($denied | each { |e| {
            agent: ($e | get agent_id? | default "unknown")
            tool: ($e | get tool_name? | default "unknown")
            policy: ($e | get policy_name? | default "unknown")
            reason: ($e | get reason? | default "")
            ts: ($e | get ts? | default "")
        }})
        reviews_pending: ($reviews | each { |e| {
            agent: ($e | get agent_id? | default "unknown")
            tool: ($e | get tool_name? | default "unknown")
            ts: ($e | get ts? | default "")
        }})
    }

    if $json {
        $report | to json
    } else {
        print $"# Governance Audit Report — ($target_date)"
        print ""
        print "## Summary"
        print ""
        print $"| Metric          | Count |"
        print $"| --------------- | ----- |"
        print $"| Total events    | ($report.governance_events.total) |"
        print $"| Denied          | ($report.governance_events.denied) |"
        print $"| Reviews         | ($report.governance_events.reviews) |"
        print $"| Allowed         | ($report.governance_events.allowed) |"
        print $"| Agents started  | ($report.agent_lifecycle.started) |"
        print $"| Agents done     | ($report.agent_lifecycle.completed) |"
        print $"| Agents blocked  | ($report.agent_lifecycle.blocked) |"
        print ""

        if ($report.denials | length) > 0 {
            print "## Denials"
            print ""
            for denial in $report.denials {
                print $"- **($denial.agent)** tried `($denial.tool)` — denied by `($denial.policy)`: ($denial.reason)"
            }
            print ""
        }

        if ($report.reviews_pending | length) > 0 {
            print "## Pending Reviews"
            print ""
            for review in $report.reviews_pending {
                print $"- **($review.agent)** requested `($review.tool)` at ($review.ts)"
            }
            print ""
        }

        if ($report.denials | length) == 0 and ($report.reviews_pending | length) == 0 {
            print "No policy violations or pending reviews."
        }
    }

    trace-end $_tid
}
