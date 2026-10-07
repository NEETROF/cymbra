# Cymbra Lingua — Claude Code plugin

Track your English and Spanish vocabulary from your AI-agent sessions, **locally**. Every
Claude Code reply feeds per-word exposure counters, in the language it is written in; a
statusline shows the last reply's coverage; `/vocab` surfaces the session's unknown words; an
MCP server lets Claude manage and quiz your decks. Same analysis brain (same analyser
versions) as the browser extension, compiled natively via `lingua-core`.

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
- `lingua statusline` — prints `📖 96 % · 3 nouveaux` for the last reply, or `📖 ES 91 % · 5
  nouveaux` when several languages are followed; mute on any failure.
- `lingua vocab --transcript <path> [--add w1,w2] [--language <tag>]` — lists the session's
  unknown words, under their language when several are followed
  (dictionary form, gloss, rarity), or adds a selection to the deck (the source sentence
  is captured onto the card only then).
- `lingua mcp` — an MCP server over stdio: `list_decks`, `add_words`, `due_cards`,
  `answer_card` (FSRS `again/hard/good/easy`). Review is a conversational quiz through the
  agent.

State lives in `~/.lingua/` (SQLite, versioned schema; override the dir with `LINGUA_HOME`).

## Languages: the packs installed

Every pack in `~/.lingua/` is a language the plugin follows (`add-lingua-agent-languages`):
`pack.lingua` is the English one, and copying `es-fr.lingua` beside it adds Spanish. There is no
other setting. Each reply is read in the language the engine finds in it, fenced code left out
of that vote, and counted in that language. `$LINGUA_PACK`, when set, names the only pack
followed. Without a pack the plugin degrades silently (a mute statusline, ingestion skipped).

The plugin follows one native language — the language the glosses are written in
(`generalise-lingua-native-language`): `pack.lingua`'s, or the first readable pack's when there is
no readable `pack.lingua`. A pack glossed in another language (`es-en.lingua` beside the French
packs) is skipped, as a pack that does not load is; `lingua vocab` names it after its listing, while
the hook and the statusline stay silent.

To get the packs, build the extension's real ones (`yarn gen:pack:real` in
`apps/lingua-extension`, then copy `assets/packs/en-fr.lingua` to `~/.lingua/pack.lingua` and
`assets/packs/es-fr.lingua` beside it), or a tiny test pack with
`scripts/lingua-data/build.sh --testdata en-fr ~/.lingua/pack.lingua`.

With several languages, each MCP tool takes a `language` (`en`, `es`): `add_words`,
`due_cards` and `answer_card` refuse a call without one, so a review never mixes languages;
`list_decks` counts each language.

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
