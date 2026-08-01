---
description: Rebuild the Graphify code graphs, then work the task graph-first
argument-hint: <task, e.g. "add Steam accounts to sign-in">
allowed-tools: Bash(scripts/graphify.sh:*), Bash(graphify:*)
---
## 1. Refresh the graphs so they match the current working tree

!`scripts/graphify.sh all`

(If the line above did not execute, run `scripts/graphify.sh all` yourself before continuing.)

## 2. Task

$ARGUMENTS

## 3. How to work it — graph first, then read

Before opening any source file, use Graphify to orient and scope. **Tell me which
queries you ran and what they revealed**, then implement.

1. Locate the area: `graphify god-nodes --top 10` and/or `graphify explain "<Symbol>"`.
2. Blast radius before changing anything: `graphify affected "<Symbol>" --depth 2`.
3. Pick the graph — default `graphify-out/graph.json` is **Rust**; for the apps add
   `--graph apps/music/graphify-out/graph.json` (**Flutter**) or
   `--graph apps/back-office/graphify-out/graph.json` (**Vue**).
4. Only once the graph has pointed you to the handful of relevant files, open THOSE
   files (via their `file:line` anchors) to read and implement — no blind grep sweep.

Symbol names must be exact. Graphify **complements** grep and doesn't replace reading;
it tells you *which* files matter so you read 4 instead of 130.
