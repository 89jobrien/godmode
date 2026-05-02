---
name: "godmode:moa"
description: >
  Mixture of Agents — run multiple OpenAI proposers in parallel, synthesize into one
  high-quality response. Use when a question benefits from diverse model perspectives
  or when a single model response feels insufficient.
---

# Mixture of Agents (MoA)

Parallel proposers → single synthesizer. Based on the MoA paper (arXiv 2603.08640).

## When to Use

- Complex questions where diverse perspectives improve quality
- Creative tasks (brainstorming, writing, design decisions)
- Analysis tasks where correlated errors are a risk
- Any prompt where "what would multiple models say?" adds value

## How It Works

1. **Propose** — N OpenAI models run the same prompt in parallel (`gpt-4o-mini` x3 by default)
2. **Synthesize** — one stronger model (`gpt-4o`) sees all proposals and produces a unified answer

## Usage

### Quick (Claude orchestrates inline)

Paste the prompt, then run:

```bash
nu skills/moa/helpers/propose.nu "<your prompt>" --count 3
nu skills/moa/helpers/synthesize.nu "<your prompt>"
```

### Custom proposer count or models

```bash
# More proposers
nu skills/moa/helpers/propose.nu "<prompt>" --count 5

# Different proposer model
nu skills/moa/helpers/propose.nu "<prompt>" --model openai:gpt-4o

# Different synthesizer
nu skills/moa/helpers/synthesize.nu "<prompt>" --model openai:o4-mini
```

### From a file

```bash
let prompt = (open prompt.txt | str trim)
nu skills/moa/helpers/propose.nu $prompt
nu skills/moa/helpers/synthesize.nu $prompt
```

## Proposer Pool (default)

| Role        | Model                | Count |
| ----------- | -------------------- | ----- |
| Proposers   | `openai:gpt-4o-mini` | 3     |
| Synthesizer | `openai:gpt-4o`      | 1     |

## Output

Proposals written to `.ctx/moa-proposal-<n>.txt`. Synthesizer output printed to stdout.

## Requirements

- `aichat` installed and configured with OpenAI key (`/opt/homebrew/bin/aichat`)
- OpenAI key in `~/Library/Application Support/aichat/config.yaml`
