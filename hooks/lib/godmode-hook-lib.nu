# godmode-hook-lib.nu — shared preamble for all godmode hooks.
# Usage: use ../lib/godmode-hook-lib.nu [godmode-hook-context]

# Returns a record with pre-parsed hook context, or null if any precondition fails.
# Fields: input, git_root, project, running, pending, blocked, recent_commits
export def godmode-hook-context [] {
    let input = (try { open --raw /dev/stdin | from json } catch { return null })

    let git_root_result = do { git rev-parse --show-toplevel } | complete
    if $git_root_result.exit_code != 0 { return null }
    let git_root = $git_root_result.stdout | str trim

    let task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"
    if not ($task_file | path exists) { return null }

    let godmode_found = (which godmode | length) > 0
    if not $godmode_found { return null }

    let result = do { godmode context --json } | complete
    if $result.exit_code != 0 { return null }

    let ctx = (try { $result.stdout | str trim | from json } catch { return null })

    let running = ($ctx | get running? | default [])
    let blocked = ($ctx | get blocked? | default [])
    let pending_count = ($ctx | get pending_count? | default 0)

    {
        input: $input
        git_root: $git_root
        project: ($ctx | get project? | default "unknown")
        running: $running
        pending_count: $pending_count
        blocked: $blocked
        recent_commits: ($ctx | get recent_commits? | default [])
    }
}
