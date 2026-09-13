## Context

Cymbra Lingua's engine (`crates/lingua-core`) already holds every primitive this feature
needs, just unexposed: `Pack` owns the private `freq`/`glosses`/`lexicon`
(`packs/pack.rs`), `KnowledgeState::resolve_lemma` (`knowledge/state.rs`) resolves a
lemma to explicit-status-or-implicit-calibration-or-new, `ExposureCounters` (design D5,
`knowledge/exposure.rs`) accumulates per-lemma occurrences, and `KnownSource`
(`knowledge/status.rs`) already reserves `Srs`/`Import` provenance slots. There is no CEFR
data anywhere: "level" exists only as a wordfreq rank (1 = commonest). The learner declares
knowledge through a single frequency slider that assumes — never confirms — what it marks
known. This design adds a CEFR level source, turns the slider into a level declaration,
confirms presumed knowledge through reading, and surfaces it as a per-level ladder, all
without touching the agent plugin.

## Goals / Non-Goals

**Goals:**
- Real CEFR levels A1→C2 for English, offline, baked into the pack.
- A level declaration that quiets below-level highlighting and drives a per-level ladder.
- Honest, evidence-based confirmation: exposure promotes a presumed-known to a confirmed
  known only through reading, only below the declared level, reversibly.
- Level-targeted deck seeding.
- Everything host-testable in `lingua-core`; the agent plugin's behaviour is unchanged.

**Non-Goals:**
- No `lingua-agent` UI/commands (no CEFR statusline, no `/vocab` change). It inherits the
  engine capability but is not wired.
- No per-sense CEFR (levels are per lemma; polysemy is accepted for below-level words).
- No new external runtime dependency and no network calls (levels ship in the pack).
- No proto/wire schema change (see the provenance decision).

## Decisions

### D1 — CEFR source: CEFR-J v1.6 (A1–B2) + Octanove C1/C2 (C1–C2)
This is the only licence-clean pairing covering all six levels: CEFR-J permits commercial
use with a citation; Octanove is CC BY-SA 4.0 — the same licence family as the pack's
existing kaikki glosses, so no new licensing posture. Rejected: Cambridge EVP (commercial
prohibited), Oxford (proprietary, no C2), Kelly / CEFRLex (CC BY-NC-SA — the NC the project
bans). CEFR-J tags per headword+POS and can list a lemma at several levels; **collapse rule:
take the lowest (earliest) level per lemma**, so a word first taught at A2 counts as A2.

