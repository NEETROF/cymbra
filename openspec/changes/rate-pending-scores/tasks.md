## 1. Deck sourcing widens to pending + accepted (backend)

- [x] 1.1 Change `rating_deck` sourcing SQL (`backend/music/src/pg.rs`) from `moderation_status = 'accepted'` to `moderation_status IN ('pending','accepted')`; keep un-rated-by-caller, least-rated-first, `id` tiebreak, pagination. `rejected` never sourced.
- [x] 1.2 Mirror the sourcing in the `FakeCatalogSearchRepo::rating_deck` fake (`backend/music/src/catalog_search.rs`) so module tests exercise the pending+accepted pool.
- [x] 1.3 Update the `rating_deck` doc/comments (module + repo trait) to state pending+accepted, rejected excluded.

## 2. Rating gate allows pending + accepted (backend)

- [x] 2.1 Add a rating-scoped score resolution in the module (`backend/music/src/module.rs`): resolve a score that is `pending` OR `accepted`, refuse `rejected`/unknown — instead of the accepted-only `object_key(id, false)` path used today.
- [x] 2.2 Wire `submit_rating` to that resolution so a rating on a pending score is accepted, a rejected/unknown score is `NotFound`, and the one-row-per-(user,score) upsert is unchanged.
- [x] 2.3 Fake/repo support (if needed) so the module can check a score's moderation status for the rating gate without the accepted-only filter.

## 3. Rating-preview byte path (backend)

- [x] 3.1 Add a module operation `rating_preview_bytes(user_id, catalog_id)` that returns bytes for a `pending` or `accepted` score to a signed-in caller, refusing `rejected`/unknown — distinct from `get_catalog_bytes` (player-open), which stays accepted-only.
- [x] 3.2 Add the proto RPC (e.g. `GetRatingPreviewBytes`) + gRPC handler (`backend/music/proto/score.proto`, `src/grpc.rs`), authenticated-only; regenerate bindings. Do NOT relax `GetCatalogScoreBytes` or `SaveCatalogScore` (they remain accepted-only).
- [x] 3.3 Reuse the same `.mxl` decode path as `get_catalog_bytes` so the preview gets canonical MusicXML.

## 4. App: deck previews pending via the rating-preview path (apps/music)

- [x] 4.1 Add a `ratingPreviewBytes(catalogId)` call to the catalog/rating service seam (`apps/music/lib/services/`) over the new RPC.
- [x] 4.2 Point the in-card preview (`card_preview_notifier` / `in_card_preview`) at the rating-preview fetch for deck cards, keeping the read-only game-mode render and the "listen enough → unlock rating" gate; the full player-open path is untouched.
- [x] 4.3 Surface each card's `moderation_status` (already on `CatalogHit`) to the deck state so a pending card can be labelled.

## 5. App: "potential new score" label on pending cards (apps/music)

- [x] 5.1 Add a "potential new score" badge/label on `pending` cards in `rating_card.dart`, with attribution still shown; accepted cards unchanged. Positive framing (helping evaluate a candidate), no warning tone.
- [x] 5.2 Add the localized strings (`app_en/fr/es/it.arb`) for the label.

## 6. Tests

- [x] 6.1 Rust (module + fakes): deck sources pending+accepted and never rejected; already-rated excluded; least-rated-first preserved.
- [x] 6.2 Rust: `submit_rating` accepts a pending score, refuses a rejected/unknown one, upsert stays single-row; unauthenticated rejected.
- [x] 6.3 Rust: `rating_preview_bytes` serves pending+accepted, refuses rejected/unknown/unauthenticated; `get_catalog_bytes`/save stay accepted-only (regression that pending is still refused there).
- [x] 6.4 Flutter: deck notifier sources the pending+accepted pool; the in-card preview uses the rating-preview fetch; a pending card shows the "potential new score" label (widget/state tests, mocked service).
- [x] 6.5 Coverage ≥ 80% (Rust `cargo llvm-cov` workspace, Flutter `very_good_coverage`); `cargo clippy -D warnings` + `cargo fmt` + `melos run analyze` clean.

## 7. Validate & document

- [x] 7.1 `openspec validate rate-pending-scores --strict` passes.
- [x] 7.2 Update the deck/rating doc comments and any user-facing copy so "accepted-only" statements reflect pending+accepted; note that consensus prioritises the moderator queue and never auto-validates.
