#!/usr/bin/env nu
# 001-add-hook-log-dir.nu — ensure .ctx/ dir exists for hook logging
let git_root = (git rev-parse --show-toplevel | str trim)
let ctx_dir = $"($git_root)/.ctx"
if not ($ctx_dir | path exists) {
    mkdir $ctx_dir
    print "migration 001: created .ctx/"
} else {
    print "migration 001: .ctx/ already exists (skipped)"
}
