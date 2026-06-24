#!/usr/bin/env nu
# triage.nu — Per-crate rustqual triage for Rust workspaces
#
# Runs rustqual on each workspace crate individually, annotates findings
# with cross-crate context (pub API, CLI dispatch), and prints a ranked
# summary with recommended actions.
#
# Usage:
#   nu triage.nu                        # scan all crates in current workspace
#   nu triage.nu --crate crates/core    # scan one crate
#   nu triage.nu --json                 # machine-readable output

def main [
    --crate: string = ""    # path to a single crate (default: scan all)
    --json                  # output as JSON instead of table
] {
    let workspace_root = $env.PWD

    # Find all crates (dirs with Cargo.toml that are not the workspace root)
    let crate_paths = if ($crate | is-empty) {
        glob "**/Cargo.toml"
        | where { |p| $p != $"($workspace_root)/Cargo.toml" }
        | each { |p| $p | path dirname }
        | sort
    } else {
        [$"($workspace_root)/($crate)"]
    }

    if ($crate_paths | is-empty) {
        print "No crates found. Run from workspace root."
        return
    }

    # Run rustqual on each crate and collect results
    let results = $crate_paths | each { |crate_path|
        let crate_name = $crate_path | path basename
        let is_binary = ($"($crate_path)/src/main.rs" | path exists)
        let is_lib = ($"($crate_path)/src/lib.rs" | path exists)
        let crate_type = if $is_binary { "binary" } else if $is_lib { "lib" } else { "unknown" }

        # Run rustqual --json --no-fail
        let result = do {
            rustqual $crate_path --json --no-fail
            | from json
        } | complete

        if $result.exit_code != 0 {
            {
                crate: $crate_name
                path: $crate_path
                type: $crate_type
                score: null
                findings: []
                error: $result.stderr
            }
        } else {
            let data = $result.stdout | from json
            let findings = if ($data | get -i findings) != null { $data.findings } else { [] }
            {
                crate: $crate_name
                path: $crate_path
                type: $crate_type
                score: ($data | get -i score | default null)
                findings: $findings
                error: null
            }
        }
    }

    # Annotate findings with workspace context
    let annotated = $results | each { |r|
        let findings = $r.findings | each { |f|
            let action = classify_finding $f $r.type
            $f | merge {action: $action}
        }
        $r | merge {findings: $findings}
    }

    if $json {
        $annotated | to json
        return
    }

    # Print summary table
    print_summary $annotated
    print ""
    print_finding_breakdown $annotated
}

# Classify a finding and recommend an action based on crate type
def classify_finding [finding: record, crate_type: string] {
    let code = $finding | get -i code | default ""
    let fn_name = $finding | get -i function | default ""

    match $code {
        "DEAD_CODE" => {
            "config: detect_dead_code = false (cross-crate pub API)"
        }
        "TQ_UNTESTED" => {
            if $crate_type == "binary" {
                if ($fn_name | str contains "dispatch") or ($fn_name == "run") or ($fn_name == "main") {
                    "config: add to ignore_functions (CLI dispatch wiring)"
                } else {
                    "fix: move logic to lib crate + add unit test"
                }
            } else {
                "fix: add unit test in lib crate"
            }
        }
        "TQ_NO_SUT" => {
            "fix: rename test to include function name (low effort)"
        }
        "BOILERPLATE" => {
            "fix: use thiserror derive (low effort)"
        }
        "ORPHAN_SUPPRESSION" => {
            "fix: remove qual:allow(dry) — use detect_dead_code = false instead"
        }
        "VIOLATION" => {
            if ($fn_name | str contains "handle") or ($fn_name == "run") or ($fn_name == "open") or ($fn_name | str contains "read") {
                "suppress: qual:allow(iosp) — I/O boundary or integration root"
            } else {
                "fix: extract pure logic into separate function"
            }
        }
        "SRP_MODULE" => {
            "fix: split file into submodules"
        }
        "SRP_STRUCT" => {
            "fix: split struct or extract method cluster"
        }
        _ => {
            "review"
        }
    }
}

def print_summary [results: list] {
    print "# Workspace Triage Summary"
    print ""

    let rows = $results | each { |r|
        let finding_count = $r.findings | length
        let score_display = if $r.score == null { "error" } else { $"($r.score)%" }
        {
            Crate: $r.crate
            Type: $r.type
            Score: $score_display
            Findings: $finding_count
        }
    }

    print ($rows | table)
}

def print_finding_breakdown [results: list] {
    print "# Finding Breakdown by Action"
    print ""

    # Collect all findings with crate context
    let all_findings = $results | each { |r|
        $r.findings | each { |f|
            $f | merge {crate: $r.crate, crate_type: $r.type}
        }
    } | flatten

    if ($all_findings | is-empty) {
        print "No findings."
        return
    }

    # Group by action
    let grouped = $all_findings | group-by action

    for group in ($grouped | transpose key value) {
        print $"## ($group.key)"
        print ""
        for f in $group.value {
            let location = $f | get -i file | default ""
            let line = $f | get -i line | default ""
            let fn_name = $f | get -i function | default ""
            print $"  [$($f.crate)] ($location):($line) ($fn_name)"
        }
        print ""
    }
}
