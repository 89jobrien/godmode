#!/usr/bin/env nu
# agent-governance/hook.nu — PreToolUse/Agent hook
#
# Resolves the effective governance policy for the agent being dispatched,
# validates tool access against the policy, checks subagent constraints,
# and emits governance trace events. Blocks dispatch if policy is violated.
#
# Input: Claude hook JSON on stdin (tool_input with prompt, description, etc.)
# Output: JSON { "decision": "approve"|"block", "reason": "..." }
#
# Always exits 0. Outputs decision JSON to stdout.

use ../_lib/trace.nu *
let _tid = (trace-start "agent-governance" "hook.nu")

let input = try { open --raw /dev/stdin | from json } catch {
    print '{"decision":"approve"}'
    exit 0
}

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    print '{"decision":"approve"}'
    exit 0
}

let git_root = $git_result.stdout | str trim
let skill_dir = $"($git_root)/skills/agent-governance"
let policies_dir = $"($skill_dir)/policies"
let traces_dir = $"($git_root)/.ctx/godmode/traces"

# Ensure trace directory exists
if not ($traces_dir | path exists) {
    try { mkdir $traces_dir }
}

# ---------------------------------------------------------------------------
# 1. Extract agent identity from tool_input
# ---------------------------------------------------------------------------

let tool_input = $input | get -i tool_input | default {}
let description = $tool_input | get -i description | default ""
let prompt_text = $tool_input | get -i prompt | default ""
let subagent_type = $tool_input | get -i subagent_type | default ""

# Try to identify the agent name from description or prompt
let agent_name = (detect-agent-name $description $prompt_text $subagent_type)

# ---------------------------------------------------------------------------
# 2. Check for governance policy files
# ---------------------------------------------------------------------------

let default_policy_path = $"($policies_dir)/default.yaml"
if not ($default_policy_path | path exists) {
    print --stderr "[godmode:agent-governance] No default policy found — governance not enforced"
    print '{"decision":"approve"}'
    trace-end $_tid
    exit 0
}

# ---------------------------------------------------------------------------
# 3. Resolve effective policy (prefer godmode CLI, fall back to nu helper)
# ---------------------------------------------------------------------------

let godmode_on_path = (which godmode | length) > 0
let resolve_agent = if ($agent_name != "") { $agent_name } else { "unknown" }

let resolve_result = if $godmode_on_path {
    do { godmode policy resolve $resolve_agent --json } | complete
} else {
    do {
        nu $"($skill_dir)/helpers/resolve-policy.nu" $resolve_agent --json
    } | complete
}

let policy = if $resolve_result.exit_code == 0 {
    try { $resolve_result.stdout | from json } catch { null }
} else {
    null
}

if $policy == null {
    # Fail open with warning — policy resolution failed
    print --stderr "[godmode:agent-governance] Policy resolution failed — approving with warning"
    emit-governance-event $traces_dir "warn" $agent_name "Agent" "policy_resolution_failed" ""
    print '{"decision":"approve"}'
    trace-end $_tid
    exit 0
}

# ---------------------------------------------------------------------------
# 4. Check subagent constraints
# ---------------------------------------------------------------------------

let subagent_rules = $policy | get -i subagent | default {}

# Check max_concurrent (count running agents from wave status)
let max_concurrent = $subagent_rules | get -i max_concurrent | default 5
if $max_concurrent == 0 {
    let reason = "Policy forbids subagent dispatch (max_concurrent: 0)"
    print --stderr $"[godmode:agent-governance] BLOCKED: ($reason)"
    emit-governance-event $traces_dir "denied" $agent_name "Agent" $reason ""
    print $'{"decision":"block","reason":"($reason)"}'
    trace-end $_tid
    exit 0
}

# Check no_commit_to_main — warn in prompt if true
let no_main = $subagent_rules | get -i no_commit_to_main | default true
let must_verify = $subagent_rules | get -i must_verify_branch | default true

# Check Agent tool is in allowed_tools
let allowed = $policy | get -i allowed_tools | default []
if ($allowed | length) > 0 and not ("Agent" in $allowed) {
    let reason = $"Policy for ($agent_name) does not include Agent in allowed_tools"
    print --stderr $"[godmode:agent-governance] BLOCKED: ($reason)"
    emit-governance-event $traces_dir "denied" $agent_name "Agent" $reason ""
    print $'{"decision":"block","reason":"($reason)"}'
    trace-end $_tid
    exit 0
}

