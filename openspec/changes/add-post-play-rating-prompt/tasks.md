## 1. Backend — the caller's own rating of a score

- [x] 1.1 Add `GetMyScoreRatingRequest { catalog_id }` / `GetMyScoreRatingResponse { rated, optional verdict, optional stars }` and the `GetMyScoreRating` rpc to `ScoreService` in `backend/music/proto/score.proto`, documenting the fail-closed behaviour (unknown/`rejected` → `rated = false`, no existence oracle).
- [x] 1.2 Add a `rating_for(user_id, catalog_id) -> Option<Rating>` read to the rating port + Postgres adapter in `backend/music/src/score_rating.rs`, with the in-memory fake kept in sync.
- [x] 1.3 Expose it on the music module as `my_score_rating(user_id, catalog_id)`, reusing `is_pending_or_accepted` so an unknown or `rejected` id returns "not rated" rather than an error.
- [x] 1.4 Wire the gRPC handler in `backend/music/src/grpc.rs`: identity from the interceptor (never the body), `UNAUTHENTICATED` without one.
- [x] 1.5 Tests: rated → verdict/stars returned; un-rated → not rated; another user's rating invisible; `rejected`/unknown → not rated (no distinct error); unauthenticated rejected.

## 2. Backend — playing counts as coverage engagement

- [x] 2.1 Add `catalog_bytes_for_player(user_id, catalog_id)` to `backend/music/src/module.rs`, mirroring `rating_preview_bytes`: best-effort `record_engagement` then delegate to the existing `get_catalog_bytes` (signature unchanged, other callers untouched).
- [x] 2.2 Point the `GetCatalogScoreBytes` handler at the new method so a player open records engagement.
- [x] 2.3 Record engagement on play-session ingest when the session's `score_id` resolves to a catalog score, wired at the gRPC composition seam so `play_module` keeps no rewards dependency.
- [x] 2.4 Tests: player open records engagement; ingest of an offline-cached play records it; recording is idempotent across preview + play; a failing `record_engagement` does not fail the open or the ingest; a rating after a play-only engagement earns coverage points.
- [x] 2.5 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo llvm-cov --workspace --fail-under-lines 80` green.

## 3. App — service seam and generated stubs

- [x] 3.1 Regenerate the Dart gRPC stubs (`melos run gen-grpc`) for the new rpc.
- [x] 3.2 Extend `RatingService` in `apps/music/lib/services/rating_service.dart` with `myRating({required String catalogId})` returning a small `MyRating` value (rated + optional verdict/stars), implemented on `GrpcRatingService` through `authedCall`.
- [x] 3.3 Extract the star→verdict derivation out of `rating_deck_notifier.dart` into a shared pure function and make the deck use it, so the deck and the prompt cannot drift.

## 4. App — eligibility core (pure, host-testable)

- [x] 4.1 Add a pure `playedNoteFraction(List<TimedNote> notes, double furthestElapsedMs)` returning the share of the score's notes the playhead has passed — binary search over the already-sorted `notes`, counted over the whole score (not `visibleNotes`), 0 for an empty score.
- [x] 4.2 Add a pure `shouldPromptRating(...)` predicate over `(signedIn, catalogId, ratedState, declined, playedNoteFraction)` mirroring `shouldInviteToRate`, with the played-enough term as `fraction >= 0.25` and the threshold as a named constant.
- [x] 4.3 Add the pure LRU insert/trim for the refused-score memory (ordered ids, cap 200, oldest dropped).
- [x] 4.4 Unit tests: `playedNoteFraction` at 0 / boundary / all notes, on an empty score, with chords sharing a `startMs`, and unaffected by hand muting; every eligibility term individually false; unknown rated state suppresses; end-of-run satisfies the playback term; LRU keeps insertion order, dedupes, and trims at the cap.

## 5. App — playback high-water mark

- [x] 5.1 Track the furthest `elapsedMs` reached in the player notifier/state (monotonic), reset **only on score change** — a restart, a hand switch, or a loop wrap keeps it, because the user did hear that music. On a loop wrap credit `endMs`, not the wrapped-to position.
- [x] 5.2 Tests: the high-water mark survives pause, restart, and a loop wrap, resets on a new score, and a never-started player reports zero.

## 6. App — the prompt notifier

- [x] 6.1 Add a `PostPlayRatingPrompt` Riverpod notifier owning the resolved rated state, the offered-score memory (through the preferences seam), eligibility, and a `submit(stars)` action going through `ratingServiceProvider` — no widget touches the service.
- [x] 6.2 Resolve the caller's rating once when the player opens a catalog score, without awaiting it on the play path; a failed or pending read means "unknown" → no prompt.
- [x] 6.3 Record a refusal ONLY on an explicit "not this one" — never on mere display, and not needed on a rating (the score becomes rated); persist best-effort. Both surfaces expose the refusal control.
- [x] 6.4 Keep submission failures inside the notifier's `AsyncValue`, surfacing at most a localized message (no gRPC/exception string).
- [x] 6.5 Notifier tests with a mockito-generated `RatingService` mock injected via `ProviderScope` overrides: eligible → prompt; already rated → no prompt; already offered → no prompt; guest → no prompt; submit sends the derived verdict; a failed submit leaves a localized error and no crash.

## 7. App — the shared rating widget and the two surfaces

- [x] 7.1 Build a compact shared star-row widget (1–5 stars, thanks state after submit, explicit refusal control) used by both surfaces, with no listening lock.
- [x] 7.2 Mount it in `session_summary_modal.dart` between the scrollable stats and the pinned actions; rating must not pop the dialog and must not displace the action buttons.
- [x] 7.3 Add the localized strings to the four ARB files (`app_en`, `app_fr`, `app_es`, `app_it`) and regenerate.
- [x] 7.4 Funnel both exit paths in `player_screen.dart` (top-bar back button and the system back gesture) into one `_requestExit()`; make `PopScope.canPop` false only while a prompt is pending, and pop unconditionally once the sheet resolves.
- [x] 7.5 Show the early-exit prompt as a dismissible `showModalBottomSheet` with no "stay" action — rate, close, barrier tap, and back all leave the player.
- [x] 7.6 Widget tests (summary): the row appears for an eligible score and is absent otherwise; rating does not dismiss the modal; see-mistakes/retry/quit still work with and without rating; actions stay reachable at a phone-landscape size.
- [x] 7.7 Widget tests (exit): eligible → sheet shown then the player is left; rate → submitted and left; dismiss → left with no rating; a failed submit still leaves; ineligible → leaves immediately with no sheet; a second exit intent pops without re-prompting.

## 8. Gates

- [x] 8.1 `melos run analyze`, `dart format`, and `dart run custom_lint` clean.
- [x] 8.2 `flutter test --coverage --exclude-tags golden` green with line coverage ≥ 80%.
- [x] 8.3 `openspec validate add-post-play-rating-prompt --strict` passes.
- [x] 8.4 Manual pass on a device: play a catalog score to the end and rate from the summary; abandon another past a quarter of its notes and rate from the sheet; leave a third after a handful of notes and confirm no prompt; on a fourth, close the summary WITHOUT answering and confirm the next run asks again; refuse it explicitly and confirm it never asks again; confirm a bundled score never prompts and a signed-out user never prompts.
