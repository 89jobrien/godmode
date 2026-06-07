#!/usr/bin/env nu
# check-tool.nu — Check if a tool call is allowed by the effective policy.
#
# Used by hook.nu to validate individual tool invocations.
# Returns JSON: { "action": "allow"|"deny"|"review", "reason": "..." }
#
# Usage:
#   nu check-tool.nu <agent-name> <tool-name> [--input <content>] [--level standard]

use ../../_lib/trace.nu *

def main [
    agent_name: string
    tool_name: string
    --input: string = ""     # tool input content to check against blocked patterns
    --level: string = ""     # governance level override
] {
    let git_root = (do { git rev-parse --show-toplevel } | complete)
    if $git_root.exit_code != 0 {
        { action: "allow", reason: "not in git repo — no policy enforcement" } | to json
        exit 0
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
        { action: "deny", reason: "policy resolution failed — fail closed" } | to json
        exit 0
    }

    let policy = $resolve_result.stdout | from json

    # 1. Check blocked_tools
    let blocked = $policy | get blocked_tools? | default []
    if ($tool_name in $blocked) {
        {
            action: "deny"
            reason: $"tool '($tool_name)' is in blocked_tools for ($agent_name)"
        } | to json
        exit 0
    }

    # 2. Check allowed_tools (if non-empty, tool must be in list)
    let allowed = $policy | get allowed_tools? | default []
    if ($allowed | length) > 0 and not ($tool_name in $allowed) {
        {
            action: "deny"
            reason: $"tool '($tool_name)' not in allowed_tools for ($agent_name)"
        } | to json
        exit 0
    }

    # 3. Check require_human_approval
    let approvals = $policy | get require_human_approval? | default []
    if ($tool_name in $approvals) or ("*" in $approvals) {
        {
            action: "review"
            reason: $"tool '($tool_name)' requires human approval for ($agent_name)"
        } | to json
        exit 0
    }

    # 4. Check content against blocked_patterns
    if ($input != "") {
        let patterns = $policy | get blocked_patterns? | default []
        for pattern in $patterns {
            let match_result = do { echo $input | grep -qP $pattern } | complete
            if $match_result.exit_code == 0 {
                {
                    action: "deny"
                    reason: $"content matches blocked pattern: ($pattern)"
                } | to json
                exit 0
            }
        }
    }

    { action: "allow", reason: "passed all policy checks" } | to json
}
