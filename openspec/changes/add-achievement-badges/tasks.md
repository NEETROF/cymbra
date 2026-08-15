## 1. Registry core (pure, host-testable)

- [x] 1.1 Create `backend/music/src/badges_core.rs`: `BadgeFamily` (play, consistency,
      ranking, contribution, curation, learning), `BadgeMetric` (extended with the play /
      consistency / ranking / contribution metrics), `LocalizedText` (the `{en, fr, es, it}`
      map), and `BadgeDef { key, family, metric, threshold, track, tier, label, description }`.
- [x] 1.2 Port the seven curation badges into `REGISTRY` with **identical keys and
      thresholds**, tagging `curator_1/2/3` and `sharp_ear_1/2` with their track + tier.
- [x] 1.3 Add the play, consistency, ranking, contribution and learning entries to
      `REGISTRY` with first-pass thresholds (design Open Questions) and their localized
      label + description.
- [x] 1.4 Add `BadgeCounters` (one value per metric) and `fn value(&self, metric)`.
- [x] 1.5 Implement the consistency folds in `badges_core`: distinct local days and longest
      consecutive-day run, from a list of local days (pure — no SQL window functions).
- [x] 1.6 Implement `fn evaluate(counters, granted) -> Vec<BadgeStanding>` producing, per
      registry entry, `earned = granted OR counter >= threshold` (design D3), the current
      value **clamped to the threshold when earned**, and the granted moment when known.
- [x] 1.7 Unit-test 1.5 and 1.6: threshold boundaries, clamping, the union rule (granted but
      counter now below threshold stays earned), local-day bucketing across a UTC boundary,
      and streak runs (empty, single day, gap, longest-in-the-middle).

## 2. Counter sourcing (repo seam)

- [x] 2.1 Create `backend/music/src/badges.rs` with the `BadgeRepo` trait: a single
      `counters(user_id) -> BadgeCounters` plus `granted_badges(user_id)` and
      `insert_grant`, reusing `music.curation_grants` (`grant_kind='badge'`).
- [x] 2.2 Implement the play + consistency counters against `music.play_sessions` (session
      count, distinct `score_id`, sessions above the accuracy threshold, and the distinct
      local-day list from `played_at` shifted by `tz_offset_minutes`), and union
      `music.practice_sessions`' local days into the **consistency** counters only — a
      scoreless measure-range run is time at the keyboard, but has no sub-score to claim a
      play counter with.
- [x] 2.3 Implement the ranking counters against `music.leaderboard_bests` (per-piece
      placements) and `music.global_season_snapshots` (closed-season standings).
- [x] 2.4 Implement the contribution counters against `music.catalog_scores`
      (`proposed_by` + accepted status) and `music.soundfonts` (`uploaded_by` +
      `moderation_status = 'accepted'`).
- [x] 2.5 Reuse the existing curation counters (rating / aligned / first-rater) as registry
      metrics rather than re-querying them a second way, and count completed courses off
      `music.course_progress` (`completed_at IS NOT NULL`) for the learning metric.
- [x] 2.6 Verify every counter query is served by an existing index; add none unless a query
      plan says otherwise, and note it in the migration if it does.
- [x] 2.7 `#[automock]` the trait and add repo tests (or a sqlx-backed integration check)
      covering the local-day shift and the accepted-only filters.

## 3. Awarding + projection module

- [x] 3.1 Create `backend/music/src/badges_module.rs`: fetch counters once, evaluate, grant
      every newly-due badge (idempotent insert), then project the standings.
- [x] 3.2 Wire grant-on-read (design D2) so a newly defined badge is awarded retroactively
      on the next read with no backfill.
- [x] 3.3 Have `CurationRewardsModule` source its badge list from the registry instead of
      `curation_rewards_core::BADGES`; delete `BADGES` / `earned_badges` from
      `curation_rewards_core.rs` once nothing references them.
