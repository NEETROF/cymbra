## Why

`add-soundfont-catalog-db` made the SoundFont catalog data (the `music.soundfonts`
table), but the only way to add a font today is a SQL insert plus a manual object
upload — ops-only, error-prone, and easy to get half-done (a row with no object, or
an object with no row). The back office already administers the music catalog
(moderation, metadata editing, roles, flags); it should also own the SoundFont
catalog, so a moderator/admin can add, edit, and remove instrument sounds from a UI
— with the bytes and the metadata handled together. This is also the surface where
**paid, rewards-unlockable** fonts will later be curated (the `tier` field already
exists); this change builds the management screen, free-only for now.

## What Changes

- **New back-office "Sound fonts" screen** (nav entry gated to music-scope admins):
  a table of the catalog fonts and an add/edit form (label, license, attribution,
  tier) with a `.sf2` file picker, plus remove.
- **Admin-scoped catalog writes** on the backend: create / update / delete /
  admin-list SoundFont catalog rows (moderator/admin in the `music` scope, mirroring
  catalog moderation/editing). Create and delete keep the row and the stored object
  **in sync** (create writes both; delete removes both).
- **`.sf2` upload path**: an admin-gated way to put the (large, tens–hundreds of MB)
  font bytes into the private `cymbra-soundfonts` bucket — an HTTP upload route
  symmetric with the existing `GET /soundfonts/{id}` delivery (streamed, not buffered
  through gRPC-web).
- **`tier` is editable but stays informational**: every font is free (served to any
  signed-in identity) for now; marking a font `paid` records intent for the future
  rewards/entitlement system (`add-curation-rewards`) — this change adds **no**
  paid-gating or rewards wiring.
- **Default unchanged**: only the seeded CC0 default (`upright-piano-kw`, used by the
  back-office preview) ships by default; everything else is added through this screen.
- The server-owned catalog (`SoundFontRepo`) stays the **single source of truth**, so
  a change made here is immediately reflected by the app's `ListSoundFonts` and the
  delivery route.

## Capabilities

### New Capabilities
- `soundfont-catalog-admin`: administer the SoundFont catalog from the back office —
  list, create (upload `.sf2` + metadata), edit metadata, and remove fonts, gated to
  music-scope admins, keeping the catalog row and its stored object in sync.

### Modified Capabilities
- `soundfont-delivery`: the persisted catalog gains **administrative writes** — its
  rows and their stored objects can be created, updated, and deleted through an
  admin-gated surface (previously the catalog was read-only after seeding), and the
  private store gains an authenticated **upload** counterpart to its download route.

## Impact

- **Depends on** `add-soundfont-catalog-db` (the table, `SoundFontRepo`, delivery
  route) and the back-office admin session / scope-aware role model; sequences after
  them.
- **Backend (Rust)**:
  - `cymbra-music`: extend `SoundFontRepo` with `upsert` / `update_meta` / `delete`;
    the `ScoreService` gains admin-gated `AdminListSoundFonts`, `CreateSoundFont`
    (metadata; object handled by the upload route or a bytes field), `UpdateSoundFont`,
    `DeleteSoundFont` (guarded by `require_moderator_or_admin` in the `music` scope).
  - `backend/server/src/soundfont.rs`: add an admin-gated upload route (`PUT`/`POST
    /soundfonts/{id}`) that streams bytes into the private store, and object removal on
    delete; reuse the existing auth/entitlement/store seams.
  - `backend/music/proto/score.proto`: new admin messages + RPCs (tonic regen).
    Public gRPC surface → regenerate the back-office (and app, if it consumes any)
    stubs.
- **Back-office (Vue 3 + TS)**:
  - `src/router.ts`: `/soundfonts` route + nav entry, `meta: { admin: true }`
    (music-scope).
  - `src/stores/soundfonts.ts`: a Pinia store behind the `api()` client seam, state as
    an `Async<T>` union; a `SoundFontsView.vue` table + add/edit form with a `.sf2`
    file input, wired to the upload route + the admin RPCs.
  - `yarn gen` regenerates the TS stubs; Playwright e2e via the gated fake-client seam.
- **App (`apps/music`)**: no change — it already lists via `ListSoundFonts` and picks
  up new fonts automatically.
- **Tests**: Rust repo + admin-RPC + upload-route tests; back-office store + view unit
  tests and a Playwright flow (add → appears → edit → delete) against the fake seam.
