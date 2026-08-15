---
name: devloop-session-primer
description: Dispatch the devloop-session-primer agent to produce a fast pre-flight briefing for a devloop dev session — git state, GKG health, pending doob todos, and recent CI status in one pass. Use at the start of a devloop session, or to quickly re-orient after a context switch back into devloop.
---

# Devloop Session Primer

Dispatch the `devloop-session-primer` agent (haiku, read-only: Read, Glob,
Grep, Bash) whenever work is starting or resuming in the `devloop` project
and a quick orientation is needed before diving in.

## When to use

- Start of a devloop dev session.
- Re-orienting after a context switch away from devloop and back.
- Before picking up work, to confirm git state, GKG health, pending doob
  todos, and recent CI status haven't drifted since last touched.

## What it produces

A single compact briefing, no prose — just the facts needed to start work:
current branch and working-tree status, GKG health, outstanding doob todos
scoped to devloop, and the latest CI run status.

## How to invoke

Use the Agent tool with `subagent_type: devloop-session-primer`. No arguments
required; the agent runs its checks concurrently against the current devloop
checkout.
