#!/usr/bin/env nu
# analyze-diff.nu — Mechanical diff analysis for godmode:decompose.
#
# Usage:
#   nu analyze-diff.nu --base main --branch feat/my-branch
#   nu analyze-diff.nu --verify-coverage --source feat/my-branch --splits "split-1 split-2"
#
# Output: JSON report on stdout.

def main [
    --base: string = "main"          # Base branch to diff against
    --branch: string = ""            # Branch to analyze (default: current)
    --verify-coverage                # Check that splits cover all source files
    --source: string = ""            # Source branch (for --verify-coverage)
    --splits: string = ""            # Space-separated split branch names (for --verify-coverage)
] {
    if $verify_coverage {
        verify_coverage $source $splits
    } else {
        analyze $base $branch
    }
}

# ── Main analysis ────────────────────────────────────────────────────────────

def analyze [base: string, branch: string] {
    let target = if ($branch | is-empty) {
        do { git branch --show-current } | complete | get stdout | str trim
    } else {
        $branch
    }

    let changed_files = (
        do { git diff $"($base)...($target)" --name-only } | complete
        | get stdout
        | lines
        | where { |f| not ($f | is-empty) }
    )

    if ($changed_files | length) == 0 {
        error make { msg: $"No changed files between ($base) and ($target)" }
    }

    let crate_map   = build_crate_map
    let crate_groups  = group_by_crate $changed_files $crate_map
    let concern_groups = group_by_concern $changed_files
    let coupling      = detect_coupling $changed_files $base $target
    let proposed      = propose_splits $crate_groups $concern_groups $coupling

    {
        source_branch: $target
        base: $base
        total_files: ($changed_files | length)
        files: $changed_files
        crate_groups: $crate_groups
        concern_groups: $concern_groups
        coupling_warnings: $coupling
        proposed_splits: $proposed
    } | to json
}

# ── Crate mapping (cargo metadata) ──────────────────────────────────────────

def build_crate_map [] {
    let meta = do { cargo metadata --no-deps --format-version 1 } | complete

    if $meta.exit_code != 0 {
        # Not a Rust workspace — return empty map
        return {}
    }

    let packages = $meta.stdout | from json | get packages? | default []

    $packages | each { |pkg|
        let manifest = $pkg.manifest_path
        # Derive the crate src root from the manifest path
        let crate_root = $manifest | path dirname
        { name: $pkg.name, root: $crate_root }
    } | reduce -f {} { |entry, acc|
        $acc | insert $entry.name $entry.root
    }
}

def group_by_crate [files: list<string>, crate_map: record] {
    mut groups = {}

    for file in $files {
        let matched_crate = (
            $crate_map
            | items { |name, root|
                if ($file | str starts-with $"($root)/") {
                    $name
                } else {
                    null
                }
            }
            | where { |v| $v != null }
            | first
            | default "workspace-root"
        )

        if ($matched_crate in $groups) {
            let existing = $groups | get $matched_crate
            $groups = ($groups | update $matched_crate ($existing | append $file))
        } else {
            $groups = ($groups | insert $matched_crate [$file])
        }
    }

    $groups
}

# ── Concern classification (path patterns) ──────────────────────────────────

def classify_concern [file: string] -> string {
    match true {
        ($file =~ 'Cargo\.toml$' or $file =~ 'Cargo\.lock$')                  => "deps"
        ($file =~ '^\.github/')                                                  => "ci"
        ($file =~ '/tests/' or $file =~ '^tests/')                              => "tests"
        ($file =~ '/benches/' or $file =~ '^benches/')                          => "benches"
        ($file =~ '\.(md|txt|rst)$' or $file =~ '^docs/')                      => "docs"
        ($file =~ '/examples/' or $file =~ '^examples/')                        => "examples"
        ($file =~ '\.nu$' or $file =~ '\.sh$')                                 => "scripts"
        _                                                                        => "logic"
    }
}

def group_by_concern [files: list<string>] {
    mut groups = {}

    for file in $files {
        let concern = classify_concern $file

        if ($concern in $groups) {
            let existing = $groups | get $concern
            $groups = ($groups | update $concern ($existing | append $file))
        } else {
            $groups = ($groups | insert $concern [$file])
        }
    }

    $groups
}

# ── Coupling detection (shared imports / module references) ──────────────────