# Check Agent tool is not in blocked_tools
let blocked = $policy | get -i blocked_tools | default []
if "Agent" in $blocked {
    let reason = $"Policy for ($agent_name) blocks Agent tool"
    print --stderr $"[godmode:agent-governance] BLOCKED: ($reason)"
    emit-governance-event $traces_dir "denied" $agent_name "Agent" $reason ""
    print $'{"decision":"block","reason":"($reason)"}'
    trace-end $_tid
    exit 0
}

# ---------------------------------------------------------------------------
# 5. Scan prompt/description for blocked patterns
# ---------------------------------------------------------------------------

let patterns = $policy | get -i blocked_patterns | default []
let content_to_check = $"($description)\n($prompt_text)"

for pattern in $patterns {
    let match_result = try {
        $content_to_check | find --regex $pattern
    } catch { [] }
    if ($match_result | length) > 0 {
        let reason = $"Content matches blocked pattern: ($pattern)"
        print --stderr $"[godmode:agent-governance] BLOCKED: ($reason)"
        emit-governance-event $traces_dir "denied" $agent_name "Agent" $reason $pattern
        print $'{"decision":"block","reason":"($reason)"}'
        trace-end $_tid
        exit 0
    }
}

# ---------------------------------------------------------------------------
# 6. Approved — emit audit event and inject governance context
# ---------------------------------------------------------------------------

emit-governance-event $traces_dir "allowed" $agent_name "Agent" "passed all checks" ""

# Inject governance reminders into stderr (visible to orchestrator)
let category = $policy | get -i _resolved?.category? | default ""
let level = $policy | get -i _resolved?.level? | default "standard"

mut reminders = []
if $no_main {
    $reminders = ($reminders | append "Do NOT commit to main — verify branch first")
}
if $must_verify {
    $reminders = ($reminders | append "Run `git branch --show-current` before every commit")
}
let max_retries = $subagent_rules | get -i max_retries_on_failure | default 3
$reminders = ($reminders | append $"Max retries on failure: ($max_retries)")
let require_commit = $subagent_rules | get -i require_commit_before_done | default true
if $require_commit {
    $reminders = ($reminders | append "Must commit before reporting done")
}

let blocked_flags = $subagent_rules | get -i blocked_flags | default []
if ($blocked_flags | length) > 0 {
    $reminders = ($reminders | append $"Blocked flags: ($blocked_flags | str join ', ')")
}

if ($reminders | length) > 0 {
    let reminder_text = $reminders | each { |r| $"  - ($r)" } | str join "\n"
    print --stderr $"[godmode:agent-governance] Policy: ($level)/($category) for ($agent_name)\n($reminder_text)"
}

print '{"decision":"approve"}'
trace-end $_tid
exit 0

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Detect agent name from dispatch context.
# Checks: explicit agent configs in agents/cfg/, subagent_type, keywords in description.
def detect-agent-name [description: string, prompt: string, subagent_type: string] {
    let git_root = (do { git rev-parse --show-toplevel } | complete).stdout | str trim
    let cfg_dir = $"($git_root)/agents/cfg"

    # 1. Check if subagent_type matches a known agent config
    if ($subagent_type != "") {
        let cfg_path = $"($cfg_dir)/($subagent_type).cfg.yaml"
        if ($cfg_path | path exists) {
            return $subagent_type
        }
        # Try with -agent suffix
        let cfg_path2 = $"($cfg_dir)/($subagent_type)-agent.cfg.yaml"
        if ($cfg_path2 | path exists) {
            return $"($subagent_type)-agent"
        }
    }

    # 2. Scan description for known agent names
    if ($cfg_dir | path exists) {
        let cfg_files = try { ls $cfg_dir | where name =~ '.cfg.yaml$' } catch { [] }
        for file in $cfg_files {
            let name = $file.name
                | path basename
                | str replace '.cfg.yaml' ''
            if ($description | str downcase | str contains ($name | str downcase)) {
                return $name
            }
        }
    }

    # 3. Check for common agent type keywords
    let types = ["Explore" "Coder" "Research"]
    for t in $types {
        if $subagent_type == $t { return $"subagent-($t | str downcase)" }
    }

    ""
}

# Write a governance audit event to JSONL.
def emit-governance-event [
    traces_dir: string
    action: string           # "allowed" | "denied" | "warn"
    agent_name: string
    tool_name: string
    reason: string
    pattern: string
] {
    let audit_file = $"($traces_dir)/governance-audit.jsonl"
    let entry = {
        ts: (date now | format date "%Y-%m-%dT%H:%M:%S%z")
        event: "governance.check"
        action: $action
        agent_id: $agent_name
        tool_name: $tool_name
        reason: $reason
        pattern: $pattern
        session_id: (try { session-id } catch { "unknown" })
    }
    try { $entry | to json --raw | $"($in)\n" | save --append $audit_file }
}
