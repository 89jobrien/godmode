#!/usr/bin/env nu
# Conformance suite for the using-doob skill's shell scripts.
# Exercises find-handoffs.sh and reconcile-handoff.sh against synthetic
# fixture trees, including edge cases (macOS "Application Support" paths
# with spaces, exclude-dir handling, maxdepth boundary, sed project-name
# parsing). Also checks the SKILL.md prose against the real ci.sh it
# documents, so doc drift shows up as a failing test, not a support ticket.
#
# Usage: nu run-conformance.nu
# Exit code 0 = all green, 1 = at least one failure (details printed).

def skill-dir [] {
    $env.FILE_PWD | path dirname
}

def make-tmp [] {
    let tmp_root = ($env | get -o CLAUDE_JOB_TMP | default "/tmp")
    let base = ($tmp_root | path join $"doob-skill-conformance-(random int 100000..999999)")
    mkdir $base
    $base
}

# ---- fixture builders -------------------------------------------------

def fixture-basic [root: string] {
    let repo = ($root | path join "foo")
    mkdir ($repo | path join ".ctx")
    mkdir ($repo | path join ".git")
    "content: ok\n" | save -f ($repo | path join ".ctx" "HANDOFF.foo.foo.yaml")
}

def fixture-excluded-dirs [root: string] {
    let t = ($root | path join "bar" "target" "sub")
    mkdir $t
    "should-not-be-found\n" | save -f ($t | path join "HANDOFF.bar.bar.yaml")
    let nm = ($root | path join "bar" "node_modules" "pkg")
    mkdir $nm
    "should-not-be-found\n" | save -f ($nm | path join "HANDOFF.bar.bar.yaml")
}

# macOS app-support-style path: contains a space, sits at exactly maxdepth 4
# (root/Library/Application Support/App/file = depth 4).
def fixture-app-support-space [root: string] {
    let d = ($root | path join "Library" "Application Support" "App")
    mkdir $d
    "content: app-support\n" | save -f ($d | path join "HANDOFF.appsup.appsup.yaml")
}

# One level past maxdepth 4 -- must NOT be found. root/a/b/c/d/file = depth 5.
def fixture-beyond-maxdepth [root: string] {
    let d = ($root | path join "a" "b" "c" "d")
    mkdir $d
    "content: too-deep\n" | save -f ($d | path join "HANDOFF.deep.deep.yaml")
}

# Non-git directory -- script should still list the file but report
# "(not a git repo)" for the repo line rather than erroring out.
def fixture-non-git [root: string] {
    let d = ($root | path join "orphan" ".ctx")
    mkdir $d
    "content: orphan\n" | save -f ($d | path join "HANDOFF.orphan.orphan.yaml")
}

# ---- test cases ---------------------------------------------------------

def run-find-handoffs [root: string] {
    let script = (skill-dir | path join "scripts" "find-handoffs.sh")
    (bash $script $root | complete)
}

def test-cases []: nothing -> list<record> {
    [
        {
            name: "basic discovery finds HANDOFF file in a git repo"
            check: {|out|
                $out.stdout | str contains "HANDOFF.foo.foo.yaml"
            }
        }
        {
            name: "target/ directories are excluded"
            check: {|out|
                not ($out.stdout | str contains "bar/target")
            }
        }
        {
            name: "node_modules/ directories are excluded"
            check: {|out|
                not ($out.stdout | str contains "node_modules")
            }
        }
        {
            name: "macOS 'Application Support' path with a space is found intact"
            check: {|out|
                $out.stdout | str contains "Library/Application Support/App/HANDOFF.appsup.appsup.yaml"
            }
        }
        {
            name: "files beyond maxdepth 4 are not found"
            check: {|out|
                not ($out.stdout | str contains "HANDOFF.deep.deep.yaml")
            }
        }
        {
            name: "non-git directories report '(not a git repo)' instead of erroring"
            check: {|out|
                ($out.exit_code == 0) and ($out.stdout | str contains "HANDOFF.orphan.orphan.yaml") and ($out.stdout | str contains "(not a git repo)")
            }
        }
    ]
}

