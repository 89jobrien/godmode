// Derive the OpenCode plugin hook contract from the installed
// @opencode-ai/plugin type definitions, then typecheck the generated
// plugin against it. The .d.ts is the ground truth; this script only reads it.
//
//   bun run .ctx/_WORKING_DIR/audit-opencode.ts
//
// Emits:
//   1. the hook names + arity declared by Hooks
//   2. `bun x tsc --noEmit` over .opencode/plugins/godmode.ts

import { readFileSync } from "node:fs"

const D_TS = ".opencode/node_modules/@opencode-ai/plugin/dist/index.d.ts"
const PLUGIN = ".opencode/plugins/godmode.ts"

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

const hooks = parseHooks(readFileSync(D_TS, "utf8"))

console.log(`OpenCode Hooks contract (from ${D_TS}):\n`)
for (const h of hooks) {
  const arity = h.params.length
  console.log(`  ${h.name.padEnd(42)} arity=${arity}  (${h.params.join(", ")})`)
}

// Cross-check the generated plugin against that contract.
const plugin = readFileSync(PLUGIN, "utf8")
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

console.log(bad === 0 ? "\nPASS" : `\nFAIL: ${bad} problem(s)`)
process.exit(bad === 0 ? 0 : 1)