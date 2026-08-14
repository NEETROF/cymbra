## 1. Award math (pure, host-testable)

- [ ] 1.1 Create `backend/music/src/play_rewards_core.rs` with `PlayRewardConfig`: the
      accuracy floor, the per-piece diminishing bands (`(times_already_paid_inclusive,
      points)`, the `coverage_bands` shape re-aimed), the difficulty weights + neutral
      fallback, the daily cap, and the flat practice amount. All defaults in `Default`.
- [ ] 1.2 Implement `fn performance_award(accuracy, times_already_paid, level, already_today,
      cfg) -> i64`: zero below the floor, else the band for `times_already_paid`, scaled by
      the difficulty weight, clamped by the remaining daily headroom. Never negative.
- [ ] 1.3 Implement `fn practice_award(already_awarded_today, cfg) -> i64` — the flat amount
      once, zero thereafter.
- [ ] 1.4 Reuse the leaderboard's difficulty weighting rather than a second scale: read the
      weights from `GlobalConfig::level_weights` (or lift the shared function), with an
      unknown/absent level weighing neutral, never zero.
- [ ] 1.5 Unit-test 1.2 and 1.3 exhaustively: floor boundary (just below / exactly at), each
      band edge, the approach to zero, weight applied per level + neutral fallback, cap
      clamping (headroom smaller than the award, at the cap, past the cap), and that no input
      combination yields a negative award.
- [ ] 1.6 Add a test that states the anti-grind property directly: replaying one piece all
      day earns strictly less than playing the same number of distinct pieces, and never
      exceeds the daily cap.

## 2. Ledger + repo

- [ ] 2.1 Migration: extend the `award_kind` CHECK on `music.curation_points` with
      `performance` and `practice`; add nullable `award_key TEXT` and the partial unique
      index `(user_id, award_key) WHERE award_key IS NOT NULL` (the shape
      `curation_points_coverage_once_idx` already uses). Additive — document that existing
      rows carry NULL and no existing read changes.
- [ ] 2.2 Add `AwardKind::Performance` / `AwardKind::Practice` with their persisted strings.
- [ ] 2.3 Extend `CurationRewardsRepo`: append an award carrying an `award_key` (returning
      whether it was newly inserted), count how many times a piece has already paid a user,
      and read what play has paid today. Implement on `PgCurationRewardsRepo` and the
      in-memory fake, mirroring the existing coverage-once semantics.
- [ ] 2.4 Verify each new query is served by an existing index (`curation_points_user_idx`,
      plus the new partial unique index for the per-piece count); note it in the migration if
      one has to be added.
- [ ] 2.5 Repo tests on the fake: the same `award_key` inserts once, two different keys both
      insert, and the per-piece count only counts `performance` rows.

## 3. Awarding module

- [ ] 3.1 Extend `CurationRewardsSink` with `award_performance(user, piece, accuracy, level,
      session_id)` and `award_practice(user, local_day)`, both returning the points actually
      awarded (0 = gated out / capped / already paid).
- [ ] 3.2 Implement them on `CurationRewardsModule` over the pure math + repo, keyed on the
      `award_key` (session id / local day) so a retry is a no-op.
- [ ] 3.3 Grant any newly-due badge after a play award, the way the rating paths already do —
      a level or milestone crossed by playing must not wait for the next profile read.
- [ ] 3.4 Module tests against the fake: a good run pays; the same session id replayed pays
      once; the same piece pays less each time and eventually zero; the cap clamps and the
      next day restores; a sub-floor run pays zero; practice pays once per local day.

## 4. Ingest wiring

- [ ] 4.1 Call the two sink methods from `PlayGrpc` where `record_engagement` is already
      called, so `PlayModule` stays unaware of rewards.
- [ ] 4.2 Resolve the piece's catalog difficulty through the already-wired
      `CatalogSearchRepo` (the same seam that tells a catalog score from an upload today); an
      unresolvable piece weighs neutral.
- [ ] 4.3 Make the award **best-effort with respect to the ack**: a failure logs and is
      swallowed, the session is still stored and acknowledged. Add a test proving a failing
      award does not fail the ingest.
- [ ] 4.4 Derive the practice award's local day from `practiced_at` + `tz_offset_minutes`,
      reusing `play_core::local_day` rather than a second implementation.

## 5. API

- [ ] 5.1 Add `points_awarded` to `RecordPlaySessionResponse` and `RecordPracticeResponse`
      (additive fields on existing responses — no new RPC).
- [ ] 5.2 Populate them from the sink's return value; a sink that is not wired reports 0.
- [ ] 5.3 Handler tests: a paying session reports its amount, a sub-floor session reports 0,
      a replayed session reports 0 the second time.
- [ ] 5.4 Regenerate the Dart gRPC stubs (`melos run gen-grpc`).

## 6. App

- [ ] 6.1 Carry `pointsAwarded` through the play-sync outbox path into the session-summary
      state (it arrives on the ack the client already awaits).
- [ ] 6.2 Show the "+N" cue on the session summary when the amount is positive, and show
      nothing when it is zero — reusing the rating action's existing cue treatment.
- [ ] 6.3 Reuse the existing reward-celebration path for a level crossed by playing.
- [ ] 6.4 Add the l10n keys for the cue to all supported locales.
- [ ] 6.5 Widget tests: cue shown for a positive amount, absent for zero, and the summary
      otherwise unchanged.
- [ ] 6.6 Notifier tests with mockito-generated mocks via `ProviderScope` overrides for the
      awarded amount reaching the summary state.

## 7. Gates

- [ ] 7.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 7.2 `cargo llvm-cov --workspace --fail-under-lines 80`.
- [ ] 7.3 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`, then
      `melos run analyze`, `dart format`, and `dart run custom_lint`.
- [ ] 7.4 `flutter test --coverage --exclude-tags golden` and confirm the coverage gate.
- [ ] 7.5 `openspec validate add-play-rewards --strict`.
- [ ] 7.6 Manual on-device pass: a good run shows its "+N"; replaying the same piece visibly
      pays less; a deliberately bad run pays nothing; a practice run pays once and a second
      practice the same day pays nothing.
