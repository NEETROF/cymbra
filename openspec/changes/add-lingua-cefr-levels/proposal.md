## Why

Today "level" in Cymbra Lingua exists only as a frequency rank, and a learner
declares what they know through a single "I know the N most common words"
slider. That slider assumes knowledge it never confirms (inflating any
per-level view), and it forces the reader to keep seeing words below their
level highlighted. Learners think in CEFR terms (A1–C2) and want two things:
to see where they stand per level, and to stop being bothered by words they
already know while the app quietly confirms them through reading. The engine
was built for this — per-lemma exposure counters (design D5) and per-`known`
provenance already exist — so the missing pieces are a CEFR level source and
the logic that turns exposure into a confirmed "known".

## What Changes

- Ship a per-lemma **CEFR level** for English in the data pack, sourced from
  CEFR-J Wordlist v1.6 (A1–B2) and Octanove Vocabulary Profile C1/C2 v1.0
  (C1–C2). **BREAKING** for the pack format: adds a level column, bumps the
  pack format and `ANALYZER_VERSION` (the `build.mjs` pack/engine guard already
  refuses a stale pack). Licensing follows the existing kaikki posture (CEFR-J
  citation string; Octanove CC BY-SA 4.0 attribution + share-alike on the
  derived table).
- **Level-based presumed-known**: the learner declares a CEFR level; lemmas
  *below* it resolve to an implicit presumed-known (not highlighted) —
  replacing the frequency-rank calibration threshold for English. Languages
  without CEFR data keep the frequency-band behaviour as a labelled fallback.
- **Exposure-confirmed known**: a new, **caller-driven** engine operation
  promotes a below-level lemma to an explicit `Known` (new
  `KnownSource::Exposure` provenance) once it has been *read* on enough
  distinct days without the learner ever giving it an explicit status. The D5
  invariant is preserved — recording exposure still never changes a status by
  itself; promotion is a separate operation the extension opts into, so the
  agent plugin keeps its v1 behaviour untouched. Four guardrails keep it from
  becoming the category's most-hated auto-known behaviour: only below the
  declared level, spread over distinct days, reversible via its distinct
  provenance, and any explicit status wins.
- **CEFR ladder** on the stats screen: A1→C2, each level showing three tiers —
  presumed / confirmed / to-learn — plus an estimated-position summary. Where a
  language has no CEFR data the ladder degrades to frequency bands, labelled as
  estimated.
- **Level-targeted deck feeding** ("Renforcer un niveau"): pick a level, add N
  words (capped, commonest-first by default), seeded straight into the deck,
  skipping lemmas already tracked or in the deck.
- Out of MVP scope: the `lingua-agent` Claude Code plugin gains no new UI or
  commands. It inherits the shared engine capability but is not wired here.

## Capabilities

### New Capabilities
<!-- None. This extends existing lingua capabilities; requirements are added to their specs via deltas. -->

### Modified Capabilities
- `lingua-data-packs`: the pack format gains an optional per-lemma CEFR level
  section (source, level, licensing metadata); the builder pipeline joins
  CEFR-J + Octanove to lemmas; format version + `ANALYZER_VERSION` bump.
- `lingua-knowledge-model`: presumed-known MAY be expressed as a CEFR level
  when the pack provides levels (else the existing frequency rank); a new
  caller-driven exposure-promotion operation and `KnownSource::Exposure`
  provenance are added (the D5 "record never changes status" invariant stays);
  a per-level `band_stats` primitive reports presumed/confirmed/to-learn counts;
  the `Exposure` record gains the minimal state needed to count distinct days.
- `lingua-decks-review`: a level-targeted seeding operation adds cards for a
  chosen set of lemmas without a real web encounter, using the reserved
  `EncounterSource::Import`; capped and idempotent against tracked/in-deck
  lemmas.
- `lingua-browser-extension`: the popup replaces the frequency slider with a
  CEFR level picker (frequency slider kept as the non-CEFR fallback); the stats
  screen gains the CEFR ladder; highlighting is gated at the declared level and
  above; a "Renforcer un niveau" control drives level-targeted seeding.

## Impact

- **Product**: Cymbra Lingua only. Consumes the shared `lingua-core` engine and
  the existing `id-*` account/sync plumbing (no change there). No impact on
  Music, Live, back-office, or site.
- **Engine (`crates/lingua-core`)**: new `Pack` rank/level-band enumerator over
  the already-private `freq`/`glosses`; `band_stats` folding `resolve_lemma`;
  `KnownSource::Exposure` + caller-driven `promote_by_exposure`; a distinct-day
  field on `Exposure`; `Deck::seed_lemmas` + `Card::seeded`. Level-aware
  `resolve_lemma`. All host-testable, no WASM.
- **Pack (`crates/lingua-pack`, `scripts/lingua-data/`)**: CEFR join step,
  `SOURCES.md` entries + licence guard for CEFR-J/Octanove, format + version
  bump; every declared pack and the fixtures regenerate.
- **WASM (`crates/lingua-wasm`)**: JSON bindings for `bandStats`, `seedBand`,
  `promoteByExposure`, level get/set (thin glue, excluded from coverage).
- **Extension (`apps/lingua-extension`)**: level picker, CEFR ladder on the
  stats screen, level-gated highlighting, "Renforcer un niveau" UI; calls the
  promotion op as pages are read.
- **Compatibility**: `ANALYZER_VERSION` bump means the real gitignored pack must
  be regenerated (`gen:pack:real`); the `build.mjs` guard fails the build
  otherwise. No proto/wire change (statuses already carry provenance; the sync
  protocol's `KnownWordsService` maps `Exposure` onto the existing wire
  provenance).
