#!/usr/bin/env nu
# propose.nu — run N OpenAI proposers in parallel, save outputs to .ctx/moa-proposal-<n>.txt
# Usage: nu propose.nu <prompt> [--count <N>]

def main [
    prompt: string
    --count: int = 3   # number of proposer calls
    --model: string = "openai:gpt-4o-mini"
] {
    let root = (git rev-parse --show-toplevel | str trim)
    let workdir = $"($root)/.ctx/_WORKING_DIR"
    mkdir $workdir

    # Clear old proposals
    ls $workdir | where name =~ "moa-proposal-" | each { |f| rm $f.name }

    print $"Dispatching ($count) proposers \(($model)\)..."

    let indices = (seq 1 $count | each { |i| $i })

    $indices | par-each { |i|
        let out_file = $"($workdir)/moa-proposal-($i).txt"
        let result = (do { aichat -m $model $prompt } | complete)
        if $result.exit_code == 0 {
            $result.stdout | save --force $out_file
            print $"  [($i)] done"
        } else {
            print $"  [($i)] FAILED: ($result.stderr)"
            "" | save --force $out_file
        }
    }

    print $"Proposals saved to ($workdir)/moa-proposal-*.txt"
}
