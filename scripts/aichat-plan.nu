#!/usr/bin/env nu
# aichat-plan.nu — send a planning prompt to aichat using the project default model
#
# Usage:
#   nu scripts/aichat-plan.nu "Your prompt here"
#   echo "Your prompt" | nu scripts/aichat-plan.nu
#   nu scripts/aichat-plan.nu --mini "Lightweight task"
#
# Models:
#   default  → openai:gpt-5.2
#   --mini   → openai:gpt-5.2-mini

def main [
    prompt?: string  # Prompt text (or pipe via stdin)
    --mini           # Use gpt-5.2-mini instead of gpt-5.2
    --model: string  # Override model entirely
] {
    let m = if $model != null {
        $model
    } else if $mini {
        "openai:gpt-5.2-mini"
    } else {
        "openai:gpt-5.2"
    }

    let text = if $prompt != null {
        $prompt
    } else {
        $in | str trim
    }

    if ($text | is-empty) {
        error make { msg: "No prompt provided. Pass as argument or pipe via stdin." }
    }

    aichat --model $m $text
}
