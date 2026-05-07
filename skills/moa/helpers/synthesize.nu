#!/usr/bin/env nu
# synthesize.nu — read proposals and run synthesizer via aichat (openai)
# Usage: nu synthesize.nu <original-prompt> [--model <model>]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/trace.nu") *
use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [
    original_prompt: string
    --model: string = "openai:gpt-5.2"
] {
    let root = (repo-root)
    let tid = (trace-start "moa" "synthesize.nu" $"model=($model)")
    let workdir = $"($root)/.ctx/_WORKING_DIR"

    let proposals = (
        ls $workdir
        | where name =~ "moa-proposal-"
        | sort-by name
        | each { |f|
            let n = ($f.name | path basename | str replace "moa-proposal-" "" | str replace ".txt" "")
            let text = (open $f.name | str trim)
            $"## Response ($n)\n\n($text)"
        }
        | str join "\n\n---\n\n"
    )

    if ($proposals | is-empty) {
        trace-error $tid 1 "no proposals found in _WORKING_DIR"
        print "No proposals found. Run propose.nu first."
        exit 1
    }

    let synth_prompt = $"You have been provided with responses from several AI models to the following prompt:

**Original prompt:** ($original_prompt)

Here are the responses:

($proposals)

Your task: synthesize these into a single, high-quality response. Critically evaluate each, identify the strongest elements, correct any errors, and produce a unified answer that is better than any individual response."

    print "Running synthesizer..."
    let result = (do { run-external "aichat" "-m" $model $synth_prompt } | complete)
    if $result.exit_code != 0 {
        trace-error $tid $result.exit_code $result.stderr
        print $"ERROR: synthesizer failed: ($result.stderr)"
        exit $result.exit_code
    }

    trace-end $tid
    print $result.stdout
}
