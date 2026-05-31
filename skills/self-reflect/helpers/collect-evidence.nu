#!/usr/bin/env nu
# self-reflect/helpers/collect-evidence.nu
# Gather session evidence for reflection. Run manually or as reference.

print "--- Recent commits (24h) ---"
let commits = do { git log --oneline --since="24 hours ago" } | complete
if $commits.exit_code == 0 {
    print $commits.stdout
} else {
    print "(no commits)"
}

print "\n--- Task graph state ---"
let tasks = do { godmode task list } | complete
if $tasks.exit_code == 0 {
    print $tasks.stdout
} else {
    print "(no task graph)"
}

print "\n--- Working dir scratch files ---"
let scratch = do { ls .ctx/_WORKING_DIR/ } | complete
if $scratch.exit_code == 0 {
    print $scratch.stdout
} else {
    print "(no scratch files)"
}

print "\n--- Stash list ---"
let stash = do { git stash list } | complete
if $stash.exit_code == 0 and ($stash.stdout | str trim | str length) > 0 {
    print $stash.stdout
} else {
    print "(no stashes)"
}
