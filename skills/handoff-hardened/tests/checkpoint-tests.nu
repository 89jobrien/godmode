use std/assert
use ../helpers/checkpoint.nu [load-checkpoint-from pending-repos repo-status reset-checkpoint-for-today update-repo]
use ../helpers/discover-dirty-repos.nu [classify-repo]

let repos = [{ repo: "alpha" }, { repo: "beta" }]
let terminal_repos = { alpha: { status: "COMPLETE" }, beta: { status: "VERIFIED" } }
let stale = { run_date: "2000-01-01", repos: $terminal_repos }
assert equal ((pending-repos $stale $repos) | length) 2
let reset = (reset-checkpoint-for-today $stale)
assert equal $reset.run_date (date now | format date "%Y-%m-%d")
assert equal ($reset.repos | columns | length) 0

let current = { run_date: (date now | format date "%Y-%m-%d"), repos: $terminal_repos }
assert equal ((pending-repos $current $repos) | length) 0
assert equal (repo-status { run_date: null, repos: {} } "missing") "PENDING"
let updated = (update-repo { run_date: null, repos: {} } "alpha" { status: "RETRY", error: "network" })
assert equal $updated.repos.alpha.status "RETRY"
assert equal $updated.repos.alpha.attempts 0

let temp = ($env.PWD | path join $".ctx/godmode/_WORKING_DIR/checkpoint-test-(random uuid).json")
"{" | save --force $temp
let recovered = (load-checkpoint-from $temp)
assert equal $recovered.run_date (date now | format date "%Y-%m-%d")
assert equal ($recovered.repos | columns | length) 0
rm $temp

assert equal (classify-repo "clean" "/tmp/clean" {exit_code: 0 stdout: "" stderr: ""} {exit_code: 1 stdout: "" stderr: ""} null) null
let dirty = (classify-repo "dirty" "/tmp/dirty" {exit_code: 0 stdout: " M file" stderr: ""} {exit_code: 1 stdout: "" stderr: ""} null)
assert $dirty.dirty
assert (not $dirty.unpushed)
let ahead = (classify-repo "ahead" "/tmp/ahead" {exit_code: 0 stdout: "" stderr: ""} {exit_code: 0 stdout: "origin/main" stderr: ""} {exit_code: 0 stdout: "2\t3" stderr: ""})
assert equal [$ahead.ahead $ahead.behind $ahead.unpushed] [2 3 true]
let broken = (classify-repo "broken" "/tmp/broken" {exit_code: 128 stdout: "" stderr: "fatal"} {exit_code: 1 stdout: "" stderr: ""} null)
assert equal $broken.error "git status failed: fatal"

let detached = (classify-repo "detached" "/tmp/detached" {exit_code: 0 stdout: "" stderr: ""} {exit_code: 1 stdout: "" stderr: "fatal: HEAD is detached"} null)
assert equal $detached.error "detached HEAD"
let upstream_failed = (classify-repo "upstream" "/tmp/upstream" {exit_code: 0 stdout: "" stderr: ""} {exit_code: 128 stdout: "" stderr: "fatal: transport failure"} null)
assert equal $upstream_failed.error "git upstream failed: fatal: transport failure"
let rev_list_failed = (classify-repo "rev-list" "/tmp/rev-list" {exit_code: 0 stdout: "" stderr: ""} {exit_code: 0 stdout: "origin/main" stderr: ""} {exit_code: 128 stdout: "" stderr: "fatal: bad revision"})
assert equal $rev_list_failed.error "git rev-list failed: fatal: bad revision"
let invalid_counts = (classify-repo "counts" "/tmp/counts" {exit_code: 0 stdout: "" stderr: ""} {exit_code: 0 stdout: "origin/main" stderr: ""} {exit_code: 0 stdout: "not-counts" stderr: ""})
assert equal $invalid_counts.error "git rev-list returned invalid counts"
