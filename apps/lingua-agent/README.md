# Cymbra Lingua — Claude Code plugin

Track your English vocabulary from your AI-agent sessions, **locally**. Every Claude Code
reply feeds per-word exposure counters; a statusline shows the last reply's coverage;
`/vocab` surfaces the session's unknown words; an MCP server lets Claude manage and quiz
your deck. Same analysis brain (same `analyzer_version`) as the browser extension,
compiled natively via `lingua-core`.

**Local-only by construction** (`add-lingua-agent`, design D2): no transcript content is
ever persisted — only dictionary forms, counters, timestamps, and the sentences you
explicitly capture through `/vocab`. The binary opens **no network connection** (a tested
invariant).

## The `lingua` binary

A single Rust binary (`apps/lingua-agent/rust`, built by the workspace) with four
subcommands:

- `lingua ingest` — reads the Claude Code `Stop`-hook JSON on stdin (or `--transcript
  <path>`), extracts the assistant text, and increments exposure counters. Idempotent on
  the transcript byte offset, so re-runs over a growing transcript only ingest new turns.
- `lingua statusline` — prints `📖 96 % · 3 nouveaux` for the last reply; mute on any failure.
- `lingua vocab --transcript <path> [--add w1,w2]` — lists the session's unknown words
  (dictionary form, gloss, rarity), or adds a selection to the deck (the source sentence
  is captured onto the card only then).
- `lingua mcp` — an MCP server over stdio: `list_decks`, `add_words`, `due_cards`,
  `answer_card` (FSRS `again/hard/good/easy`). Review is a conversational quiz through the
  agent.

State lives in `~/.lingua/` (SQLite, versioned schema; override the dir with `LINGUA_HOME`).
The analysis pack is read from `$LINGUA_PACK` or `~/.lingua/pack.lingua`; without it the
plugin degrades silently (a mute statusline, ingestion skipped) — build one with
`scripts/lingua-data/build.sh --testdata en-fr ~/.lingua/pack.lingua`.

## Installing the plugin

`.claude-plugin/plugin.json` + `hooks/hooks.json` (the `Stop` hook → `lingua ingest`),
`.mcp.json` (the `lingua mcp` server) and `commands/vocab.md` (`/vocab`) target the Claude
Code plugin format. Put the built `lingua` binary on your `PATH` (`cargo install --path
apps/lingua-agent/rust`), then install the plugin directory.

**Statusline** is a user setting rather than a plugin capability, so configure it manually
in `~/.claude/settings.json` (task 1.7):

```json
{ "statusLine": { "type": "command", "command": "lingua statusline" } }
```

## Scope

Claude Code is the first `SessionSource`; Codex/Aider/Gemini adapters implement the same
trait later. The `~/.lingua/` store is **not** synced (transcripts are confidential) — the
sync changes cover the extension and the app, not the plugin.
