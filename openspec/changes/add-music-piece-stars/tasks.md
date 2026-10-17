# Tasks

## 1. Contract

- [ ] 1.1 Add `best_overall_sync_pct` to `LeaderboardStanding` in `leaderboard.proto` (new field number) and map the stored best into it in the Rust standings read; verify `buf breaking` against main passes, generated code compiles, and a mockall-backed RPC test returns the value.

## 2. Derivation and state

- [ ] 2.1 Write `state/piece_stars_core.dart` with `starsFor(bestPct)`; verify unit tests on 59/60/79/80/94/95 and a null best.
- [ ] 2.2 Add `pieceStarsProvider(scoreId)` reading `MyStandings` and requesting the piece like the rank badge does; verify provider tests for loaded, missing and refreshed-after-delivery standings.

## 3. Surfaces

- [ ] 3.1 Build `widgets/star_row.dart` (filled/outlined, compact and large); verify widget tests for 0 to 3 stars.
- [ ] 3.2 Show it on score cards (signed in only), the path's piece nodes and the daily-session pieces; verify widget tests per surface including the guest case.
- [ ] 3.3 Show this run's stars in the session summary, hidden for practice runs, and raise the new-star cue once from a listener comparing to the pre-run standing; verify summary tests for new stars, same stars, first run and practice.
- [ ] 3.4 Add the four-locale accessibility labels ("2 of 3 stars"); verify ARB alignment and `dart run custom_lint`.

## 4. Integration

- [ ] 4.1 Integration test: play the fixture score to a known percentage, see the summary stars, return to the library and see the card updated after delivery; verify `melos run integration` passes.
