# add-lingua-decks-review — Cymbra Lingua: decks, cards and FSRS review in the core

## Why

`add-lingua-knowledge-model` can say what the user knows; nothing yet helps them learn
what they do not. This change adds the learning brick to the core: decks and cards with
provenance (the originating sentence, capturable only at the moment of the encounter),
FSRS review scheduling, and full backup/restore of the state — the safety net of the
local phase (before the sync, the state lives in a single browser profile). Anki-format
export is deferred to a later change: the card schema, designed field-by-field
exportable from day 1, keeps it cheap. Everything stays pure `lingua-core` logic,
host-tested: the UI surfaces (side panel, drawer, container app) arrive in the following
changes and consume this engine as is.

**Position in the stack (3/12)**: add-lingua-analysis → add-lingua-knowledge-model →
**add-lingua-decks-review** → add-lingua-data-pack → add-lingua-wasm →
add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox →
add-lingua-apple → add-lingua-agent → add-lingua-backend →
add-lingua-connected-clients. **Explicit prerequisite: `add-lingua-knowledge-model`**
(statuses, `srs` provenance, the (language, lemma) key — moving a word to "known" from
review writes into the knowledge model).

## What Changes

- **`decks/` module in `crates/lingua-core`**: a versioned serialisable card schema —
  lemma, encountered form, originating context sentence, source of the encounter (URL or
  agent session identifier, timestamp), gloss, an optional media slot (`media`, with
  `source: capture|stock|generated` and `sync_policy` — not populated in this change
  but present in the schema). Multi-word expressions are cards in their own right.
- **FSRS integration** (the `fsrs` crate, version pinned): `again/hard/good/easy`
  grading, due dates, a due-card count computable at any moment; parameters stored on
  the state.
- **"I know this" during review** → `known` status with provenance `srs` in the
  knowledge model, the card kept out of the queue (history intact).
- **Backup/restore**: full export of the state to a versioned file (cards, statuses,
  calibration, FSRS parameters) and identical restore — never a silent loss.
  (Anki-format export: deferred, the schema stays field-by-field serialisable.)
- The "review right next to the reading" requirement (a visible due count, a session
  launchable from the side panel / injected panel, the answer hidden) is laid down here
  as the capability's contract; its surfaces are delivered by
  `add-lingua-extension-review` (then `add-lingua-apple`), which consume this engine.

## Capabilities

### New Capabilities
- `lingua-decks-review`: decks and cards — a card is lemma + encountered form +
  originating sentence + source + optional media (day-1 schema, image capture deferred),
  FSRS review, lossless backup/restore (Anki export deferred), multi-word expressions,
  review reachable right next to the reading.

### Modified Capabilities
_None. `lingua-knowledge-model` is consumed as is (statuses and the `srs` provenance)._

## Impact

- **Products**: Lingua (a new core module — nothing consumed outside the Lingua stack);
  **Cymbra ID / Music / Live / back office / site: untouched** (no proto, no backend
  crate, no existing app modified).
- **Tree**: `crates/lingua-core` (the `decks/` module) only — no new `apps/*`/`packages/*`
  unit.
- **CI**: covered by the existing Rust lane (fmt/clippy/llvm-cov ≥ 80%), already wired
  to the crate by `add-lingua-analysis`; no new lane, nothing to add to `ci-units`.
- **New dependencies**: `fsrs` (Rust), version pinned.
- **Out of scope**: every review UI (`add-lingua-extension-review`,
  `add-lingua-apple`), image capture on cards (the `media` schema is ready, the capture
  is deferred), card sync (`add-lingua-backend` / `add-lingua-connected-clients`).
