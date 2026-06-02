# godmode-hook-lib.nu — shared preamble for all godmode hooks.
# Usage: use ../lib/godmode-hook-lib.nu [godmode-hook-context emit-trace]

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

# ---------------------------------------------------------------------------
# Tracing — shared JSONL activity log
# ---------------------------------------------------------------------------

# Resolve the git root, or return pwd as fallback.
def trace-git-root [] {
    let r = do { git rev-parse --show-toplevel } | complete
    if $r.exit_code == 0 { $r.stdout | str trim } else { $env.PWD }
}

# Get the current HEAD short hash, or "none" if unavailable.
def trace-git-hash [] {
    let r = do { git rev-parse --short HEAD } | complete
    if $r.exit_code == 0 { $r.stdout | str trim } else { "none" }
}

# Truncate a string to at most N characters.
def truncate [n: int] {
    let s = ($in | into string)
    if ($s | str length) <= $n { $s } else { $s | str substring 0..$n }
}

# Append a trace record to `.ctx/traces/activity.jsonl`.
#
# Parameters:
#   --name       : identifier (e.g. "pre-commit", "gm-brainstorm-agent")
#   --kind       : "hook", "agent", or "skill"
#   --status     : "ok", "error", or "skipped"
#   --output     : full output string (first/last 100 chars are extracted)
#   --last-hash  : git hash before the operation ran
#   --parent-task: optional parent task id
#   --child-tasks: optional list of child task ids
#   --tool-calls : number of tool calls made
#   --tools      : list of tool names used
#   --hooks      : list of hook names used
#   --skills     : list of skill names used
#   --agents     : list of agent names used
export def emit-trace [
    --name: string
    --kind: string = "hook"
    --status: string = "ok"
    --output: string = ""
    --last-hash: string = ""
    --parent-task: string = ""
    --child-tasks: list<string> = []
    --tool-calls: int = 0
    --tools: list<string> = []
    --hooks: list<string> = []
    --skills: list<string> = []
    --agents: list<string> = []
] {
    let root = trace-git-root
    let now = (date now | format date "%Y-%m-%dT%H:%M:%S%z")
    let current_hash = trace-git-hash
    let last = if ($last_hash | is-empty) { $current_hash } else { $last_hash }

    let first_100 = ($output | truncate 100)
    let last_100 = if ($output | str length) > 100 {
        $output | str substring (($output | str length) - 100)..
    } else {
        $output
    }

    let record = {
        name: $name
        kind: $kind
        status: $status
        last_git_hash: $last
        current_git_hash: $current_hash
        created_at: $now
        updated_at: $now
        parent_task: (if ($parent_task | is-empty) { null } else { $parent_task })
        child_tasks: $child_tasks
        tool_calls_count: $tool_calls
        tools_used: $tools
        first_100_output: $first_100
        last_100_output: $last_100
        hooks_used: $hooks
        skills_used: $skills
        agents_used: $agents
    }

    let trace_dir = $"($root)/.ctx/traces"
    let trace_file = $"($trace_dir)/activity.jsonl"

    # Ensure directory exists
    if not ($trace_dir | path exists) {
        mkdir $trace_dir
    }

    $record | to json --raw | $"($in)\n" | save --append $trace_file
}
