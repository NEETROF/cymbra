## Context

`add-soundfont-catalog-db` put the SoundFont catalog in `music.soundfonts`
(`SoundFontRepo` in `cymbra-music`), read by `ScoreService.ListSoundFonts` and the
`GET /soundfonts/{id}` delivery route ([backend/server/src/soundfont.rs](backend/server/src/soundfont.rs)).
Adding a font is currently a manual SQL insert + object upload. The back office
already administers the music catalog behind music-scope moderator/admin gating
(`require_moderator_or_admin`, routes with `meta: { admin: true }`) and follows the
`vue-frontend-architecture` skill (a Pinia store behind the injectable `api()`
client seam, async state as one `Async<T>` union). This change adds the management
surface there.

The one real design tension is the **font bytes**: a `.sf2` is tens to hundreds of
MB, so how it is uploaded matters.

## Goals / Non-Goals

**Goals:**
- A music-admin back-office screen to list / add / edit / remove catalog fonts.
- Add a font as one action: the `.sf2` bytes **and** the metadata land together, so
  the catalog row and the stored object are never out of sync.
- Reuse `SoundFontRepo` as the single source of truth (app + delivery reflect edits
  immediately).
- Everything gated to music-scope moderator/admin; testable behind the existing seams.

**Non-Goals:**
- **No paid gating / rewards wiring.** `tier` is editable and recorded, but every
  font is served free; unlocking paid fonts via rewards is a later change
  (`add-curation-rewards` + an entitlement source).
- No change to the app (`apps/music`) — it already lists via `ListSoundFonts`.
- No versioning/history of fonts, no bulk import, no object-store browser.
- No editing of the bundled default's bytes (the CC0 default ships in the app too).

## Decisions

### Decision: Font bytes go over an admin-gated HTTP route, not gRPC-web
Add an authenticated, admin-gated **upload** route symmetric with the delivery route
(e.g. `POST /soundfonts` multipart: the `.sf2` plus the metadata fields, or `PUT
/soundfonts/{id}` for a replace), streaming the body into the private store. **Why:**
gRPC-web unary buffers the entire message in browser and server memory and trips
message-size limits; a 100–300 MB font is a streaming HTTP upload, exactly the
inverse of the existing streaming GET. It reuses the same `SoundfontAuth` (JWT) +
private `ObjectStorage` seam. **Alternatives:** bytes in a `CreateSoundFont` gRPC
(rejected: size/memory); a presigned bucket URL for a direct browser→bucket PUT
(rejected: the bucket is private with no public surface, and the backend must own
write + validation + the row/object sync).

### Decision: Bytes over HTTP, metadata CRUD over admin gRPC
Split by payload size: the **create/replace-bytes** path is the HTTP upload route
(and it also writes the catalog row, so row + object are atomic from the caller's
view); **metadata-only edit, delete, and admin-list** are admin gRPC RPCs on
`ScoreService` (`UpdateSoundFont`, `DeleteSoundFont`, `AdminListSoundFonts`). **Why:**
keeps big transfers off gRPC while small metadata ops stay in the familiar
gRPC/store pattern the rest of the back office uses. **Trade-off:** two transports in
one screen; justified by the byte-size gap and hidden behind the store.

### Decision: Server validates and owns the object key
On upload the server verifies the `.sf2` is a real SoundFont (RIFF/`sfbk` header)
before storing, and derives the `object_key` itself (`{id}.sf2`) so the client never
picks storage keys. A create is refused if the id already exists (use replace to
overwrite bytes). **Why:** a bad or misnamed object can never enter the catalog; the
row and object are written together so a listing never shows a font whose bytes are
missing. **Trade-off:** the id is chosen at create and immutable (edits are
metadata/bytes, not id) — simplest and matches the delivery route keyed by id.

### Decision: Delete removes row **and** object
`DeleteSoundFont` deletes the catalog row and best-effort deletes the stored object.
**Why:** no orphaned bytes, no orphaned rows. **Trade-off:** if object deletion fails
(store hiccup) the row is still removed so the app stops offering it; a stray object
is a cheap, sweepable leftover — logged, not fatal. The bundled default is
delete-guarded (or simply left in place; removing its row only affects the
back-office preview, not the app).

### Decision: Admin gating mirrors the music catalog admin surfaces
Both the gRPC RPCs and the HTTP upload/delete require a music-scope
moderator/admin — the gRPC side via `require_moderator_or_admin(&id)`, the HTTP side
by verifying the same access token and its roles in the route. The back-office nav
entry uses `meta: { admin: true }` and the store surfaces authorization failures as
an `Async` error, never a raw status. **Why:** consistency with moderation/editing;
the server is the gate, the UI mirrors it.

