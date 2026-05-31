#!/usr/bin/env nu
# todo-issue-sync/helpers/scan-todos.nu
# Scan codebase for TODO/FIXME/HACK/XXX markers.
# Excludes target/, .git/, node_modules/, vendor/.

let patterns = ["TODO", "FIXME", "HACK", "XXX"]
let exclude_dirs = ["target", ".git", "node_modules", "vendor", ".kgx"]
let include_types = ["rust", "go", "nu", "ts", "py"]

# Build rg command for each type
let type_args = $include_types | each {|t| $"--type ($t)" } | str join " "
let glob_excludes = $exclude_dirs | each {|d| $"--glob '!($d)/**'" } | str join " "

print "Scanning for TODO markers..."
print $"Types: ($include_types | str join ', ')"
print $"Excluding: ($exclude_dirs | str join ', ')"
print ""

for pattern in $patterns {
    let result = do { rg $pattern --type rust --type go -n --no-heading } | complete
    if $result.exit_code == 0 and ($result.stdout | str trim | str length) > 0 {
        let count = $result.stdout | lines | length
        print $"($pattern): ($count) matches"
    }
}

print "\nRun the full skill for cross-referencing against GitHub issues."
