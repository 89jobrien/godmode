#!/usr/bin/env nu
# propose.nu — run N OpenAI proposers in parallel, save outputs to .ctx/moa-proposal-<n>.txt
# Usage: nu propose.nu <prompt> [--count <N>]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [
    prompt: string
    --count: int = 3   # number of proposer calls
    --model: string = "openai:gpt-5.2-mini"
] {
    let root = (repo-root)
    let tid = (trace-start "moa" "propose.nu" $"count=($count)" $"model=($model)")
    let workdir = $"($root)/.ctx/_WORKING_DIR"
    mkdir $workdir

    # Clear old proposals
    trace-decision "moa" "propose.nu" "clear_proposals" $"($workdir)/moa-proposal-*.txt"
    ls $workdir | where name =~ "moa-proposal-" | each { |f| rm $f.name }

    print $"Dispatching ($count) proposers \(($model)\)..."

    let indices = (seq 1 $count | each { |i| $i })

    $indices | par-each { |i|
        let out_file = $"($workdir)/moa-proposal-($i).txt"
        let result = (do { run-external "aichat" "-m" $model $prompt } | complete)
        if $result.exit_code == 0 {
            $result.stdout | save --force $out_file
            print $"  [($i)] done"
        } else {
            trace-error $tid $result.exit_code $"proposer ($i): ($result.stderr)"
            print $"  [($i)] FAILED: ($result.stderr)"
            "" | save --force $out_file
        }
    }

    trace-end $tid
    print $"Proposals saved to ($workdir)/moa-proposal-*.txt"
}
