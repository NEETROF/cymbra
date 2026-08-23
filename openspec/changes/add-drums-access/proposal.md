## Why

`add-unpitched-notation` teaches the parser to read percussion notation but
deliberately leaves the admission gate closed, because letting drum scores into the
system means deciding who may see them. This change makes that decision and
enforces it.

The drum feature is being built across six changes. Until it is finished a drum
score renders as an empty staff and plays silence, so it must reach only the people
building and testing it: **staff (admin/moderator) and active members of the
`midi-drums` beta campaign**. Per the platform rule in `runtime-feature-flags`, that
restriction is enforced by the **backend** — hiding the UI is defence in depth, not
the gate.

Enforcing it server-side requires knowing which scores are percussion, which
requires a queryable instrument column — which is also the right moment to retire
`is_piano`. That facet is a proxy, not a detection: `staves >= 2` means "written on
a grand staff, therefore probably a keyboard". It is wrong in both directions — a
single-staff piano piece reads as not-piano, an organ or any two-staff arrangement
reads as piano, and a drum kit notated on two staves would read as a piano grand
staff. Now that the parse yields a real instrument classification, the proxy goes.

## What Changes

**Data model — `is_piano` becomes `instrument`**

- **BREAKING (schema):** `music.catalog_scores.is_piano` and
  `music.user_scores.is_piano` (`BOOLEAN`) become `instrument` (`TEXT`), holding
  `keyboard` | `percussion` | `unknown`.
- Backfill maps `is_piano = true → 'keyboard'` and `is_piano = false → 'unknown'`,
  **not** `'other'`: false meant "fewer than two staves", which includes
  single-staff piano. Claiming to know is worse than admitting we do not, and the
  existing readers already exclude those rows.
- Optional follow-up, not required here: a re-derivation job over the stored bytes
  to replace `unknown` with a real classification.

**Wire protocol**

- **BREAKING:** `SearchCatalogRequest.is_piano` (bool) becomes an instrument filter.
  `CatalogHit` and `ScoreRecord` carry the instrument.

**Backend enforcement**

- The `music` module gains a `FlagService` dependency and builds an evaluation
  context from what it already holds: `AuthIdentity.roles` for staff, and the
  existing `PlanSource` port for active beta memberships.
- Every path that could disclose or accept a percussion score is gated: catalog
  search, catalog and user score bytes, the saved and owned listings, upload, and
  the rating deck. A caller without the feature gets the same answer as if the
  score did not exist — never a distinguishable "forbidden".
- The gate **fails closed**: an unreadable flag store, an unwired `PlanSource` or a
  score whose instrument is `unknown` all resolve to "not percussion-eligible".

**Admission gate opens**

- `note_count` counts unpitched notes and `validate()` accepts a percussion score,
  so it can be uploaded and crawled — behind the enforcement above.

**App (`apps/music`)**

- Score Hub stops pinning the piano filter and filters by instrument instead;
  the drum option appears only when the feature is visible.
- The upload flow shows the **detected** instrument in its read-only summary —
  displayed, never chosen, like every other derived facet — and refuses a
  percussion file when the feature is not visible, with a localised reason.
- Score cards and the library show the instrument.

**Back-office (`apps/back-office`)**

- The piano checkbox in `FiltersBar.vue` becomes an instrument filter, carried
  through `stores/catalog.ts` into the catalog and review queue.
- Moderators are staff, so they see percussion scores by construction — which makes
  the notation preview reachable immediately. The console's renderer is its own
  TypeScript painter (`lib/notation/painter.ts`), and it still assumes pitched
  notation on a treble or bass staff, so a drum score must show an explicit **"not
  previewable yet"** state rather than a confidently wrong drawing a moderator would
  read as a corrupt file. Drawing percussion properly is `add-drum-notation-render`.

**Deliberately deferred, named precisely so they are not lost**

