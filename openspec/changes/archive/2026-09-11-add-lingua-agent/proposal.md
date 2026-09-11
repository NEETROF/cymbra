# add-lingua-agent — Cymbra Lingua: the Claude Code plugin (agent-session capture)

## Why

The third market gap the competitive study confirmed — ingesting the developer's daily
corpus (AI agent sessions), which no product exploits — is covered by no earlier stage of
the stack: the core (`lingua-core`) and the browser extension can analyse and review, but
every Claude Code reply stays invisible to the exposure counters. This change delivers the
complete Claude Code plugin, **local-only by construction** (transcripts are employer
code, confidential): the same analysis brain (same `analyzer_version`) as the extension,
wired into the agent's sessions.

**Position in the stack** (12 changes): **10th** — after `add-lingua-apple`, before
`add-lingua-backend`. **Explicit prerequisites: `add-lingua-decks-review`** (and
transitively `add-lingua-analysis` + `add-lingua-knowledge-model`: the lemmatisation
cascade, statuses, FSRS cards) plus `add-lingua-data-pack` (the EN→FR pack for glosses).
**Parallelisable with the extension branch** (`add-lingua-extension-reading` →
`add-lingua-firefox` → `add-lingua-apple`): no dependency in either direction.

## What Changes

- **A new Claude Code plugin** (`apps/lingua-agent`): a `lingua` Rust binary + a `Stop`
  hook (JSONL transcript ingestion), a statusline ("% known · N new"), a `/vocab` skill,
  and an MCP server for deck operations. Ingestion is **local-only by construction**
  (transcripts are confidential). A `SessionSource` architecture keeps it extensible to
  other agents (Codex, Aider — out of scope for this change).
- **A local `~/.lingua/` store** (SQLite, versioned schema): lemmas, counters, cards —
  never any transcript content, only the sentences the user explicitly captures through
  `/vocab`.
- UI vocabulary: the word "lemma" **never** appears in the plugin's user-facing output
  (statusline, `/vocab`, MCP) — the copy says "dictionary form" and "distinct words".

## Capabilities

### New Capabilities
- `lingua-agent-capture`: ingestion of AI agent sessions — the Claude Code hook, the
  statusline, `/vocab`, MCP decks, the `SessionSource` trait, local-only by default.

### Modified Capabilities
<!-- None. The plugin consumes `lingua-analysis`, `lingua-knowledge-model` and
     `lingua-decks-review` (delivered by the earlier changes in the stack) through
     `lingua-core` natively — it does not redeclare them. No `id-*`/`platform-*`
     foundation is touched: no account, no network. -->

## Impact

- **Products**: Lingua (a new plugin deliverable); **Cymbra ID / Music / Live / back
  office: untouched** (no proto, no backend crate, no existing app modified).
- **Tree**: `apps/lingua-agent` (Rust binary + the Claude Code plugin manifest: hooks,
  statusline, skill, MCP).
- **CI**: the existing Rust lane (`cargo --workspace`) covers the binary (fmt/clippy/
  llvm-cov ≥ 80%, logic host-tested); `apps/lingua-agent` is added to the `ci-units`
  filter. The "no 'lemma' in UI copy" lint extends to the plugin's user-facing output.
- **New dependencies**: `rusqlite` (the local store); `lingua-core` is already in the
  workspace.
- **Out of scope**: Codex/Aider/Gemini adapters (the `SessionSource` trait is the
  contract, not the impls), a dedicated review TUI (review goes through the agent), and
  any network sync of the `~/.lingua/` store (`add-lingua-backend` /
  `add-lingua-connected-clients` sync the extension and the app — **not the plugin**,
  restated there).
