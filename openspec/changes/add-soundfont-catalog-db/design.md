## Context

`add-soundfont-delivery` shipped an authenticated, range-capable REST route
`GET /soundfonts/{id}` ([backend/server/src/soundfont.rs](backend/server/src/soundfont.rs))
that streams a font's bytes from the private `cymbra-soundfonts` bucket, gated by
a per-font entitlement check and resolved through a **server-owned catalog** —
currently a hardcoded Rust constant `catalog()` holding a single free CC0 entry
(`upright-piano-kw`). `piano-sound-selection` added the app-side picker; its
downloadable pianos are **hardcoded in Flutter** (`builtInPianos` in
[state/piano_catalog.dart](apps/music/lib/state/piano_catalog.dart)) and include
YDP/Salamander grands that are **not actually hosted**, so choosing one fails and
falls back to the default.

This change makes the catalog **data, not code**: one source of truth in the
`music` schema that the delivery route, a new listing RPC, and the app all read.
The `music` crate already owns a Postgres pool + `sqlx` migrations
(`MIGRATOR = sqlx::migrate!("./migrations")`, schema `music.*`) and the
`ScoreService` gRPC surface, so the table, repo, and RPC live there; the server's
delivery route (which already depends on `cymbra-music`) resolves ids through the
repo.

## Goals / Non-Goals

**Goals:**
- A persisted `music.soundfonts` catalog table as the single source of truth
  (id, label, object_key, tier, license, attribution, size).
- A gRPC `ListSoundFonts` RPC so clients discover what fonts exist.
- The delivery route resolves id → object_key/tier from the table (no hardcoded
  catalog), keeping entitlement + range-streaming unchanged.
- The app's downloadable list comes from the server; no font is ever shown that
  the server doesn't have. Bundled default + user imports unchanged.
- Everything behind the existing seams; degrades gracefully (no listing → bundled
  + imports only).

**Non-Goals:**
- No admin/back-office UI to CRUD the catalog (rows are seeded by migration / ops;
  a management UI is a later change).
- No paid fonts or purchase flow (the `tier`/entitlement seam already exists and
  stays; no paid rows yet).
- No change to the byte-download transport — bytes still come from the REST
  `GET /soundfonts/{id}`; only *discovery* moves to gRPC.
- No moving of imported (user) SoundFonts to the server (still local-only).
- No object-store *listing* — availability is the DB row, not a bucket scan (a row
  without its object is an ops error, surfaced as a failed download → fallback).

## Decisions

### Decision: Catalog table + repo live in the `music` crate/schema
Add `music.soundfonts` via a new `backend/music/migrations/0012_soundfonts.sql`,
and a `SoundFontRepo` trait (+ `PgSoundFontRepo`) in `cymbra-music`. **Why:** the
`music` crate already owns the pool, the migrator, and `ScoreService`; the server
binary already depends on `cymbra-music`, so both the RPC (in the crate) and the
delivery route (in the server) resolve through one repo — no cross-crate
duplication, no second pool. **Alternatives:** a dedicated `soundfont` schema/crate
(rejected: extra pool + wiring for one small table); keep the static `catalog()`
and only add the RPC (rejected: two sources of truth that drift — the exact
problem being fixed).

### Decision: Move the font catalog types out of the server binary
`FontEntry`/`Tier`/`may_access`/`decide` reference the catalog. Move `FontEntry`
and `Tier` into `cymbra-music` (e.g. `soundfont::FontEntry`) so the repo returns
them and the server route imports them; the **pure delivery decision** logic
(`decide`, `may_access`, `Entitlements`, range parsing) stays in the server route,
now fed by a repo lookup instead of the static array. **Why:** keeps the pure,
host-tested decision logic where it is while making the catalog data-driven; the
server keeps owning HTTP/entitlement concerns. **Trade-off:** the route becomes
`async` at the lookup point (a DB read) instead of a `const` array scan — cheap and
cache-friendly; the "unconfigured store → 503" gate is preserved.

### Decision: `ListSoundFonts` on `ScoreService`, authenticated, all rows
Add `rpc ListSoundFonts(ListSoundFontsRequest) returns (ListSoundFontsResponse)`
with `SoundFont { id, label, license, attribution, tier }`. It requires an
authenticated identity (like the other catalog RPCs via `identity(&req)`) and
returns **every** catalog row (free and, later, paid — the client shows paid ones
as locked; none exist yet). **Why:** reuses the music service and its interceptor;
listing is cheap and identical for everyone, so no per-user filtering in v1.
**Alternatives:** a new `SoundFontService` (rejected: extra service wiring for one
RPC); filter to entitled-only (rejected for v1: listing paid-but-locked is useful
UX, and there are no paid fonts yet).

