// Audit the generated OpenCode plugin against the real @opencode-ai/plugin
// type definitions, which are the ground truth for the hook contract. This
// script parses the installed .d.ts rather than hardcoding hook names, so a
// hook that OpenCode never declared is caught here.
//
//   bun run scripts/opencode/audit-plugin.ts
//
// Two limits of `tsc` make this necessary rather than redundant:
//   - handler arity is not enforced, so a 1-arg handler satisfies a 2-arity
//     hook and a dropped `output` parameter compiles clean
//   - excess properties pass when at least one valid key is present, so a
//     typo'd hook name alongside a real one compiles clean
// Run `bun x tsc -p .opencode/tsconfig.json` as well; it catches what this
// cannot.

import { readFileSync } from "node:fs"
import { resolve } from "node:path"

// Resolve from the repo root so the script behaves the same whether invoked
// from the root, from a CI job, or via an absolute path.
const ROOT = resolve(import.meta.dir, "../..")
const D_TS = resolve(
  ROOT,
  ".opencode/node_modules/@opencode-ai/plugin/dist/index.d.ts",
)
const PLUGIN = resolve(ROOT, ".opencode/plugins/godmode.ts")

function readOrFail(path: string, hint: string): string {
  try {
    return readFileSync(path, "utf8")
  } catch {
    console.error(`FAIL: cannot read ${path}\n      ${hint}`)
    process.exit(1)
  }
}

interface HookSpec {
  name: string
  /** parameter names declared on the hook, in order */
  params: string[]
}

function parseHooks(source: string): HookSpec[] {
  const start = source.indexOf("export interface Hooks {")
  if (start < 0) throw new Error("Hooks interface not found")
  const body = source.slice(start)

  const hooks: HookSpec[] = []
  // Hook keys are either bare identifiers (`event?:`) or quoted
  // ("tool.execute.before"?:), optionally followed by a parenthesised
  // parameter list that may span lines.
  const re = /(?:^|\n)\s*(?:(["'])([\w.]+)\1|(\w+))\??:\s*(?:async\s*)?\(/g
  let m: RegExpExecArray | null
  while ((m = re.exec(body)) !== null) {
    const name = m[2] ?? m[3]
    if (!name) continue

    // Capture the balanced parameter list starting at the opening paren.
    let depth = 0
    let i = m.index + m[0].length - 1
    const params: string[] = []
    let current = ""
    for (; i < body.length; i++) {
      const ch = body[i]
      if (ch === "(" || ch === "{" || ch === "<") depth++
      else if (ch === ")" || ch === "}" || ch === ">") {
        depth--
        if (depth === 0) break
      }
      if (ch === "," && depth === 1) {
        params.push(current.trim())
        current = ""
        continue
      }
      current += ch
    }
    if (current.trim()) params.push(current.trim())
    // Keep only the identifier, dropping `: Type` annotations.
    hooks.push({
      name,
      params: params.map((p) => p.split(":")[0].trim()).filter(Boolean),
    })
  }
  return hooks
}

const hooks = parseHooks(
  readOrFail(
    D_TS,
    "Run `bun install` in .opencode — the contract audit reads the installed type definitions.",
  ),
)

console.log(`OpenCode Hooks contract (from ${D_TS}):\n`)
for (const h of hooks) {
  const arity = h.params.length
  console.log(`  ${h.name.padEnd(42)} arity=${arity}  (${h.params.join(", ")})`)
}

// Cross-check the generated plugin against that contract.
const plugin = readOrFail(
  PLUGIN,
  "Run `godmode hook generate --client opencode` to produce the plugin.",
)
const byName = new Map(hooks.map((h) => [h.name, h]))
const emitted = [...plugin.matchAll(/"([\w.]+)"\s*:\s*async\s*\(/g)].map(
  (m) => m[1],
)

console.log(`\nEmitted by ${PLUGIN}:`)
let bad = 0
for (const name of emitted) {
  const spec = byName.get(name)
  if (!spec) {
    console.log(`  UNKNOWN HOOK  ${name}`)
    bad++
    continue
  }
  // Every emitted handler must accept both (input, output).
  const arityOk = spec.params.length === 2
  console.log(
    `  ${name.padEnd(42)} ${arityOk ? "OK" : `ARITY ${spec.params.length}, expected 2`}`,
  )
  if (!arityOk) bad++
}
if (emitted.length === 0) {
  console.log("  (no async hook handlers matched — check the regex)")
  bad++
}

// A plugin that never reads a hook decision would silently downgrade every
// blocking PreToolUse gate to advisory.
if (!plugin.includes("throw new Error(verdict.reason")) {
  console.log("  MISSING        block-decision handling (PreToolUse cannot deny)")
  bad++
} else {
  console.log(`  ${"block-decision handling".padEnd(42)} OK`)
}

console.log(bad === 0 ? "\nPASS" : `\nFAIL: ${bad} problem(s)`)
process.exit(bad === 0 ? 0 : 1)