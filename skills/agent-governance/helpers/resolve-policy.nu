#!/usr/bin/env nu
# resolve-policy.nu — Resolve the effective governance policy for an agent.
#
# Loads default.yaml, overlays the category policy (from agents/cfg/*.cfg.yaml),
# then overlays the governance level if specified. Composition follows
# most-restrictive-wins: blocked lists union, allowed lists intersect,
# rate limits take minimum, human-approval unions.
#
# Usage:
#   nu resolve-policy.nu <agent-name> [--level standard] [--json]
#
# Examples:
#   nu resolve-policy.nu gm-orchestrator
#   nu resolve-policy.nu gm-cap-agent --level strict
#   nu resolve-policy.nu gm-dispatch --json

use ../../_lib/trace.nu *

def main [
    agent_name: string      # agent name (matches agents/cfg/<name>.cfg.yaml)
    --level: string = ""    # governance level override (open/standard/strict/locked)
    --json                  # output as JSON instead of table
] {
    let _tid = (trace-start "agent-governance" "resolve-policy.nu" $agent_name)

    let git_root = (do { git rev-parse --show-toplevel } | complete)
    if $git_root.exit_code != 0 {
        print --stderr "[resolve-policy] Not in a git repo"
        exit 1
    }
    let root = $git_root.stdout | str trim
    let skill_dir = $"($root)/skills/agent-governance"

    # 1. Load default policy
    let default_path = $"($skill_dir)/policies/default.yaml"
    if not ($default_path | path exists) {
        print --stderr $"[resolve-policy] Missing default policy: ($default_path)"
        exit 1
    }
    let default_policy = open $default_path

    # 2. Resolve agent category from cfg
    let cfg_path = $"($root)/agents/cfg/($agent_name).cfg.yaml"
    let category = if ($cfg_path | path exists) {
        let cfg = open $cfg_path
        $cfg | get category? | default ""
    } else {
        ""
    }

    # 3. Load category overlay if it exists
    let category_path = $"($skill_dir)/policies/by-category/($category).yaml"
    let category_policy = if ($category != "" and ($category_path | path exists)) {
        open $category_path
    } else {
        null
    }

    # 4. Load level overlay if specified
    let effective_level = if ($level != "") { $level } else {
        $default_policy | get level? | default "standard"
    }
    let level_path = $"($skill_dir)/policies/levels/($effective_level).yaml"
    let level_policy = if ($level_path | path exists) {
        open $level_path
    } else {
        null
    }

    # 5. Compose: default -> category -> level (most-restrictive-wins)
    let effective = (compose-policies $default_policy $category_policy $level_policy)

    # 6. Annotate with resolution metadata
    let result = $effective | merge {
        _resolved: {
            agent: $agent_name
            category: $category
            level: $effective_level
            sources: (
                ["default.yaml"]
                | if ($category_policy != null) {
                    append $"by-category/($category).yaml"
                } else { $in }
                | if ($level_policy != null) {
                    append $"levels/($effective_level).yaml"
                } else { $in }
            )
        }
    }

    if $json {
        $result | to json
    } else {
        print $"Agent:    ($agent_name)"
        print $"Category: ($category)"
        print $"Level:    ($effective_level)"
        print ""
        print "Allowed tools:"
        let allowed = $result | get allowed_tools? | default []
        if ($allowed | is-empty) {
            print "  (all — no restriction)"
        } else {
            $allowed | each { |t| print $"  - ($t)" }
        }
        print ""
        print "Blocked tools:"
        let blocked = $result | get blocked_tools? | default []
        if ($blocked | is-empty) {
            print "  (none)"
        } else {
            $blocked | each { |t| print $"  - ($t)" }
        }
        print ""
        print $"Max calls/dispatch: ($result | get max_calls_per_dispatch? | default 200)"
        print ""
        let approvals = $result | get require_human_approval? | default []
        if ($approvals | length) > 0 {
            print "Require human approval:"
            $approvals | each { |a| print $"  - ($a)" }
        }
    }

    trace-end $_tid
}

# Compose multiple policies with most-restrictive-wins semantics.
# - blocked_tools / blocked_patterns / require_human_approval: union
# - allowed_tools: intersection (if both non-empty)
# - max_calls_per_dispatch: minimum
# - subagent: per-field most-restrictive
def compose-policies [
    base: record
    category: any       # record or null
    level: any          # record or null
] {
    mut result = $base

    for overlay in [$category $level] {
        if ($overlay == null) { continue }

        # Union blocked_tools
        let overlay_blocked = $overlay | get blocked_tools? | default []
        let base_blocked = $result | get blocked_tools? | default []
        $result = ($result | merge {
            blocked_tools: ($base_blocked | append $overlay_blocked | uniq)
        })

        # Union blocked_patterns
        let overlay_patterns = $overlay | get blocked_patterns? | default []
        let base_patterns = $result | get blocked_patterns? | default []
        $result = ($result | merge {
            blocked_patterns: ($base_patterns | append $overlay_patterns | uniq)
        })

        # Union require_human_approval
        let overlay_approvals = $overlay
            | get require_human_approval? | default []
        let base_approvals = $result
            | get require_human_approval? | default []
        $result = ($result | merge {
            require_human_approval: (
                $base_approvals | append $overlay_approvals | uniq
            )
        })

        # Intersect allowed_tools (only if overlay specifies any)
        let overlay_allowed = $overlay | get allowed_tools? | default []
        if ($overlay_allowed | length) > 0 {
            let base_allowed = $result | get allowed_tools? | default []
            if ($base_allowed | length) > 0 {
                $result = ($result | merge {
                    allowed_tools: (
                        $base_allowed
                        | where { |t| $t in $overlay_allowed }
                    )
                })
            } else {
                $result = ($result | merge {
                    allowed_tools: $overlay_allowed
                })
            }
        }

        # Min max_calls_per_dispatch
        let overlay_max = $overlay
            | get max_calls_per_dispatch? | default 9999
        let base_max = $result
            | get max_calls_per_dispatch? | default 200
        $result = ($result | merge {
            max_calls_per_dispatch: ([$base_max $overlay_max] | math min)
        })
    }

    $result
}
