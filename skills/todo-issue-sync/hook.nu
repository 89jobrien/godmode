#!/usr/bin/env nu
# todo-issue-sync/hook.nu — PostToolUse/Bash hook
# After `gh issue create`, prints the created issue URL and syncs it into the godmode task graph.
# Always exits 0. Degrades gracefully.

let input = open --raw /dev/stdin | from json

let cmd = try { $input.tool_input.command | default "" } catch { "" }
let output = try { $input.tool_response | default "" } catch { "" }

if not ($cmd | str contains "gh issue create") {
    exit 0
}

# Extract URL from output (gh prints the URL on its own line)
let url = $output | lines | where { |l| $l | str starts-with "https://" } | first | default ""

if ($url | is-empty) {
    print "[godmode:todo-issue-sync] Issue created — run `godmode task pull --github` to sync."
    exit 0
}

print $"[godmode:todo-issue-sync] Issue created: ($url)"

# Extract issue number from URL (last path segment)
let issue_num = $url | split row "/" | last | str trim
let task_id = $"gh-($issue_num)"

# Extract title from the gh issue create command
# --title "..." or --title '...'
let title_match = $cmd | parse --regex '--title ["\'](?P<title>[^"\']+)["\']'
let title = if ($title_match | length) > 0 {
    $title_match | get 0.title
} else {
    $"GitHub issue #($issue_num)"
}

# Add to godmode task graph
let result = do { godmode task add $task_id $title } | complete
if $result.exit_code == 0 {
    print $"[godmode:todo-issue-sync] Task ($task_id) added to godmode graph."
} else {
    # May already exist — not fatal
    print $"[godmode:todo-issue-sync] Note: could not add task ($task_id) — ($result.stderr | str trim)"
}

exit 0
