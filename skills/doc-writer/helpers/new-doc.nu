#!/usr/bin/env nu
# new-doc.nu — scaffold a new documentation file from a template
#
# Usage:
#   nu skills/doc-writer/helpers/new-doc.nu <type> <name>
#
# Types: readme, claude, architecture, api, skill
#
# Example:
#   nu skills/doc-writer/helpers/new-doc.nu readme my-crate

def main [doc_type: string, name: string] {
    let templates = {
        readme: $"# ($name)\n\nOne-paragraph description of what this is.\n\n## Install\n\n```bash\n# install command\n```\n\n## Quickstart\n\n```bash\n# example command\n```\n\n## Key Concepts\n\n## Usage\n\n## See Also\n",
        claude: $"# CLAUDE.md\n\nThis file provides guidance to Claude Code when working in this repository.\n\n## Build & Test\n\n```bash\n# build\n# test\n# lint\n```\n\n## Architecture\n\n## Conventions\n\n## Constraints\n",
        architecture: $"# Architecture: ($name)\n\n## Overview\n\n## Components\n\n| Component | Owns | Does NOT own |\n| --------- | ---- | ------------ |\n\n## Data Flow\n\n## Key Decisions\n\n| Decision | Rationale | Tradeoff |\n| -------- | --------- | -------- |\n",
        api: $"# API Reference: ($name)\n\n## Overview\n\n## Exports\n\n",
        skill: $"---\nname: \"godmode:($name)\"\ndescription: >\n  TODO\nrequires: []\nnext: []\n---\n\n# ($name | str capitalize)\n\n## When to Use\n\n## Process\n\n1.\n2.\n3.\n\n## Output Format\n\n## Handoff\n",
    }

    if $doc_type not-in $templates {
        print $"Unknown doc type: ($doc_type). Valid types: ($templates | columns | str join ', ')"
        exit 1
    }

    let content = $templates | get $doc_type
    print $content
}
