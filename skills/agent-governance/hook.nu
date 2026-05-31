#!/usr/bin/env nu
# agent-governance/hook.nu — PreToolUse/Agent hook
# Warns if dispatching a subagent without a governance policy file in the repo.
# Checks for governance-policy.yaml or .ctx/governance-*.yaml.
# Always exits 0.

let input = open --raw /dev/stdin | from json

let git_result = do { git rev-parse --show-toplevel } | complete
if $git_result.exit_code != 0 {
    exit 0
}

let git_root = $git_result.stdout | str trim

# Check for any governance policy file
let has_policy = (
    ($"($git_root)/governance-policy.yaml" | path exists)
    or (try {
        ls $"($git_root)/.ctx/"
        | where name =~ "governance"
        | length
        | $in > 0
    } catch { false })
)

if not $has_policy {
    eprintln "[godmode:agent-governance] Dispatching agent without a governance policy — see /godmode:agent-governance"
}

exit 0