### Decision: Create/edit in a right-to-left drawer with an audio preview
Create and edit happen in a right-side drawer (mirroring `FlagDrawer`), not inline:
the create drawer has the `.sf2` picker + metadata, the edit drawer has metadata only
(id/bytes immutable — change bytes by remove + re-add). The drawer embeds an **audio
preview**: the moderator picks a catalog piece and plays it rendered with the
candidate font. **Why:** a font must be *heard* to be judged; a drawer keeps the table
uncluttered and matches the other admin surfaces. **How:** the app's wasm renderer
already takes `render(scoreBytes, sf2Bytes)`, so `useScorePlayer` is parameterised
with an optional SoundFont-bytes ref — the candidate is the picked file's bytes on
create, or the stored font fetched from the delivery route on edit; a change of font
re-renders. Piece list + piece bytes come through the store (`searchCatalog` /
`getCatalogScoreBytes`). **Trade-off:** preview needs the object present (edit) or a
picked file (create); it degrades to a non-fatal message and never blocks saving.

### Decision: Back-office store + view per the frontend architecture skill
A `soundfonts` Pinia store owns all calls behind `api()` (+ a thin fetch wrapper for
the HTTP upload bound to the token getter); its list and each mutation are modelled
as an `Async<T>` union matched with `ts-pattern` — no scattered loading/error refs,
errors live in the union. `SoundFontsView.vue` renders the table + add/edit form and
only calls the store. **Why:** the two hard rules of `vue-frontend-architecture`;
keeps the view backend-agnostic and Playwright-testable via the gated fake seam.

## Risks / Trade-offs

- **Large upload reliability** (100–300 MB over the browser) → a dropped connection
  leaves no row (row is written on successful upload) but possibly a partial object.
  Mitigation: write the row only after the object is fully stored; a partial object
  without a row is inert and sweepable. Consider a size cap + progress UI.
- **Row/object atomicity** across two stores (Postgres + object store) → no XA
  transaction. Mitigation: order operations so the failure mode is always "inert
  leftover", never "listed font with no bytes": create = put object → insert row;
  delete = delete row → delete object.
- **Admin auth on the HTTP route** → the delivery route only needed a user id; upload
  needs roles. Mitigation: verify the token's roles (music-scope moderator/admin) in
  the route, reusing `token::verify`; unauthorized → 401/403 before any store write.
- **Two regen pipelines** (tonic `build.rs`; back-office `yarn gen`) for the new
  proto. Mitigation: documented in tasks.
- **`tier` implies a capability that doesn't exist yet** → a `paid` font is still
  served free today. Mitigation: document clearly; the delivery entitlement seam
  already exists, so wiring rewards later is additive and does not touch this screen.

## Migration Plan

Additive, sequenced after `add-soundfont-catalog-db`:
1. Extend `SoundFontRepo` (`upsert`, `update_meta`, `delete`) + Pg impl.
2. Add the admin gRPC RPCs (`AdminListSoundFonts`/`UpdateSoundFont`/`DeleteSoundFont`)
   and the HTTP upload/delete route; regenerate stubs.
3. Back-office store + view + nav entry; `yarn gen`.
No data migration (the table exists). **Rollback:** remove the route/RPCs/screen; the
catalog reverts to seed-and-SQL management with the app/delivery unaffected.

## Open Questions

- **Upload shape** — one multipart `POST /soundfonts` that carries file + metadata and
  creates the row, vs a two-step (create row `pending` → `PUT /soundfonts/{id}` bytes)?
  Lean: single multipart create for simplicity; revisit if resumable uploads are needed.
- **Max font size** — cap uploads (e.g. reject > N MB) or accept any? Decide with the
  bundle/prod storage budget; a cap + clear error is safer.
- **Replace bytes on edit** — does the edit form allow swapping the `.sf2`, or only
  metadata (delete + re-add to change bytes)? Lean: metadata-only edit in v1, replace
  via re-upload (`PUT /soundfonts/{id}`) if cheap.
- **Admin-list vs reuse ListSoundFonts** — a separate `AdminListSoundFonts` (shows all
  fields incl. object presence/size) vs reusing the user listing. Lean: a dedicated
  admin list so the screen can show storage/tier detail the app listing omits.
