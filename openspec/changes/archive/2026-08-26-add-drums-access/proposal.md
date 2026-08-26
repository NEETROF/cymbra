## Why

`add-unpitched-notation` teaches the parser to read percussion notation but
deliberately leaves the admission gate closed, because letting drum scores into the
system means deciding who may see them. This change makes that decision and
enforces it.

The drum feature is being built across eight changes — `add-unpitched-notation`,
this one, `add-drum-kit-view`, `add-instrument-context`, then
`add-drum-notation-render`, `add-drum-audio-channel`, `add-drum-input-mapping` and
`add-drum-scoring`. Until it is finished a drum score is presented **wrongly**, not
blankly: the staff view is near-empty, but the waterfall would draw the score's GM
key numbers as falling *piano* notes and the piano synthesizer would sound them —
a confident wrong rendering, which is worse than an empty one. So the feature must
reach only the people building and testing it: **staff (admin/moderator) and
active members of the `midi-drums` beta campaign** — and even for them, the app
shows an explicit "drums not playable yet" state instead of entering the player
(see `music-drums-visibility`). Per the platform rule in `runtime-feature-flags`, that
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
- The column is filled by **re-deriving the instrument from each row's stored
  bytes**, never by translating `is_piano`. The flag cannot be translated — `true`
  only ever meant "two or more staves" and `false` covers single-staff piano *and*
  drum parts alike — and the corpus already holds real percussion scores (verified
  on prod: ~252 scores with unpitched notes, ~29 pure drum parts), so a flag
  translation would record real percussion rows as `unknown` and the gate would
  serve them to everyone.
- The bytes live in the object store, so this cannot be a SQL migration. The
  schema migration only **adds** the column (`NOT NULL DEFAULT 'unknown'`, CHECK
  on the three values); a separate **application-level backfill** — a one-shot
  admin/worker pass that streams each object, parses it with the new classifier,
  and updates the row, recording `unknown` for anything unreadable or unparseable —
  fills it. The gate is not a boundary until that pass completes, and the flag is
  not activated before it has.

**Wire protocol**

- **BREAKING:** `SearchCatalogRequest.is_piano` (bool) is **retired** — `reserved
  6; reserved "is_piano";` — and the instrument filter takes a **fresh** field
  number, never number 6. Reusing 6 would hand an old client's boolean varint to
  the new field (a wire-type mismatch for a string, a silent wrong filter for an
  enum). With the number retired, shipped app versions that still pin
  `isPiano: true` silently lose that filter and see unfiltered results, including
  `unknown`-instrument rows — accepted: the pin's premise ("the corpus is
  piano-only") is gone, and the backend still withholds everything they may not
  see. `CatalogHit` and `ScoreRecord` carry the instrument.

**Backend enforcement**

- The `music` module gains a `FlagService` dependency and builds an evaluation
  context from what it already holds: `AuthIdentity.roles` for staff, and the
  existing `PlanSource` port for active beta memberships. The declared key is
  **`drums.enabled`** (app `music`), matching the registry's `<feature>.enabled`
  convention.
- **Every path that can disclose or accept a percussion score is gated** — an
  obligation on the whole serving surface, enumerated in `music-drums-visibility`:
  not just search, bytes, the saved listing, upload and the rating deck, but also
  the metadata read by id (`GetCatalogScore`), the deck's per-score preview bytes
  (`GetRatingPreviewBytes`), the save/rating submissions (existence oracles), the
  daily unlock, and the HTTP audio-preview clip route. On read paths a caller
  without the feature gets the same answer as if the score did not exist — never a
  distinguishable "forbidden"; on the accepting paths (upload, propose) the
  refusal is a typed, localisable one. A caller's **own** uploads are exempt:
  listing, fetching and deleting one's own scores always works (no disclosure to
  self), only proposing them requires eligibility.
- The gate **fails closed**: an unreadable flag store, an unwired `PlanSource` or a
  score whose instrument is `unknown` all resolve to "not percussion-eligible".
- **Interim scoring is off**: until `add-drum-scoring`, a percussion score's play
  submissions engage no rewards, no leaderboards, no streak and no daily-access
  consumption — those artifacts are permanent (ledger rows, monotone bests,
  badges) and keyboard-shaped scoring of a drum part would bake wrong data.

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
- Opening a percussion score shows a localised **"drums not playable yet"** state
  instead of entering the player — the app-side mirror of the console's guard,
  removed by `add-drum-kit-view` when the percussion presentation lands. Without
  it the waterfall would confidently draw and sound the drum part as piano.

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
  uploader** (`backend/server/src/soundfont.rs:383`) and filters nothing today. That change
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
  unpreviewable rather than drawing it with keyboard assumptions; the existing
  browser-render requirement gets the matching percussion carve-out so the two
  rules cannot contradict after archive.
- `moderation-console`: the review queue badges the instrument, and the Play
  control refuses to audition a percussion score through the piano channel.
- `corpus-manifest`: the ingest metadata requirement names the derived
  `instrument` instead of the retired `is_piano` flag, and the manifest stops
  carrying the flag.

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

- `backend/music/migrations/`: one additive migration, two tables (the column
  only — the backfill is application-level, not SQL).
- `backend/music/src/`: `pg.rs` (`META_COLS`, `bind_meta`, `meta_from_row`, the
  search predicate, the rating-deck predicate at `pg.rs:796`), `catalog_search.rs`,
  `grpc.rs`, `module.rs`, `repo.rs`; plus the one-shot re-derivation pass (admin
  command or worker job) over the stored bytes.
- `backend/music/proto/score.proto` + generated clients for the app and the console.
- `backend/feature-flags/src/registry.rs`: one declared key (`drums.enabled`).
- `crates/musicxml-core/src/meta.rs`: `ScoreSummary.is_piano` (`staves >= 2`) is
  removed here — `add-unpitched-notation` deliberately leaves it, so this change
  owns its retirement.
- `crates/score-crawler/src/`: `metadata.rs`, `manifest.rs`, `catalog.rs`,
  `crawl.rs`, `output.rs` — the crawler derives and writes `is_piano` today, and
  switches to deriving/writing the instrument.
- `apps/music/rust/src/api/musicxml.rs`: the bridged `ScoreSummary` swaps
  `is_piano` for the instrument classification, followed by
  `flutter_rust_bridge_codegen generate` (public API change).
- `apps/music/lib`: `catalog_search_notifier.dart:270`, `catalog_service.dart:131`,
  `score_upload_notifier.dart`, the upload screen, the score card, the
  score-opening flow (interim "drums not playable yet" state).
- `apps/back-office/src`: `FiltersBar.vue:55`, `stores/catalog.ts`, `CatalogView.vue`,
  `ReviewView.vue` (badge), `ScorePreview.vue` + the Play control (guards).

**Precedent to follow.** `backend/notifications/src/dispatch.rs:38` already resolves
flags server-side with `FlagService` + `EvalContext`, defaulting every unreadable
value to disabled. The music module should mirror that shape rather than invent one.

**Operational prerequisite.** The `midi-drums` beta campaign (kind `feature`) must
exist and have its testers enrolled before the flag's `beta:midi-drums` scope
reaches anyone but staff. Creating it is an admin action in the plan console, not
code.
