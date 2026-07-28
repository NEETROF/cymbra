## 1. Backend — deck sourcing

- [x] 1.1 Add a `rating_deck(user_id, limit, offset)` read to the catalog port
      (`CatalogSearchRepo`): the caller's **un-rated** `accepted` (piano) scores,
      `ORDER BY rating_count ASC, id ASC`, paginated. Pg: `LEFT JOIN score_ratings`
      on `(catalog_score_id, user_id)` with `r.user_id IS NULL`; Fake: mirror via a
      shared `FakeScoreRatingRepo` view.
- [x] 1.2 Add `ScoreModule::list_rating_deck(user_id, limit, offset)` clamping the
      page size, and a `ListRatingDeck` RPC (proto + Rust + Dart bindings).
- [x] 1.3 gRPC handler: authenticated caller only; maps to the module read.

## 2. App — deck consumes the new source

- [x] 2.1 Add a `ratingDeck({limit, offset})` call to the catalog service seam
      (maps the wire hits), with a fake for tests.
- [x] 2.2 Point the deck notifier at `ratingDeck` instead of `search`.

## 3. App — invite stop condition

- [x] 3.1 Persist a dismissal count in `RatingActivity`; after a configured number
      of dismissals `ratingInviteVisible` returns false permanently. A rating still
      resets the snooze window.

## 4. Tests & verification

- [x] 4.1 Rust: deck read excludes rated, orders least-rated first, empties when
      all rated; unauthenticated rejected. `cargo llvm-cov ... --fail-under-lines 80`.
- [x] 4.2 Flutter: notifier sources via the deck seam (no repeats); invite stops
      after N dismissals. `flutter test --coverage` ≥ 80%.
- [x] 4.3 `melos run analyze` + `dart format` + `cargo fmt`/`clippy` clean;
      regenerate codegen (`build_runner`, proto/frb) as needed.
- [x] 4.4 `openspec validate improve-rating-deck-sourcing --strict` passes.
</content>
