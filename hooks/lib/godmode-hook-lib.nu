# godmode-hook-lib.nu — shared preamble for all godmode hooks.
# Usage: use ../lib/godmode-hook-lib.nu [godmode-hook-context]

# Returns a record with pre-parsed hook context, or null if any precondition fails.
# Fields: input (raw stdin record), git_root, tasks, running, pending, blocked
export def godmode-hook-context [] {
    let input = (try { open --raw /dev/stdin | from json } catch { return null })

    let git_root_result = do { git rev-parse --show-toplevel } | complete
    if $git_root_result.exit_code != 0 { return null }
    let git_root = $git_root_result.stdout | str trim

    let task_file = $"($git_root)/.ctx/GODMODE.tasks.yaml"
    if not ($task_file | path exists) { return null }

    let godmode_found = (which godmode | length) > 0
    if not $godmode_found { return null }

    let result = do { godmode task list --json } | complete
    if $result.exit_code != 0 { return null }

    let tasks = (try { $result.stdout | str trim | from json } catch { return null })
    let tasks_type = ($tasks | describe)
    if not ($tasks_type | str starts-with "list") and not ($tasks_type | str starts-with "table") {
        return null
    }

    let running = ($tasks | where { |t| ($t | get status? | default "") == "running" })
    let pending = ($tasks | where { |t| ($t | get status? | default "") == "pending" })
    let blocked = ($tasks | where { |t| ($t | get status? | default "") == "blocked" })

    {
        input: $input
        git_root: $git_root
        tasks: $tasks
        running: $running
        pending: $pending
        blocked: $blocked
    }
}