### D2 — Level lives in the pack; format + `analyzer_version` bump
Add an optional per-lemma level table to the pack container, parallel to `freq`/`glosses`,
present only for pairs with licence-clean CEFR data. This changes the pack format, so
`ANALYZER_VERSION` bumps — which forces regenerating every declared pack + the three
fixtures + the real gitignored pack (`gen:pack:real`). The `build.mjs` guard (#381) already
fails the extension build against a stale pack, so the landmine is caught at build time, not
runtime. Alternative rejected: computing levels in the UI from rank bands — dishonest (no
canonical rank→CEFR table) and it would still need the enumerator in core.

### D3 — Presumed-known = level-aware `resolve_lemma`, reusing `Known(Calibration)`
`resolve_lemma` gains a level dimension: below the declared level → implicit
`Known(Calibration)` (exactly today's "not highlighted, counts as known" behaviour, keyed
on level instead of rank). This means the ladder's **presumed** tier is just implicit
`Known(Calibration)` and the **confirmed** tier is any explicit `Known(_)`. No new status
kind is needed for presumed — the existing `Calibration` provenance already means "assumed,
not proven". For languages without a level table, `resolve_lemma` falls back to the rank
threshold unchanged. The level source is passed in like `FrequencyRanks` is today (a trait),
keeping the module deterministic and host-testable.

### D4 — Exposure→known is a NEW `KnownSource::Exposure`, promoted by an explicit op
`exposure.rs` documents a deliberate v1 refusal: "exposure never changes a status —
LingQ's auto-known-on-page-turn is the most-hated behaviour". We honour that invariant on
`record()` and add a **separate, caller-driven** `promote_by_exposure(...)` that the
extension calls. It promotes only when the lemma is below the declared level, has no
explicit status, and has ≥ N distinct read-days (default N=4). Because an explicit status
always wins in `resolve_lemma`, any user interaction permanently blocks promotion — the
"no interaction" gate is free. Promotions carry `KnownSource::Exposure` so they stay
distinguishable and bulk-reversible. The agent plugin never calls the op, so its behaviour
is untouched. Alternative rejected: promoting inside `record()` — would resurrect exactly
the hated behaviour and change the agent's semantics.

### D5 — Distinct-day counting needs a minimal `Exposure` schema addition
Today `Exposure` = `{ occurrences, last_source, last_seen }`. Raw occurrences over-count
(one article repeating a word 20×). Promotion keys on **distinct UTC days read**, so
`Exposure` gains a small bounded field — a count of distinct recent days (e.g. the last day
stamped + a distinct-day counter, or a tiny fixed-size ring of recent day numbers). Kept
bounded so state stays small and serialises deterministically. In the extension MVP every
exposure is web reading, so all exposures are "read" by construction and no source
discrimination is needed; the read/agent-source distinction is deferred with the agent
integration.

### D6 — Wire provenance: add `"exposure"`, degrade gracefully, no proto break
`Status::wire_provenance`/`from_wire` map provenance as a string (`manual|srs|import`).
Add `"exposure"`. This is a string value, not a proto field change, so `buf breaking` sees
no FILE-level break. Old clients' `from_wire` maps an unknown value to `manual` — a lossy
but safe degrade (an exposure-known still syncs as a known). New clients round-trip it.

### D7 — Band stats fold `resolve_lemma`; counts by scanning
`band_stats` enumerates a level/band's lemmas and folds `resolve_lemma` to tally
confirmed / presumed / to-learn. Ranks aren't dense, so denominators come from scanning the
band, not arithmetic — exact, and cheap at pack scale (not per-keystroke). `Pack` gets a
rank/level-band enumerator over its private `freq`/`glosses`/`lexicon`.

### D8 — Seeding uses `EncounterSource::Import`, capped + idempotent
`Deck::seed_lemmas` + `Card::seeded` add cards for chosen lemmas with `EncounterSource::
Import` (the reserved variant) instead of fabricating a URL/session. Capped per call
(default ~50), skips lemmas already carded or with an explicit status, order chosen by the
caller (commonest-first default).

### D9 — Implementation slices, core-first (one change, stacked PRs)
1. Engine (`lingua-core`): band enumerator, `band_stats`, `KnownSource::Exposure` +
   `promote_by_exposure`, `Exposure` distinct-day field, level-aware `resolve_lemma`,
   `Deck::seed_lemmas`/`Card::seeded` — all host-testable, no WASM, no pack-format change
   yet (levels injected via the trait in tests).
2. Pack (`crates/lingua-pack`, `scripts/lingua-data/`): CEFR join, SOURCES/NOTICE + licence
   guard, format + `analyzer_version` bump, regenerate fixtures + real pack.
3. WASM (`crates/lingua-wasm`): JSON bindings (`bandStats`, `seedBand`, `promoteByExposure`,
   level get/set) — thin glue, coverage-excluded.
4. Extension: level picker, CEFR ladder, level-gated highlighting, "Renforcer un niveau",
   call `promoteByExposure` on read.

## Risks / Trade-offs

- **Reversing the documented v1 exposure stance** → mitigated by the four guardrails
  (below-level only, distinct-day threshold, reversible `Exposure` provenance, explicit
  status wins) and by keeping promotion caller-driven so the agent is unaffected.
- **Cold-start ladder looks discouraging** (A1 low before reading) → the presumed tier fills
  the bar immediately from the declared level; confirmed fills as you read, so the bar is
  never empty for below-level rows.
- **`analyzer_version` bump breaks a stale pack at runtime** → the `build.mjs` guard turns it
  into a build-time failure; the slice regenerates every declared pack + fixtures + real
  pack.
- **CEFR-J multi-level ambiguity** → fixed collapse rule (lowest level per lemma).
- **Octanove C1/C2 is small (~2,136)** → advanced-band coverage is thinner than EVP would be;
  frequency ordering backfills where a level is absent. Acceptable for MVP.
- **Presumed knowns inflate early bands** (the calibration caveat) → surfaced honestly by the
  presumed/confirmed split; the ladder never presents presumed as proven.

## Migration Plan

- Ship engine slice first (no format change) → safe to merge behind the trait.
- Pack slice bumps `analyzer_version`: regenerate all declared packs, the 3 fixtures, and the
  real pack in the same PR; CI + the `build.mjs` guard verify. Rollback = revert the pack
  slice (engine slice keeps working on the old pack with levels reported unavailable).
- No data migration for user state: an existing frequency calibration remains valid as the
  fallback; declaring a level is additive.

## Open Questions

- Default distinct-day threshold N: proposed 4. Confirm in review.
- Default seeding cap and order: proposed 50, commonest-first. Confirm.
- Should the declared CEFR level sync across devices (via the existing account state) or stay
  device-local like the frequency calibration? Proposed: sync it (it is a small preference,
  and cross-device consistency matches the sync feature already shipped).
- Distinct-day storage shape (last-day + counter vs bounded ring) — settle during the engine
  slice; both keep state bounded.
