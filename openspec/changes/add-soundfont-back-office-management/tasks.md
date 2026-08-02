## 1. Backend: catalog repo writes (`cymbra-music`)

- [x] 1.1 Extend `SoundFontRepo` with writes: `upsert_meta`/`update_meta` (label, license, attribution, tier), `insert(entry)` (fails on existing id), and `delete(id)`; keep `list`/`lookup`. Update `PgSoundFontRepo` (schema-qualified `music.soundfonts`, runtime `sqlx::query`) and the `FakeSoundFontRepo`.
- [x] 1.2 Unit-test the Pg writes (insert rejects duplicate id; update changes only metadata; delete removes the row) and the fake mirrors them.

## 2. Backend: admin gRPC RPCs (`ScoreService`)

- [x] 2.1 `backend/music/proto/score.proto`: add `AdminSoundFont` (id, label, object_key, tier, license, attribution, size, has_object) + `AdminListSoundFontsRequest/Response`, `UpdateSoundFontRequest` (id + metadata), `DeleteSoundFontRequest/Response`, and the three RPCs on `ScoreService` (tonic regen via `build.rs`).
- [x] 2.2 Implement them in `backend/music/src/grpc.rs`: gate each with `require_moderator_or_admin(&identity(&req)?)` (music scope); `AdminListSoundFonts` reads the repo; `UpdateSoundFont` writes metadata; `DeleteSoundFont` deletes the row **and** the stored object (best-effort object delete), needing the SoundFont `ObjectStorage` handle wired into `ScoreGrpc`.
- [x] 2.3 Unit-test the admin RPCs: unauthorized (no mod/admin) is refused; update/delete change the fake repo; delete also asks the fake store to remove the object.

## 3. Backend: admin-gated upload/delete route (`backend/server/src/soundfont.rs`)

- [x] 3.1 Add an authenticated, **admin-gated** upload route symmetric with the delivery route (e.g. `POST /soundfonts` multipart: `.sf2` + metadata, or `PUT /soundfonts/{id}`): verify the token is a music-scope moderator/admin (roles via `token::verify`), validate the RIFF/`sfbk` header, **stream** the body into the private store (derive `object_key = {id}.sf2`), then record the catalog row via the repo — object first, row second. Reject invalid/unauthorized/duplicate before writing.
- [x] 3.2 Wire the SoundFont `ObjectStorage` + `SoundFontRepo` + role-checking auth into the route/state and its construction in `backend/server/src/main.rs`; keep `store: None`/`repo: None` ⇒ the feature reports unavailable.
- [x] 3.3 Host-test the pure upload decision (auth present? admin? valid body?) and the handler via oneshot (unauthorized→401/403, non-admin→403, invalid body→rejected, valid→stored + row recorded), seeding fakes (no network).

## 4. Back-office: SoundFont management screen (`apps/back-office`)

- [x] 4.1 `yarn gen` so the TS stubs include the admin RPCs.
- [x] 4.2 Add `src/stores/soundfonts.ts`: a Pinia store behind the `api()` client seam (+ a thin token-bound `fetch` wrapper for the multipart upload route); list and each mutation modelled as one `Async<T>` union (`idle|loading|success|error`), matched with `ts-pattern` — errors live in the union, never thrown to the view.
- [x] 4.3 Add `src/views/SoundFontsView.vue`: a table of catalog fonts + an add form (`.sf2` file picker + label/license/attribution/tier), a metadata edit affordance, and a remove affordance — calling only the store (no direct API/fetch in the component).
- [x] 4.4 `src/router.ts`: add the `/soundfonts` route + nav entry with `meta: { admin: true }` (music scope), consistent with the roles/flags admin sections.

## 5. Tests

- [x] 5.1 Backend: repo write tests + admin-RPC tests + upload-route tests green (section 1.2 / 2.3 / 3.3).
- [x] 5.2 Back-office unit: the store drives list/add/edit/remove against the fake `api()` seam + a fake upload; the `Async` union transitions are asserted; a non-admin / error result surfaces as an error state (no raw status string).
- [x] 5.3 Back-office e2e (Playwright, gated fake-client seam, no backend): add a font → it appears in the table → edit metadata → remove it; and the screen is not reachable for a non-admin.

## 4b. Create/edit drawer + audio preview

- [x] 4b.1 Parameterise `useScorePlayer` with an optional SoundFont-bytes ref (render with the candidate `.sf2` instead of the default; re-render when it changes).
- [x] 4b.2 `SoundFontDrawer.vue`: right-to-left drawer for create (file + metadata) and edit (metadata only), with a catalog-piece picker + Play/Pause auditioning the candidate font (picked file on create, stored font on edit); calls only the store.
- [x] 4b.3 Store: `previewPieces` / `pieceBytes` / `fontBytes` (through the client seam); rewire `SoundFontsView.vue` to an "Add" button + per-row Edit that open the drawer (remove the inline add form / inline edit).
- [x] 4b.4 Update the e2e to drive the drawer (add → appears → edit → remove) and the store/unit tests accordingly.

## 6. Verify & gate

- [x] 6.1 Backend: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov --workspace --fail-under-lines 80` pass (route/glue kept in the coverage ignore as applicable; pure decision/repo logic tested).
- [x] 6.2 Back-office: `yarn gen`, `yarn typecheck`, `yarn lint`, `yarn test` (vitest) clean; `yarn build` succeeds.
- [ ] 6.3 Manually confirm: as a music-scope admin, add a `.sf2` (it appears in the app picker + downloads), edit its label/attribution, mark one paid (still delivered), and remove one (gone from app + object deleted); a non-admin cannot see the screen.
- [x] 6.4 `openspec validate add-soundfont-back-office-management --strict` passes.
