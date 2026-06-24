#!/usr/bin/env nu
# triage.nu — Per-crate rustqual triage for Rust workspaces
#
# Runs rustqual on each workspace crate individually, annotates findings
# with cross-crate context (pub API, CLI dispatch), and prints a ranked
# summary with recommended actions.
#
# Usage (run from workspace root):
#   nu triage.nu                        # scan all crates in current workspace
#   nu triage.nu --crate crates/core    # scan one crate
#   nu triage.nu --json                 # machine-readable output

def main [
    --crate: string = ""    # path to a single crate (default: scan all)
    --json                  # output as JSON instead of table
] {
    let workspace_root = $env.PWD

    let crate_paths = if ($crate | is-empty) {
        let workspace_toml = $"($workspace_root)/Cargo.toml"
        if not ($workspace_toml | path exists) {
            print $"No Cargo.toml found at ($workspace_root). Run from workspace root."
            return
        }

        let members = try {
            open $workspace_toml
            | get -o workspace.members
            | default []
            | each { |m|
                if ($m | str contains "*") {
                    glob $"($workspace_root)/($m)" | each { |p| $p }
                } else {
                    [$"($workspace_root)/($m)"]
                }
            }
            | flatten
            | where { |p| ($p | path exists) and ($"($p)/Cargo.toml" | path exists) }
        } catch {
            []
        }

        if ($members | is-empty) {
            glob $"($workspace_root)/*/Cargo.toml"
            | each { |p| $p | path dirname }
            | where { |p| $p != $workspace_root }
        } else {
            $members
        }
    } else {
        [$"($workspace_root)/($crate)"]
    }

    if ($crate_paths | is-empty) {
        print "No crates found. Check that Cargo.toml has [workspace.members]."
        return
    }

    let results = $crate_paths | each { |crate_path|
        let crate_name = $crate_path | path basename
        let is_binary = ($"($crate_path)/src/main.rs" | path exists)
        let is_lib = ($"($crate_path)/src/lib.rs" | path exists)
        let crate_type = if $is_binary { "binary" } else if $is_lib { "lib" } else { "unknown" }

        print $"  scanning ($crate_name)..."

        let out = (rustqual $crate_path --json --no-fail | complete)

        if $out.exit_code != 0 {
            {crate: $crate_name, path: $crate_path, type: $crate_type,
             score: null, findings: [], error: $out.stderr}
        } else {
            let data = try { $out.stdout | from json } catch { {} }
            let score = $data | get -o summary.quality_score | default null

            # Normalise findings from their separate sections into a flat list
            let dead = ($data | get -o dead_code | default [])
                | each { |f| {code: "DEAD_CODE", file: $f.file, line: $f.line,
                              function: $f.function_name, detail: $f.kind} }

            let tq = ($data | get -o tq_warnings | default [])
                | each { |f|
                    let code = match $f.kind {
                        "untested" => "TQ_UNTESTED"
                        "no_sut"   => "TQ_NO_SUT"
                        _          => $"TQ_($f.kind | str upcase)"
                    }
                    {code: $code, file: $f.file, line: $f.line,
                     function: $f.function_name, detail: $f.kind}
                  }

            let bp = ($data | get -o boilerplate | default [])
                | each { |f| {code: "BOILERPLATE", file: $f.file, line: $f.line,
                              function: ($f | get -o struct_name | default ""),
                              detail: $f.pattern_id} }

            let srp_struct = ($data | get -o srp.struct_warnings | default [])
                | each { |f| {code: "SRP_STRUCT", file: ($f | get -o file | default ""),
                              line: ($f | get -o line | default 0),
                              function: ($f | get -o struct_name | default ""),
                              detail: ($f | get -o detail | default "")} }

            let srp_mod = ($data | get -o srp.module_warnings | default [])
                | each { |f| {code: "SRP_MODULE", file: ($f | get -o file | default ""),
                              line: ($f | get -o line | default 0),
                              function: ($f | get -o name | default ""),
                              detail: ($f | get -o detail | default "")} }

            let violations = ($data | get -o functions | default [])
                | where { |f| ($f | get -o classification | default "") == "violation" }
                | each { |f| {code: "VIOLATION", file: ($f | get -o file | default ""),
                              line: ($f | get -o line | default 0),
                              function: ($f | get -o name | default ""),
                              detail: "logic + calls"} }

            let findings = [$dead, $tq, $bp, $srp_struct, $srp_mod, $violations]
                | flatten

            {crate: $crate_name, path: $crate_path, type: $crate_type,
             score: $score, findings: $findings, error: null}
        }
    }

    let annotated = $results | each { |r|
        let findings = $r.findings | each { |f|
            $f | merge {action: (classify_finding $f $r.type)}
        }
        $r | merge {findings: $findings}
    }

    if $json {
        print ($annotated | to json)
        return
    }

    print_summary $annotated
    print ""
    print_finding_breakdown $annotated
}

def classify_finding [finding: record, crate_type: string] {
    let code = $finding | get -o code | default ""
    let fn_name = $finding | get -o function | default ""

    match $code {
        "DEAD_CODE" => "config: detect_dead_code = false (cross-crate pub API)"
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
        "TQ_NO_SUT"   => "fix: rename test to include function name (low effort)"
        "BOILERPLATE" => "fix: use derive macro (low effort)"
        "ORPHAN_SUPPRESSION" => "fix: remove qual:allow(dry) — use detect_dead_code = false instead"
        "VIOLATION" => {
            let io_boundary = ($fn_name | str contains "handle") or ($fn_name == "run") or ($fn_name == "open") or ($fn_name | str contains "read")
            if $io_boundary { "suppress: qual:allow(iosp) — I/O boundary or integration root" } else { "fix: extract pure logic into separate function" }
        }
        "SRP_MODULE" => "fix: split file into submodules"
        "SRP_STRUCT" => "fix: split struct or extract method cluster"
        _            => "review"
    }
}

def print_summary [results: list] {
    print "# Workspace Triage Summary"
    print ""
    let rows = $results | each { |r|
        let pct = if $r.score == null { "error" } else {
            $"(($r.score * 100) | math round --precision 1)%"
        }
        {Crate: $r.crate, Type: $r.type, Score: $pct, Findings: ($r.findings | length)}
    }
    print ($rows | table)
}

def print_finding_breakdown [results: list] {
    print "# Finding Breakdown by Action"
    print ""

    let all_findings = $results | each { |r|
        $r.findings | each { |f| $f | merge {crate: $r.crate} }
    } | flatten

    if ($all_findings | is-empty) {
        print "No findings."
        return
    }

    for group in ($all_findings | group-by action | transpose key value) {
        let count = ($group.value | length)
        print $"## ($group.key)  ($count) findings"
        print ""
        for f in $group.value {
            let loc = $"($f.file):($f.line)"
            print $"  [($f.crate)]  ($loc)  ($f.function)  ($f.detail)"
        }
        print ""
    }
}
