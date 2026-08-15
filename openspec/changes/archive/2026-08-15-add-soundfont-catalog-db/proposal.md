## Why

Today the server-owned SoundFont catalog is a **hardcoded Rust constant**
(`catalog()` in `backend/server/src/soundfont.rs`) and the app's picker lists
downloadable pianos from **hardcoded entries in the Flutter code** — including
grands (YDP, Salamander) that are *not actually hosted* anywhere, so selecting
them silently fails and falls back to the default. Both catalogs can drift, and
adding a font means editing and redeploying code. The catalog should be **data,
not code**: a single source of truth in the database that the delivery route, the
new listing endpoint, and the app all read — so the app only ever proposes fonts
that genuinely exist on the server, and ops can add a font by inserting a row +
uploading the object (no code change).

## What Changes

- **Persist the SoundFont catalog in a database table** (in the `music` schema),
  owned by the `cymbra-music` crate: `id`, `label`, `object_key`, `tier`
  (free/paid), `license`, `attribution`, `size_bytes`. Seed the CC0 default
  (`upright-piano-kw`) row from a migration.
- **Add a gRPC `ListSoundFonts` RPC** on `ScoreService` that lists the catalog's
  fonts (id, label, license, attribution, tier) to an authenticated caller, so
  clients can discover what is available.
- **Refactor the existing REST delivery route** (`GET /soundfonts/{id}`, change
  `add-soundfont-delivery`) to resolve id → object_key/tier from the **table**
  instead of the hardcoded `catalog()`, keeping the entitlement check and
  range-streaming exactly as they are.
- **Drive the app's downloadable-piano catalog from `ListSoundFonts`** behind an
  injectable gRPC seam, and **remove the hardcoded fictional YDP/Salamander
  entries**. The bundled CC0 default and user imports are unchanged; the bundled
  default id is filtered out of the server list to avoid a duplicate row. If the
  listing is unavailable, the picker degrades to bundled default + imports.

## Capabilities

### New Capabilities
<!-- none: this extends the existing soundfont-delivery capability -->

### Modified Capabilities
- `soundfont-delivery`: the server-owned catalog requirement changes from an
  unspecified server-owned mapping to a **persisted database catalog** (the source
  of truth the delivery route resolves through), and a new **authenticated
  listing** requirement is added so clients can enumerate available fonts and
  never present a font absent from the server.

## Impact

- **Depends on** `add-soundfont-delivery` (the route, bucket, entitlement seam)
  and `piano-sound-selection` (the app catalog/picker/selection); sequences after
  both.
- **Database**: new migration adding the `soundfonts` table in the `music` schema
  + a seed row for the CC0 default. New `SoundFontRepo` (trait + Pg impl) in
  `cymbra-music`.
- **Backend (Rust)**:
  - `cymbra-music`: `SoundFontRepo`, the font catalog types (moved here from
    `backend/server/src/soundfont.rs` so both the delivery route and the RPC share
    one source), and the `ListSoundFonts` handler on `ScoreGrpc`.
  - `backend/server/src/soundfont.rs`: resolve id → entry via the repo instead of
    the static `catalog()`; entitlement + range-streaming logic unchanged. The
    `store: None` "unconfigured → 503" behavior is preserved.
  - `backend/music/proto/score.proto`: `SoundFont` message + `ListSoundFonts` RPC
    (tonic regen via `build.rs`). Public gRPC surface → re-run the Dart stub gen.
- **Flutter (`apps/music`)**:
  - New injectable `SoundFontCatalogService` (gRPC seam over `ListSoundFonts`) +
    provider exposing `download`-kind `PianoEntry`s; a fake for tests.
  - `state/piano_catalog.dart`: `builtInPianos` keeps **only** the bundled CC0
    default; the catalog unions bundled default + server download list + user
    imports; remove the YDP/Salamander constants.
  - `gen-grpc` regenerates `score.pbgrpc.dart`.
- **Tests**: Rust repo/RPC unit tests + delivery-route tests updated to the
  repo-backed lookup; Flutter tests drive the download list from a fake gRPC
  catalog seam (the widget/selection tests that referenced the hardcoded grands
  now use the fake).
- **Back-office**: `yarn gen` regenerates its TS stubs (no behavior change unless
  it later lists fonts).
