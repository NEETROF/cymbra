# Design — add-lingua-knowledge-model

## Context

The second brick of the Lingua stack, sitting directly on `add-lingua-analysis` (from
which it inherits the `lingua-core` crate, the lemmatisation cascade, the frequency
ranks and the `analyzer_version` contract). The knowledge model is the type vocabulary
every later surface shares — and it is that sharing which will make merging the local
stores mechanical at the sync change (`add-lingua-backend`).

## Goals / Non-Goals

**Goals:**
- The knowledge state per `(studied language, lemma)`: statuses, provenance, implicit
  known by calibration, multi-candidate resolution.
- The pair-keyed L1/L2 profile from day 1 (zero retrofit when a pair is added).
- Frequency-rank calibration: the cold-start primer.
- Exposure counters, the substrate of the future inference.

**Non-Goals:**
- Cards, FSRS and the "known" inference from review (`add-lingua-decks-review` — the
  `srs` provenance field is declared here, wired there).
- Real frequency/gloss data (`add-lingua-data-pack`); any UI; any surface persistence
  (`chrome.storage`, SQLite — each surface stores, the model defines the types).

## Decisions

### D1 — `(studied language, lemma)` key, no POS, four statuses
- The key is the normalised lemma text per studied language; multi-word expressions are
  lemmas with spaces. **POS is not part of the key** in v1: ambiguity (`can` as noun or
  verb) resolves in the learner's favour — known if any candidate is — and the popup
  will show every sense. Rejected alternative: a (lemma, POS) key — it doubles table
  size, requires a tagger (quality/weight), and the pedagogical gain is marginal at the
  level we target.
- Statuses: explicit `learning`, `known`, `ignored`; "new" is the absence of an entry;
  `known` is also **implicit** below the calibration threshold (frequency rank ≤ N).
  `ignored` counts as known in the percentage. An explicit status always wins over
  calibration.
- The `known_source` field (`manual | calibration | srs | import`) prepares the
  Migaku-style SRS inference without wiring it (the `srs` provenance will be written by
  `add-lingua-decks-review`).

### D2 — Frequency-rank calibration: the answer to the cold start
With no primer, day 1 highlights ~60% of the page — the category's documented
drop-off point. A single slider — "I know the N most common words" — makes the page
readable immediately, without marking a single word. Calibration is a *threshold*, not
a bulk write: no explicit status is created (the implicit one is recomputed on every
analysis), so moving the slider is free and reversible. Ranks come from the pack's
frequency table (mini-fixtures here, real wordfreq with `add-lingua-data-pack`).

### D3 — L1/L2 profile: a distinct `native_language`, everything pair-keyed
The profile carries `native_language` (the language of comfort: glosses, future
translations; default: the system locale, later the Cymbra ID locale — the UI locale is
not the language of the glosses) and the studied languages. **Everything downstream is
keyed by pair (L2→L1)** from day 1: glosses, packs, knowledge state, the future MT
direction. MVP = (en→fr) only, but adding (es→fr) is data, not code. The rule "never
analyse the L1" (the language gate) is already carried by `add-lingua-analysis`'s
per-block detection; the profile supplies it the L1.

### D4 — Exposure counters: record without interpreting
Per (studied language, lemma): a counter of occurrences encountered, the source of the
last encounter, a timestamp. In v1 exposure **never modifies a status** — pitfall #1 of
the category: LingQ's auto-known-on-page-turn, explicitly rejected. It is input data for
the future inference ("known" deduced from the SRS/from exposure), fed by agent
ingestion (`add-lingua-agent`) and by reading (`add-lingua-extension-reading`).

### D5 — Interface vocabulary: never "lemma" on screen
The target user does not know the word "lemma" (a direct user lesson, re-learned during
the preshot). The invariant is declared in this capability — the model is what names the
concepts — and applied and linted by every surface in the stack
(`add-lingua-extension-reading`, `add-lingua-agent`): "dictionary form" for the
canonical form, "distinct words" for counts of unique lemmas.

## Risks / Trade-offs

- [No POS: false "knowns" on homographs (`can` noun vs verb)] → accepted in the
  learner's favour; the cost of a tagger (weight, quality, doubled key) exceeds the gain
  at the level we target. Revisitable at pack v2 without breaking the key (POS would
  remain a display attribute).
- [Implicit statuses = a result that depends on the frequency table] → the table is
  versioned with the pack; the *explicit* status always wins, so a pack update never
  overwrites a user's decision.
- [Two local stores (extension / plugin) unreconciled in v1] → accepted (a decision
  inherited from the stack): the sync change will merge them server-side; this change
  supplies exactly the shared type vocabulary that will make that merge mechanical.

## Migration Plan

Nothing to migrate (new module, no surface persistence yet). The types are versioned
serialisable from day 1 — that is the contract the surface stores (extension storage,
plugin SQLite) and the sync will consume.

## Open Questions

- The calibration slider's default (0? 1,000?) on first open — to settle alongside the
  calibration UI (`add-lingua-extension-reading`), with no impact on the model.
- Whether a bulk import primer (LingQ, Anki, …) is worth building at all — deferred
  until real demand; if built, it targets the reserved `import` provenance.