- **SoundFonts by instrument** → `add-drum-audio-channel`. The axis already exists
  (`music.soundfonts.instrument`, `NOT NULL DEFAULT 'piano'`, whose migration comment
  already reads "piano, and later guitar/drums/…"), but it is **declared by the
  uploader** (`server/src/soundfont.rs:383`) and filters nothing today. That change
  must: filter the app's sound picker and the console's `SoundFontPicker.vue` by
  family; carry the instrument through the admin upload, the user import and the
  propose flow; remember a **separate choice per family**, so loading a drum score
  after a piano score does not keep a piano font and play silence; and ideally
  **verify** the declared family from the file's preset banks — an SF2 exposing bank
  128 is a drum kit — so the one soundfont facet that decides which synthesizer
  channel is used is not merely trusted.
- **Percussion drawing** → `add-drum-notation-render`, in **two** painters against
  one shared geometry: `apps/music/lib/painters/` (Dart) and
  `apps/back-office/src/lib/notation/painter.ts` (TypeScript). They are independent
  implementations, so percussion clef, alternative noteheads and two-voice layout
  land twice or the console drifts from the app.
- **Percussion audio** → `add-drum-audio-channel`, also in **two** places:
  `apps/music/rust/src/api/audio_core.rs` and `crates/audio-wasm/src/lib.rs:42`,
  which carries its own `PIANO_CHANNEL = 0` copy annotated "matches the app".

## Capabilities

### New Capabilities

- `music-drums-visibility`: who the drum feature is exposed to (staff plus active
  `midi-drums` campaign members), how that audience is resolved server-side, and
  the requirement that every disclosing path enforce it and fail closed.

### Modified Capabilities

- `score-facets`: the staff-count `is_piano` facet is **replaced** by the derived
  instrument classification, and `note_count` counts unpitched notes so the
  playable-notes gate admits a percussion score.
- `catalog-search`: the `is_piano` filter is replaced by an instrument filter, and
  results are constrained by the caller's drum eligibility regardless of the filter
  they ask for.
- `score-hub`: the hub stops constraining to piano — its "While the corpus is
  piano-only" condition no longer holds — and constrains by instrument instead.
- `score-upload`: validation accepts a percussion score; the detected instrument is
  displayed and cannot be declared; an ineligible contributor is refused.
- `music-percussion-notation`: the validation gate, deliberately left closed by
  `add-unpitched-notation`, opens here.
- `web-notation-render`: the console preview declares a percussion score
  unpreviewable rather than drawing it with keyboard assumptions.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the flag client, the parser's instrument classification | instrument filter in the hub, instrument in the upload summary, the upload refusal |
| **Back-office** (`apps/back-office`) | the same catalog API | instrument filter replacing the piano checkbox |
| **Platform** (feature flags) | the existing `beta:<campaign>` scope, resolved server-side | one declared key; the `music` module becomes a flag consumer |
| **Plans** | the existing `PlanSource` port already wired into `music` | the `midi-drums` campaign, created through the admin plan console |
| **ID / Live / Site** | — | untouched |

**Code**

- `backend/music/migrations/`: one migration, two tables, with backfill.
- `backend/music/src/`: `pg.rs` (`META_COLS`, `bind_meta`, `meta_from_row`, the
  search predicate, the rating-deck predicate at `pg.rs:796`), `catalog_search.rs`,
  `grpc.rs`, `module.rs`, `repo.rs`.
- `backend/music/proto/score.proto` + generated clients for the app and the console.
- `backend/feature-flags/src/registry.rs`: one declared key.
- `apps/music/lib`: `catalog_search_notifier.dart:270`, `catalog_service.dart:131`,
  `score_upload_notifier.dart`, the upload screen, the score card.
- `apps/back-office/src`: `FiltersBar.vue:55`, `stores/catalog.ts`, `CatalogView.vue`.

**Precedent to follow.** `backend/notifications/src/dispatch.rs:38` already resolves
flags server-side with `FlagService` + `EvalContext`, defaulting every unreadable
value to disabled. The music module should mirror that shape rather than invent one.

**Operational prerequisite.** The `midi-drums` beta campaign (kind `feature`) must
exist and have its testers enrolled before the flag's `beta:midi-drums` scope
reaches anyone but staff. Creating it is an admin action in the plan console, not
code.
