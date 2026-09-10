# add-lingua-knowledge-model — Cymbra Lingua: lexical knowledge state

## Why

The analysis pipeline (`add-lingua-analysis`) knows how to produce lemmas; what is
missing is **what the user knows about them**. The knowledge model is the product's
central contract: it is what the decks, the extension, the agent plugin and — later —
the sync all consume, and its `(studied language, lemma)` key plus its statuses are
what makes the count honest (the differentiator against LingQ/Readlang, which count
surface forms). This change also delivers the two answers to the cold start — without
them, day 1 highlights 60% of the page: **frequency-rank calibration** and the **LingQ
import** (which doubles as an acquisition weapon: "migrate from LingQ, keep your
history").

**Position in the stack** (12 changes, implementation order): **2/12.** Explicit
prerequisite: **add-lingua-analysis** (the `lingua-core` crate, lemmatisation,
frequency ranks, `analyzer_version`). Then: add-lingua-decks-review →
add-lingua-data-pack → add-lingua-wasm → add-lingua-extension-reading →
add-lingua-extension-review → add-lingua-firefox → add-lingua-apple →
add-lingua-agent → add-lingua-backend → add-lingua-connected-clients.

## What Changes

- **`knowledge/` module of `crates/lingua-core`**: explicit statuses (`learning`,
  `known`, `ignored`; "new" = no entry) with provenance
  (`manual`/`calibration`/`srs`/`import`), implicit "known" below the calibration
  threshold (frequency rank ≤ N), multi-candidate resolution in the learner's favour
  (known if any candidate is).
- **L1/L2 profile**: `native_language` (the language of comfort) kept distinct from the
  studied languages; every API keyed by pair (L2→L1). The MVP ships only
  (English → French), but adding a pair is data, not code.
- **LingQ import (CSV)**: imported entries are lemmatised, then marked `known` with
  provenance `import` — the cold start for the target segment.
- **Exposure counters** per (language, lemma): occurrences encountered, source,
  timestamp — with no effect on statuses in v1 (input data for the future
  Migaku-style inference).
- **The UI vocabulary invariant** is laid down in this change: the word "lemma" never
  appears on screen ("dictionary form", "distinct words") — every later surface in the
  stack applies and lints it.

## Capabilities

### New Capabilities
- `lingua-knowledge-model`: the knowledge state per lemma and per studied language —
  statuses (new/learning/known/ignored), "known" inferable from the SRS, frequency-rank
  calibration at startup, LingQ import (CSV), L1/L2 profile (native language ≠ studied
  language, everything keyed by pair), exposure counters.

### Modified Capabilities
_None. This change stays local to the `lingua-core` crate: it neither consumes nor
modifies `id-*`/`platform-*`._

## Impact

- **Products**: Lingua (new); **Cymbra ID / Music / Live / back office / site:
  untouched** (no proto, no backend crate, no existing app modified).
- **Tree**: `crates/lingua-core` only (the `knowledge/` module already laid out by
  `add-lingua-analysis`); no new unit — the existing Rust CI lane already covers the
  crate.
- **Dependencies**: none new (minimal CSV parsing; `serde` is already there).
- **Out of scope**: cards/FSRS and the "known" inference from the SRS
  (`add-lingua-decks-review`), the real frequency/gloss pack
  (`add-lingua-data-pack`), the surfaces that display this data (extension, Apple app,
  plugin — later changes), multi-store reconciliation (`add-lingua-backend`).
