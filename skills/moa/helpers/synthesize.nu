#!/usr/bin/env nu
# synthesize.nu — read proposals and run synthesizer via aichat (openai)
# Usage: nu synthesize.nu <original-prompt> [--model <model>]

def main [
    original_prompt: string
    --model: string = "openai:gpt-4o"
] {
    let root = (git rev-parse --show-toplevel | str trim)
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
        print "No proposals found. Run propose.nu first."
        exit 1
    }

    let synth_prompt = $"You have been provided with responses from several AI models to the following prompt:

**Original prompt:** ($original_prompt)

Here are the responses:

($proposals)

Your task: synthesize these into a single, high-quality response. Critically evaluate each, identify the strongest elements, correct any errors, and produce a unified answer that is better than any individual response."

    print "Running synthesizer..."
    aichat -m $model $synth_prompt
}
