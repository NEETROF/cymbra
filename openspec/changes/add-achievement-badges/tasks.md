## 1. Registry core (pure, host-testable)

- [ ] 1.1 Create `backend/music/src/badges_core.rs`: `BadgeFamily` (play, consistency,
      ranking, contribution, curation, learning), `BadgeMetric` (extended with the play /
      consistency / ranking / contribution metrics), `LocalizedText` (the `{en, fr, es, it}`
      map), and `BadgeDef { key, family, metric, threshold, track, tier, label, description }`.
- [ ] 1.2 Port the seven curation badges into `REGISTRY` with **identical keys and
      thresholds**, tagging `curator_1/2/3` and `sharp_ear_1/2` with their track + tier.
- [ ] 1.3 Add the play, consistency, ranking and contribution entries to `REGISTRY` with
      first-pass thresholds (design Open Questions) and their localized label + description.
- [ ] 1.4 Add `BadgeCounters` (one value per metric) and `fn value(&self, metric)`.
- [ ] 1.5 Implement the consistency folds in `badges_core`: distinct local days and longest
      consecutive-day run, from a list of local days (pure — no SQL window functions).
- [ ] 1.6 Implement `fn evaluate(counters, granted) -> Vec<BadgeStanding>` producing, per
      registry entry, `earned = granted OR counter >= threshold` (design D3), the current
      value **clamped to the threshold when earned**, and the granted moment when known.
- [ ] 1.7 Unit-test 1.5 and 1.6: threshold boundaries, clamping, the union rule (granted but
      counter now below threshold stays earned), local-day bucketing across a UTC boundary,
      and streak runs (empty, single day, gap, longest-in-the-middle).

## 2. Counter sourcing (repo seam)

- [ ] 2.1 Create `backend/music/src/badges.rs` with the `BadgeRepo` trait: a single
      `counters(user_id) -> BadgeCounters` plus `granted_badges(user_id)` and
      `insert_grant`, reusing `music.curation_grants` (`grant_kind='badge'`).
- [ ] 2.2 Implement the play + consistency counters against `music.play_sessions` (session
      count, distinct `score_id`, sessions above the accuracy threshold, and the distinct
      local-day list from `played_at` shifted by `tz_offset_minutes`).
- [ ] 2.3 Implement the ranking counters against `music.leaderboard_bests` (per-piece
      placements) and `music.global_season_snapshots` (closed-season standings).
- [ ] 2.4 Implement the contribution counters against `music.catalog_scores`
      (`proposed_by` + accepted status) and `music.soundfonts` (`uploaded_by` +
      `moderation_status = 'accepted'`).
- [ ] 2.5 Reuse the existing curation counters (rating / aligned / first-rater) as registry
      metrics rather than re-querying them a second way.
- [ ] 2.6 Verify every counter query is served by an existing index; add none unless a query
      plan says otherwise, and note it in the migration if it does.
- [ ] 2.7 `#[automock]` the trait and add repo tests (or a sqlx-backed integration check)
      covering the local-day shift and the accepted-only filters.

## 3. Awarding + projection module

- [ ] 3.1 Create `backend/music/src/badges_module.rs`: fetch counters once, evaluate, grant
      every newly-due badge (idempotent insert), then project the standings.
- [ ] 3.2 Wire grant-on-read (design D2) so a newly defined badge is awarded retroactively
      on the next read with no backfill.
- [ ] 3.3 Have `CurationRewardsModule` source its badge list from the registry instead of
      `curation_rewards_core::BADGES`; delete `BADGES` / `earned_badges` from
      `curation_rewards_core.rs` once nothing references them.
- [ ] 3.4 Module tests with `mockall`: idempotent re-evaluation (one grant, unchanged
      moment), retroactive award of a newly added badge, and an earned badge surviving its
      counter dropping to zero.

## 4. API

- [ ] 4.1 Add `AchievementBadge` to `backend/music/proto/score.proto` — key, family, metric,
      threshold, track, tier, earned, current value, optional granted-at, and the localized
      label + description maps (JSON strings, following the `courses.title_json` precedent).
- [ ] 4.2 Add `GetAchievements` (authenticated, no request fields) to `ScoreService`
      returning the full registry projected for the caller.
- [ ] 4.3 Mark `CuratorRewards.badges` deprecated in the proto while keeping it populated
      with the curation subset (design D8).
- [ ] 4.4 Implement the gRPC handler in `backend/music/src/grpc.rs` with auth handling
      matching `GetCuratorRewards`; add handler tests including the unauthenticated case.
- [ ] 4.5 Regenerate the Dart gRPC stubs (`melos run gen-grpc`).

## 5. App — state and service seam

- [ ] 5.1 Add `apps/music/lib/services/achievements_service.dart` behind an injectable
      provider, calling `GetAchievements`.
- [ ] 5.2 Add the Freezed view models: `AchievementBadgeView` (with `progress`, `earnedAt`,
      `family`, `track`, `tier`) and `AchievementsView` grouped by family.
- [ ] 5.3 Add `apps/music/lib/state/achievements_notifier.dart` (`@riverpod`, `AsyncValue`)
      exposing the grouped, earned-first families and collapsing each track to its highest
      earned tier plus the next locked one (design D5).
- [ ] 5.4 Resolve the localized label/description client-side against the active display
      language with an `en` fallback (design D6).
- [ ] 5.5 Persist a "achievements last seen" timestamp (its own key, not the curator
      activity one) and derive the **new** marker from `earnedAt` against it.
- [ ] 5.6 Notifier tests with mockito-generated mocks via `ProviderScope` overrides: grouping,
      earned-first ordering, track collapsing, new-marker computation, locale fallback.

## 6. App — Achievements surface

- [ ] 6.1 Add `apps/music/lib/widgets/achievements_section.dart`: families as sections,
      earned-first, per-badge icon, progress indicator on locked tiles, new marker.
- [ ] 6.2 Do not render a family that has no badges (the learning family ships empty).
- [ ] 6.3 Add the badge detail sheet: what it takes, current standing, earned date, and the
      full tier ladder for a track with earned tiers marked.
- [ ] 6.4 Mount the section in `ProfileScreen` and **remove** `_BadgeGrid` / `_BadgeTile` /
      `_badgeLabel` / `_metricLabel` and the badges title from
      `apps/music/lib/widgets/curator_rewards_section.dart`.
- [ ] 6.5 Isolate the "mark achievements seen" side effect in its own listener widget
      (architecture rule 4), as `_CuratorSeenListener` does.
- [ ] 6.6 Add the l10n keys for the section chrome (title, family names, "N/M", "earned on",
      "new") to all supported locales — badge names themselves come from the server.
- [ ] 6.7 Widget tests: grouped grid, locked tile progress, empty family hidden, detail sheet
      ladder, new marker clearing, and the loading/error states.
- [ ] 6.8 Update `apps/music/test/screens/profile_screen_test.dart` and
      `curator_rewards_screens_test.dart` for the moved grid.

## 7. Gates

- [ ] 7.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 7.2 `cargo llvm-cov --workspace --fail-under-lines 80`.
- [ ] 7.3 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`, then
      `melos run analyze`, `dart format`, and `dart run custom_lint`.
- [ ] 7.4 `flutter test --coverage --exclude-tags golden` and confirm the coverage gate.
- [ ] 7.5 `openspec validate add-achievement-badges --strict`.
- [ ] 7.6 Manual on-device pass: a play-only account sees earned play/consistency badges, an
      existing curator account keeps every badge it had, and the detail sheet renders a track
      ladder correctly.