- [x] 3.4 Module tests with `mockall`: idempotent re-evaluation (one grant, unchanged
      moment), retroactive award of a newly added badge, and an earned badge surviving its
      counter dropping to zero.

## 4. API

- [x] 4.1 Add `AchievementBadge` to `backend/music/proto/score.proto` — key, family, metric,
      threshold, track, tier, earned, current value, optional granted-at, and the localized
      label + description maps (JSON strings, following the `courses.title_json` precedent).
- [x] 4.2 Add `GetAchievements` (authenticated, no request fields) to `ScoreService`
      returning the full registry projected for the caller.
- [x] 4.3 Mark `CuratorRewards.badges` deprecated in the proto while keeping it populated
      with the curation subset (design D8).
- [x] 4.4 Implement the gRPC handler in `backend/music/src/grpc.rs` with auth handling
      matching `GetCuratorRewards`; add handler tests including the unauthenticated case.
- [x] 4.5 Regenerate the Dart gRPC stubs (`melos run gen-grpc`).

## 5. App — state and service seam

- [x] 5.1 Add `apps/music/lib/services/achievements_service.dart` behind an injectable
      provider, calling `GetAchievements`.
- [x] 5.2 Add the Freezed view models: `AchievementBadgeView` (with `progress`, `earnedAt`,
      `family`, `track`, `tier`) and `AchievementsView` grouped by family.
- [x] 5.3 Add `apps/music/lib/state/achievements_notifier.dart` (`@riverpod`, `AsyncValue`)
      exposing the grouped, earned-first families and collapsing each track to its highest
      earned tier plus the next locked one (design D5).
- [x] 5.4 Resolve the localized label/description client-side against the active display
      language with an `en` fallback (design D6).
- [x] 5.5 Persist a "achievements last seen" timestamp (its own key, not the curator
      activity one) and derive the **new** marker from `earnedAt` against it.
- [x] 5.6 Notifier tests with mockito-generated mocks via `ProviderScope` overrides: grouping,
      earned-first ordering, track collapsing, new-marker computation, locale fallback.

## 6. App — Achievements surface

- [x] 6.1 Add `apps/music/lib/widgets/achievements_section.dart`: families as sections,
      earned-first, per-badge icon, progress indicator on locked tiles, new marker.
- [x] 6.2 Do not render a family that has no badges, so a family may be declared ahead of
      the counter that will feed it.
- [x] 6.3 Add the badge detail sheet: what it takes, current standing, earned date, and the
      full tier ladder for a track with earned tiers marked.
- [x] 6.4 Mount the section in `ProfileScreen` and **remove** `_BadgeGrid` / `_BadgeTile` /
      `_badgeLabel` / `_metricLabel` and the badges title from
      `apps/music/lib/widgets/curator_rewards_section.dart`.
- [x] 6.5 Isolate the "mark achievements seen" side effect in its own listener widget
      (architecture rule 4), as `_CuratorSeenListener` does.
- [x] 6.6 Add the l10n keys for the section chrome (title, family names, "N/M", "earned on",
      "new") to all supported locales — badge names themselves come from the server.
- [x] 6.7 Widget tests: grouped grid, locked tile progress, empty family hidden, detail sheet
      ladder, new marker clearing, and the loading/error states.
- [x] 6.8 Update `apps/music/test/screens/profile_screen_test.dart` and
      `curator_rewards_screens_test.dart` for the moved grid.

## 7. Gates

- [x] 7.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 7.2 `cargo llvm-cov --workspace --fail-under-lines 80`.
- [x] 7.3 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`, then
      `melos run analyze`, `dart format`, and `dart run custom_lint`.
- [x] 7.4 `flutter test --coverage --exclude-tags golden` and confirm the coverage gate.
- [x] 7.5 `openspec validate add-achievement-badges --strict`.
- [x] 7.6 Manual on-device pass: a play-only account sees earned play/consistency badges, an
      existing curator account keeps every badge it had, and the detail sheet renders a track
      ladder correctly.
