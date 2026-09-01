use std/assert
use ../helpers/checkpoint.nu [pending-repos reset-checkpoint-for-today]

let repos = [{ repo: "alpha" }, { repo: "beta" }]
let terminal_repos = {
    alpha: { status: "COMPLETE" }
    beta: { status: "VERIFIED" }
}

let stale = {
    run_date: "2000-01-01"
    repos: $terminal_repos
}
assert equal ((pending-repos $stale $repos) | length) 2
let reset = (reset-checkpoint-for-today $stale)
assert equal $reset.run_date (date now | format date "%Y-%m-%d")
assert equal ($reset.repos | columns | length) 0

let current = {
    run_date: (date now | format date "%Y-%m-%d")
    repos: $terminal_repos
}
assert equal ((pending-repos $current $repos) | length) 0