def detect_coupling [files: list<string>, base: string, target: string] {
    # Extract identifiers imported in the diff: use crate::, mod, pub use, type aliases
    let diff = do { git diff $"($base)...($target)" -- ...$files } | complete | get stdout

    # Find all "use crate::" paths introduced in the diff
    let imports = (
        $diff
        | lines
        | where { |l| ($l | str starts-with "+") and ($l | str contains "use ") }
        | each { |l|
            $l | parse --regex 'use ([\w:]+)' | get capture0? | default []
        }
        | flatten
        | uniq
    )

    if ($imports | length) == 0 {
        return []
    }

    # For each import path, find which changed files reference it
    mut warnings = []

    for import_path in $imports {
        let referencing = $files | where { |f|
            let content = do { git show $"($target):($f)" } | complete
            if $content.exit_code != 0 { return false }
            $content.stdout | str contains $import_path
        }

        if ($referencing | length) >= 2 {
            $warnings = ($warnings | append {
                files: $referencing
                reason: $"shared import: ($import_path)"
            })
        }
    }

    $warnings
}

# ── Split proposal ────────────────────────────────────────────────────────────

def propose_splits [
    crate_groups: record
    concern_groups: record
    coupling: list
] {
    # Primary split axis: crate. Secondary: concern within crate.
    # deps and ci are always their own splits regardless of crate.

    mut splits = []
    mut split_id = 1

    # Always-separate concerns
    let always_separate = ["deps" "ci" "docs" "benches" "examples" "scripts"]

    for concern in $always_separate {
        if ($concern in $concern_groups) {
            let files = $concern_groups | get $concern
            if ($files | length) > 0 {
                $splits = ($splits | append {
                    id: $split_id
                    files: $files
                    crate: "workspace"
                    concern: $concern
                    coupled_to: []
                })
                $split_id = ($split_id + 1)
            }
        }
    }

    # Logic and test splits per crate
    for crate_name in ($crate_groups | columns) {
        let crate_files = $crate_groups | get $crate_name

        let logic_files = $crate_files | where { |f| (classify_concern $f) == "logic" }
        let test_files  = $crate_files | where { |f| (classify_concern $f) == "tests" }

        if ($logic_files | length) > 0 {
            $splits = ($splits | append {
                id: $split_id
                files: $logic_files
                crate: $crate_name
                concern: "logic"
                coupled_to: (find_coupling_ids $logic_files $coupling $splits)
            })
            $split_id = ($split_id + 1)
        }

        if ($test_files | length) > 0 {
            $splits = ($splits | append {
                id: $split_id
                files: $test_files
                crate: $crate_name
                concern: "tests"
                coupled_to: (find_coupling_ids $test_files $coupling $splits)
            })
            $split_id = ($split_id + 1)
        }
    }

    $splits
}

def find_coupling_ids [files: list<string>, coupling: list, existing_splits: list] -> list {
    mut coupled = []

    for warning in $coupling {
        let overlaps = $warning.files | where { |f| $f in $files }
        if ($overlaps | length) > 0 {
            # Find which existing split contains the other file(s)
            let others = $warning.files | where { |f| not ($f in $files) }
            for split in $existing_splits {
                for other in $others {
                    if $other in $split.files {
                        $coupled = ($coupled | append $split.id)
                    }
                }
            }
        }
    }

    $coupled | uniq
}

# ── Coverage verification ─────────────────────────────────────────────────────

def verify_coverage [source: string, splits_str: string] {
    let split_branches = $splits_str | split row " " | where { |s| not ($s | is-empty) }

    let source_files = (
        do { git diff $"main...($source)" --name-only } | complete
        | get stdout | lines | where { |f| not ($f | is-empty) }
    )

    mut covered = []

    for branch in $split_branches {
        let branch_files = (
            do { git diff $"main...($branch)" --name-only } | complete
            | get stdout | lines | where { |f| not ($f | is-empty) }
        )
        $covered = ($covered | append $branch_files)
    }

    let covered_unique = $covered | uniq
    let orphaned = $source_files | where { |f| not ($f in $covered_unique) }
    let duplicated = (
        $covered
        | group-by { |f| $f }
        | items { |file, occurrences| if ($occurrences | length) > 1 { $file } else { null } }
        | where { |v| $v != null }
    )

    {
        source_files: ($source_files | length)
        covered_files: ($covered_unique | length)
        orphaned: $orphaned
        duplicated: $duplicated
        ok: (($orphaned | length) == 0 and ($duplicated | length) == 0)
    } | to json
}
