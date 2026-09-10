# Design — add-lingua-agent

## Context

The upstream bricks of the stack supply everything needed: the lemmatisation cascade and
the contractual `analyzer_version` (`add-lingua-analysis`), statuses and exposure counters
(`add-lingua-knowledge-model`), cards and FSRS scheduling (`add-lingua-decks-review`), and
the EN→FR pack with its glosses (`add-lingua-data-pack`). This change only wires that
brain, compiled natively, into Claude Code sessions. The upstream exploration settled the
constraints: in-place highlighting inside the TUI is **impossible** (there is no
post-processing of the rendered output); what is viable is the `Stop` hook (it receives
`transcript_path`, a JSONL file), the statusline, a skill and an MCP server. Transcripts
are confidential (employer code) → local-only by default, non-negotiable.

Inherited and not re-litigated here: the `(studied language, lemma)` key without POS and
the exposure counters (`add-lingua-knowledge-model`), FSRS through the pinned `fsrs` crate
(`add-lingua-decks-review`), and determinism at a given `analyzer_version`
(`add-lingua-analysis`).

## Goals / Non-Goals

**Goals:**
- Every Claude Code reply feeds the exposure counters automatically; `/vocab` and the
  statusline surface the state.
- One brain: the same analysis (same `analyzer_version`) produces the same results in the
  extension (WASM) and in the plugin (native).
- The cards in the plugin store stay reviewable (through the agent) and exportable.

**Non-Goals:**
- Syncing the `~/.lingua/` store (transcripts are confidential; syncing the other surfaces
  is `add-lingua-backend` / `add-lingua-connected-clients`, and the plugin is explicitly
  excluded there).
- Adapters for other agents (Codex, Aider, Gemini) — the `SessionSource` trait is the
  contract, the impls come later.
- A dedicated review TUI, or highlighting inside the terminal.

## Decisions

### D1 — The Claude Code plugin: one binary, four attachment points
`lingua` (a Rust binary over native `lingua-core`) plus a plugin manifest: the `Stop` hook
(reads `transcript_path`, extracts the assistant text from the JSONL, ingests the
exposures), the statusline (reads the same transcript, shows `📖 96 % · 3 nouveaux` — the
plugin ships French UI copy), the `/vocab` skill (lists the session's unknown words, their
glosses, adds them to the deck), and the MCP server (`list_decks`, `add_words`,
`due_cards`, `answer_card`). Review on the plugin side goes through the agent itself (a
conversational quiz: `due_cards` → questions → `answer_card`) — no dedicated TUI in v1,
which still leaves the plugin store's cards reviewable and exportable.

### D2 — Local-only by construction
The data lives in `~/.lingua/` (SQLite through `rusqlite`, versioned schema) — **never any
transcript content persisted, only lemmas, counters and the sentences the user explicitly
captures**. The binary opens no network connection; the invariant is tested (and stays
tested through the sync changes, where the plugin is out of scope). The plugin store and
the extension store are **not reconciled** within the local stack (a decision inherited
from the split: a fake local sync would be thrown-away work) — the schemas share
`lingua-core`'s types so that merging them is mechanical the day it matters.

### D3 — The `SessionSource` trait: Claude Code is the first impl, not the only one
Ingestion goes through a `SessionSource` abstraction (locating sessions, extracting the
assistant text), of which Claude Code is the first implementation. A new agent (Codex,
Aider) is added by implementing the trait, without touching the analysis pipeline. The
interactive layer (decks, review) is the MCP server — the only standardised integration,
portable to every agent; rich display (a live statusline) stays Claude-Code-only.

## Risks / Trade-offs

- [The plugin store and the extension store drift apart (two local states)] → accepted and
  inherited from the local stack; merging them is the sync changes' problem (and they
  exclude the plugin — the divergence is documented in their stats UI: "excluding agent
  sessions").
- [Claude Code transcript formats are not contractual (internal JSONL)] → defensive
  parsing, silent degradation (a mute statusline, ingestion skipped), idempotence keyed on
  the transcript offset; tests run on synthetic fixtures rather than real transcripts.
- [The plugin manifest may not be able to install the statusline] → document the manual
  configuration (a dedicated task).

## Migration Plan

Nothing to migrate (a new, local-only deliverable). Rollback = uninstall the plugin; the
`~/.lingua/` store stays on disk and the user can delete it. The versioned SQLite schema
prepares future migrations.

## Open Questions

- How the Claude Code plugin is distributed (plugin marketplace vs git repo) — to be
  settled at delivery, with no impact on the architecture.