### Decision: App sources downloadable pianos from the RPC; default filtered out
`builtInPianos` keeps **only** the bundled CC0 default. A new injectable
`SoundFontCatalogService` (gRPC seam) calls `ListSoundFonts` and maps rows to
`download`-kind `PianoEntry`s, **excluding the row whose id == `defaultPianoId`**
(the default is the bundled entry — never show it twice). `pianoCatalog` becomes
`[bundled default] + [server download list] + [user imports]`. A listing failure
(offline, unauthenticated, backend down) yields an **empty** download list, so the
picker shows only the bundled default + imports — never a font that isn't there.
**Why:** this is the whole point — the app proposes exactly what the server has.
**Trade-off:** the download list is now async; the catalog provider already tolerates
async imports (`AsyncNotifier`), so the server list is folded in the same way
(`valueOrNull ?? const []`).

### Decision: Seed the CC0 default via migration; ops adds fonts as data
The migration inserts the `upright-piano-kw` row (object_key
`UprightPianoKW-20220221.sf2`, tier free, CC0). Real grands are added later by
inserting a row **and** uploading the object to the bucket — no code change.
**Why:** matches "catalog is data"; keeps the free default present out of the box.

## Risks / Trade-offs

- **Row without its object** (DB says a font exists, bucket doesn't have it) → the
  listing shows it but the download 404s. Mitigation: the app already falls back to
  the default on a failed download (non-fatal); seeding is migration-controlled and
  ops uploads the object before inserting a row. (No bucket scan in v1.)
- **Two regen pipelines** (tonic `build.rs` for Rust; `gen-grpc`/protoc for the
  Flutter Dart stubs; `yarn gen` for the back-office) → the proto change must be
  regenerated in each. Mitigation: documented in tasks; CI runs the app/back-office
  gens.
- **Delivery route becomes async/DB-bound** → a per-request DB read on the hot
  streaming path. Mitigation: it is a single indexed PK lookup on a tiny table,
  dwarfed by the byte streaming; can be cached later if ever needed.
- **Startup ordering** → the app reads the list at launch; if the backend/session
  isn't ready the list is empty and fills in on refresh. Mitigation: the provider
  is refreshable and the picker degrades gracefully; the selection/persistence path
  is unaffected (it already validates against the live catalog).
- **Migration is one-way-ish** → dropping the table later loses seeded metadata.
  Mitigation: seed is reproducible from the migration; the object bytes live in the
  bucket, not the DB.

## Migration Plan

Additive and sequenced **after** `add-soundfont-delivery` and
`piano-sound-selection`:
1. Add `0012_soundfonts.sql` (table + seed the CC0 default) — runs on server boot
   via the existing `cymbra-music::MIGRATOR`.
2. Add `SoundFontRepo`/`PgSoundFontRepo` + move `FontEntry`/`Tier` into
   `cymbra-music`; point the delivery route's lookup at the repo.
3. Add the proto RPC + `ListSoundFonts` handler; regenerate stubs (Rust auto via
   `build.rs`, Dart via `gen-grpc`, back-office via `yarn gen`).
4. App: `SoundFontCatalogService` + provider; fold into `pianoCatalog`; drop the
   hardcoded grands; update tests to drive the download list from a fake seam.

**Rollback:** revert the app to the hardcoded `builtInPianos` (or empty download
list), revert the route to the static `catalog()`, and leave the table in place
(unused) or drop it via a down-migration. Delivery of the CC0 default is unaffected
throughout.

## Open Questions

- **Size column source** — store `size_bytes` in the row (ops fills it) or derive
  it from the object at delivery time? Lean: nullable column, informational only in
  v1 (the app doesn't need it to download).
- **Back-office management** — a later change can add a moderator/admin CRUD screen
  over this table (upload object + insert row) so ops never touch SQL. Out of scope
  here.
- **Listing auth scope** — v1 lists to any authenticated identity. If paid fonts
  arrive, revisit whether locked fonts are listed to everyone (probably yes, shown
  as locked) or filtered.
