#!/usr/bin/env nu
# check-tool.nu — Check if a tool call is allowed by the effective policy.
#
# Used by hook.nu to validate individual tool invocations.
# Returns JSON: { "action": "allow"|"deny"|"review", "reason": "..." }
#
# Usage:
#   nu check-tool.nu <agent-name> <tool-name> [--input <content>] [--level standard]

use ../../_lib/trace.nu *

# Emit a decision and stop. MUST print explicitly: nu only prints main's
# return value, so `{ ... } | to json; exit 0` silently discarded every
# deny/review payload and the caller saw empty stdout with exit 0.
def decide [decision: record] {
    print ($decision | to json)
    exit 0
}

def main [
    agent_name: string
    tool_name: string
    --input: string = ""     # tool input content to check against blocked patterns
    --level: string = ""     # governance level override
] {
    let git_root = (do { git rev-parse --show-toplevel } | complete)
    if $git_root.exit_code != 0 {
        decide { action: "allow", reason: "not in git repo — no policy enforcement" }
    }
    let root = $git_root.stdout | str trim
    let skill_dir = $"($root)/skills/agent-governance"

    # Resolve effective policy
    let level_flag = if ($level != "") { ["--level" $level] } else { [] }
    let resolve_result = do {
        nu $"($skill_dir)/helpers/resolve-policy.nu" $agent_name ...$level_flag --json
    } | complete

    if $resolve_result.exit_code != 0 {
        # Fail closed — if we can't resolve policy, deny
        decide { action: "deny", reason: "policy resolution failed — fail closed" }
    }

    let policy = $resolve_result.stdout | from json
    if ($policy == null) {
        decide { action: "deny", reason: $"empty policy for ($agent_name) — fail closed" }
    }

    # 1. Check blocked_tools
    let blocked = $policy | get blocked_tools? | default []
    if ($tool_name in $blocked) {
        decide {
            action: "deny"
            reason: $"tool '($tool_name)' is in blocked_tools for ($agent_name)"
        }
    }

    # 2. Check allowed_tools (if non-empty, tool must be in list)
    let allowed = $policy | get allowed_tools? | default []
    if ($allowed | length) > 0 and not ($tool_name in $allowed) {
        decide {
            action: "deny"
            reason: $"tool '($tool_name)' not in allowed_tools for ($agent_name)"
        }
    }

    # 3. Check require_human_approval
    let approvals = $policy | get require_human_approval? | default []
    if ($tool_name in $approvals) or ("*" in $approvals) {
        decide {
            action: "review"
            reason: $"tool '($tool_name)' requires human approval for ($agent_name)"
        }
    }

    # 4. Check content against blocked_patterns
    if ($input != "") {
        let patterns = $policy | get blocked_patterns? | default []
        for pattern in $patterns {
            # Match with nu's native regex engine. Do NOT shell out to
            # `grep -qP`: BSD grep (macOS) has no -P, so every blocked_patterns
            # check exited 2 and content filtering never denied.
            let matched = try { $input =~ $pattern } catch { false }
            if $matched {
                decide {
                    action: "deny"
                    reason: $"content matches blocked pattern: ($pattern)"
                }
            }
        }
    }

    decide { action: "allow", reason: "passed all policy checks" }
}
