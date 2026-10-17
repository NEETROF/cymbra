# Tasks

## 1. Contract

- [x] 1.1 Add `best_overall_sync_pct` to `LeaderboardStanding` in `leaderboard.proto` (new field number) and map the stored best into it in the Rust standings read; verify `buf breaking` against main passes, generated code compiles, and a mockall-backed RPC test returns the value. *(`MyStanding.best_overall_sync_pct`; the overall best is now stored beside the per-mode bests — migration 0034, see D2. Module test on the fake repo.)*

## 2. Derivation and state

- [x] 2.1 Write `state/piece_stars_core.dart` with `starsFor(bestPct)`; verify unit tests on 59/60/79/80/94/95 and a null best.
- [x] 2.2 Add `pieceStarsProvider(scoreId)` reading `MyStandings` and requesting the piece like the rank badge does; verify provider tests for loaded, missing and refreshed-after-delivery standings. *(`pieceStarsProvider(pieceId, catalogId)` merging the server standing with the device record; the standing seen before the delivery reset is kept via `MyStandings.known`.)*

## 3. Surfaces

- [x] 3.1 Build `widgets/star_row.dart` (filled/outlined, compact and large); verify widget tests for 0 to 3 stars.
- [x] 3.2 Show it on score cards (signed in only), the path's piece nodes and the daily-session pieces; verify widget tests per surface including the guest case. *(Path nodes and today's session read the lesson's best piece run — see D1.)*
- [x] 3.3 Show this run's stars in the session summary, hidden for practice runs, and raise the new-star cue once from a listener comparing to the pre-run standing; verify summary tests for new stars, same stars, first run and practice. *(Inline moment in the summary; practice runs open no summary.)*
- [x] 3.4 Add the four-locale accessibility labels ("2 of 3 stars"); verify ARB alignment and `dart run custom_lint`.

## 4. Integration

- [ ] 4.1 Integration test: play the fixture score to a known percentage, see the summary stars, return to the library and see the card updated after delivery; verify `melos run integration` passes. *(Not run: the integration suite plays offline as a guest — no delivery, no signed-in card. The summary and card paths are covered by widget tests; left for a signed-in dogfood pass.)*