def run-suite-find-handoffs [] {
    let root = make-tmp
    fixture-basic $root
    fixture-excluded-dirs $root
    fixture-app-support-space $root
    fixture-beyond-maxdepth $root
    fixture-non-git $root

    let out = (run-find-handoffs $root)
    let results = (test-cases | each {|t|
        let ok = (do $t.check $out)
        { name: $t.name, ok: $ok }
    })
    rm -rf $root
    $results
}

# ---- reconcile-handoff.sh project-name parsing -------------------------
# Exercises the sed extraction: HANDOFF.<project>.<rest>.yaml -> <project>
# without invoking the real doob binary (a stub is put on PATH).

def project-name-cases [] {
    [
        { file: "HANDOFF.doob.doob.yaml", expect: "doob" }
        { file: "HANDOFF.maestro-ao.maestro-ao.yaml", expect: "maestro-ao" }
        { file: "HANDOFF.multi.word.project.yaml", expect: "multi" }
        { file: "HANDOFF.cnbl.cannibalizer.state.yaml", expect: "cnbl" }
    ]
}

def run-suite-reconcile [] {
    let root = make-tmp
    let bin = ($root | path join "bin")
    mkdir $bin
    let stub = ($bin | path join "doob")
    (
        "#!/usr/bin/env bash\n"
        + "# stub: echo back the args so the test can assert on them\n"
        + "echo \"STUB_CALL: $*\"\n"
    ) | save -f $stub
    chmod +x $stub

    let script = (skill-dir | path join "scripts" "reconcile-handoff.sh")

    let results = (project-name-cases | each {|c|
        let f = ($root | path join $c.file)
        "content: x\n" | save -f $f
        let new_path = ($"($bin):($env.PATH | str join (char esep))")
        let out = (with-env {PATH: $new_path} { bash $script $f | complete })
        let ok = ($out.stdout | str contains $"-p ($c.expect)")
        { name: $"reconcile project-name parse: ($c.file) -> ($c.expect)", ok: $ok }
    })
    rm -rf $root
    $results
}

# ---- doc-conformance: SKILL.md's ci.sh description vs the real ci.sh ---
# The skill claims doob's CI gate runs `cargo deny check`. Verify that
# against the actual ci.sh so drift becomes a red test, not stale prose.

def run-suite-doc-conformance [] {
    let skill_md = (skill-dir | path join "SKILL.md")
    let real_ci = ("~/dev/doob/ci.sh" | path expand)

    mut results = []

    if not ($real_ci | path exists) {
        $results = ($results | append {
            name: "doc-conformance: ~/dev/doob/ci.sh exists to check against"
            ok: false
        })
        return $results
    }

    let ci_text = (open $real_ci)
    let skill_text = (open $skill_md)

    let skill_claims_deny = ($skill_text | str contains "cargo deny check")
    let ci_has_deny = ($ci_text | str contains "cargo deny")
    let ci_has_audit = ($ci_text | str contains "cargo audit")

    $results = ($results | append {
        name: "SKILL.md's described CI steps match ci.sh's actual steps"
        ok: (not ($skill_claims_deny and $ci_has_audit and (not $ci_has_deny)))
    })

    # Test-runner sanity: the skill must use the workspace-standard nextest form.
    let step_re = 'cargo nextest run --all-features'
    $results = ($results | append {
        name: "SKILL.md documents 'cargo nextest run --all-features'"
        ok: ($skill_text =~ $step_re)
    })

    $results
}

# ---- main ---------------------------------------------------------------

def main [] {
    print "== using-doob conformance suite =="
    let all = (
        (run-suite-find-handoffs)
        | append (run-suite-reconcile)
        | append (run-suite-doc-conformance)
    )

    for r in $all {
        if $r.ok {
            print $"  PASS  ($r.name)"
        } else {
            print $"  FAIL  ($r.name)"
        }
    }

    let fails = ($all | where ok == false)
    let total = ($all | length)
    let passed = $total - ($fails | length)
    print ""
    print $"($passed)/($total) passed"

    if ($fails | length) > 0 {
        exit 1
    }
}
