use std/assert

let root = ($env.PWD | path join $".ctx/godmode/_WORKING_DIR/skill-trace-test-(random uuid)")
mkdir $root
do { git -C $root init --quiet } | complete | ignore
let pre = ($env.PWD | path join "hooks/scripts/pre-skill-trace.nu")
let post = ($env.PWD | path join "hooks/scripts/post-skill-trace.nu")

def run-hook [script: path, payload: record, cwd: path] {
    let result = (do { $payload | to json --raw | ^nu $script } | complete)
    assert equal $result.exit_code 0
}

do {
    cd $root
    let start = { tool_use_id: "call/1", tool_input: { skill: "godmode:test-skill" } }
    run-hook $pre $start $root
    let marker = ($root | path join ".ctx/godmode/traces/.pending/call_1.trace_id")
    assert ($marker | path exists)
    run-hook $post ($start | merge { tool_response: { is_error: false } }) $root
    assert (not ($marker | path exists))

    let failed = { tool_use_id: "call-2", tool_input: { skill: "godmode:test-skill" } }
    run-hook $pre $failed $root
    run-hook $post ($failed | merge { error: "line 1\nline 2" }) $root

    let events = (open ($root | path join ".ctx/godmode/traces/trace.jsonl") | lines | each { from json })
    assert equal ($events | get event) ["skill.start" "skill.complete" "skill.start" "skill.error"]
    assert equal ($events | last | get exit_code) 1
    assert equal ($events | last | get stderr_tail) "line 1\nline 2"
}

rm -rf $root
