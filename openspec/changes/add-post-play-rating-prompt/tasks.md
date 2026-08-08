## 1. Backend — the caller's own rating of a score

- [ ] 1.1 Add `GetMyScoreRatingRequest { catalog_id }` / `GetMyScoreRatingResponse { rated, optional verdict, optional stars }` and the `GetMyScoreRating` rpc to `ScoreService` in `backend/music/proto/score.proto`, documenting the fail-closed behaviour (unknown/`rejected` → `rated = false`, no existence oracle).
- [ ] 1.2 Add a `rating_for(user_id, catalog_id) -> Option<Rating>` read to the rating port + Postgres adapter in `backend/music/src/score_rating.rs`, with the in-memory fake kept in sync.
- [ ] 1.3 Expose it on the music module as `my_score_rating(user_id, catalog_id)`, reusing `is_pending_or_accepted` so an unknown or `rejected` id returns "not rated" rather than an error.
- [ ] 1.4 Wire the gRPC handler in `backend/music/src/grpc.rs`: identity from the interceptor (never the body), `UNAUTHENTICATED` without one.
- [ ] 1.5 Tests: rated → verdict/stars returned; un-rated → not rated; another user's rating invisible; `rejected`/unknown → not rated (no distinct error); unauthenticated rejected.

## 2. Backend — playing counts as coverage engagement

- [ ] 2.1 Add `catalog_bytes_for_player(user_id, catalog_id)` to `backend/music/src/module.rs`, mirroring `rating_preview_bytes`: best-effort `record_engagement` then delegate to the existing `get_catalog_bytes` (signature unchanged, other callers untouched).
- [ ] 2.2 Point the `GetCatalogScoreBytes` handler at the new method so a player open records engagement.
- [ ] 2.3 Record engagement on play-session ingest when the session's `score_id` resolves to a catalog score, wired at the gRPC composition seam so `play_module` keeps no rewards dependency.
- [ ] 2.4 Tests: player open records engagement; ingest of an offline-cached play records it; recording is idempotent across preview + play; a failing `record_engagement` does not fail the open or the ingest; a rating after a play-only engagement earns coverage points.
- [ ] 2.5 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo llvm-cov --workspace --fail-under-lines 80` green.

## 3. App — service seam and generated stubs

- [ ] 3.1 Regenerate the Dart gRPC stubs (`melos run gen-grpc`) for the new rpc.
- [ ] 3.2 Extend `RatingService` in `apps/music/lib/services/rating_service.dart` with `myRating({required String catalogId})` returning a small `MyRating` value (rated + optional verdict/stars), implemented on `GrpcRatingService` through `authedCall`.
- [ ] 3.3 Extract the star→verdict derivation out of `rating_deck_notifier.dart` into a shared pure function and make the deck use it, so the deck and the prompt cannot drift.

## 4. App — eligibility core (pure, host-testable)

- [ ] 4.1 Add a pure `shouldPromptRating(...)` predicate over `(signedIn, catalogId, ratedState, alreadyOffered, playback)` mirroring `shouldInviteToRate`, with the "played enough" term as `fraction >= 0.15 || playedFor >= 20s` and both thresholds as named constants.
- [ ] 4.2 Add the pure LRU insert/trim for the offered-score memory (ordered ids, cap 200, oldest dropped).
- [ ] 4.3 Unit tests for both: every eligibility term individually false; unknown rated state suppresses; end-of-run always satisfies the playback term; LRU keeps insertion order, dedupes, and trims at the cap.

## 5. App — playback high-water mark

- [ ] 5.1 Track the furthest `elapsedMs` reached and the cumulative playing duration in the player notifier/state, reset on score change and on restart-from-top.
- [ ] 5.2 Tests: the high-water mark survives pause/seek-back, resets on a new score, and a never-started player reports zero.

## 6. App — the prompt notifier

- [ ] 6.1 Add a `PostPlayRatingPrompt` Riverpod notifier owning the resolved rated state, the offered-score memory (through the preferences seam), eligibility, and a `submit(stars)` action going through `ratingServiceProvider` — no widget touches the service.
- [ ] 6.2 Resolve the caller's rating once when the player opens a catalog score, without awaiting it on the play path; a failed or pending read means "unknown" → no prompt.
- [ ] 6.3 Record the catalog id in the offered memory when the prompt is **shown**, so a dismissal and a rating retire it alike; persist best-effort.
- [ ] 6.4 Keep submission failures inside the notifier's `AsyncValue`, surfacing at most a localized message (no gRPC/exception string).
- [ ] 6.5 Notifier tests with a mockito-generated `RatingService` mock injected via `ProviderScope` overrides: eligible → prompt; already rated → no prompt; already offered → no prompt; guest → no prompt; submit sends the derived verdict; a failed submit leaves a localized error and no crash.

## 7. App — the shared rating widget and the two surfaces

- [ ] 7.1 Build a compact shared star-row widget (1–5 stars, thanks state after submit) used by both surfaces, with no listening lock.
- [ ] 7.2 Mount it in `session_summary_modal.dart` between the scrollable stats and the pinned actions; rating must not pop the dialog and must not displace the action buttons.
- [ ] 7.3 Add the localized strings to the four ARB files (`app_en`, `app_fr`, `app_es`, `app_it`) and regenerate.
- [ ] 7.4 Funnel both exit paths in `player_screen.dart` (top-bar back button and the system back gesture) into one `_requestExit()`; make `PopScope.canPop` false only while a prompt is pending, and pop unconditionally once the sheet resolves.
- [ ] 7.5 Show the early-exit prompt as a dismissible `showModalBottomSheet` with no "stay" action — rate, close, barrier tap, and back all leave the player.
- [ ] 7.6 Widget tests (summary): the row appears for an eligible score and is absent otherwise; rating does not dismiss the modal; see-mistakes/retry/quit still work with and without rating; actions stay reachable at a phone-landscape size.
- [ ] 7.7 Widget tests (exit): eligible → sheet shown then the player is left; rate → submitted and left; dismiss → left with no rating; a failed submit still leaves; ineligible → leaves immediately with no sheet; a second exit intent pops without re-prompting.

## 8. Gates

- [ ] 8.1 `melos run analyze`, `dart format`, and `dart run custom_lint` clean.
- [ ] 8.2 `flutter test --coverage --exclude-tags golden` green with line coverage ≥ 80%.
- [ ] 8.3 `openspec validate add-post-play-rating-prompt --strict` passes.
- [ ] 8.4 Manual pass on a device: play a catalog score to the end and rate from the summary; abandon another mid-piece and rate from the sheet; replay both and confirm neither prompts again; confirm a bundled score never prompts and a signed-out user never prompts.
