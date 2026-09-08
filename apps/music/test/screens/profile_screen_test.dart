// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/profile_screen.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/achievements_service.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/global_leaderboard_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/profile_notifier.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/oidc_token_source.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/usage_consent.dart';
import 'package:music/widgets/play_heatmap.dart';

import '../support/auth_fakes.dart';
import '../support/global_leaderboard_fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

PlayActivity _activity() => PlayActivity(
  days: [DayActivity(day: DateTime(2024, 6, 13), count: 3, avgSyncPct: 84)],
  totalSessions: 5,
);

/// The own-profile now embeds the curator-rewards section; this fake seam lets the
/// profile tests resolve it without a live backend (returns an empty standing).
class _FakeCuratorRewards implements CuratorRewardsService {
  @override
  Future<CuratorRewardsView> getRewards() async => const CuratorRewardsView(
    lifetimePoints: 0,
    spendableBalance: 0,
    level: 0,
    levelFloor: 0,
    nextLevelAt: 50,
    totalRatings: 0,
    coverageContribution: 0,
    alignmentRate: 0,
    badges: [],
    recent: [],
  );

  @override
  Future<List<RewardShopItemView>> listShop() async => const [];

  @override
  Future<RedeemResultView> redeem(String rewardKey) async =>
      const RedeemResultView(owned: true, newBalance: 0);
}

/// The own-profile also embeds the Achievements section (change: add-achievement-
/// badges). An empty registry keeps these tests off the real gRPC channel while
/// still rendering the section (which then draws no family).
class _FakeAchievements implements AchievementsService {
  @override
  Future<List<AchievementBadgeView>> getAchievements() async => const [];
}

/// Wrap [child] in a root [ProviderContainer] (via [UncontrolledProviderScope]),
/// the repo's widget-test convention — overriding on a root container avoids the
/// `scoped_providers_should_specify_dependencies` lint that a nested
/// `ProviderScope(overrides:)` would trip.
Widget _scope(List<Override> overrides, Widget child) {
  final container = ProviderContainer(overrides: overrides);
  addTearDown(container.dispose);
  return UncontrolledProviderScope(container: container, child: child);
}

Widget _harness({
  required String targetId,
  required PlayerProfile profile,
  String? currentUserId,
  String? screenUserId,
}) => _scope([
  currentUserIdProvider.overrideWithValue(currentUserId),
  playerProfileProvider(targetId).overrideWith((ref) async => profile),
  playActivityProvider(targetId).overrideWith((ref) async => _activity()),
  // The own-profile embeds the curator-rewards section (change: add-curation-
  // rewards); feed it a fake seam so the section resolves without a backend.
  curatorRewardsServiceProvider.overrideWithValue(_FakeCuratorRewards()),
  achievementsServiceProvider.overrideWithValue(_FakeAchievements()),
  // The own-profile also embeds the global standing (change: add-global-
  // leaderboard); a fake seam keeps the test off the real gRPC channel.
  globalLeaderboardServiceProvider.overrideWithValue(
    FakeGlobalLeaderboardService(),
  ),
  preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
], localizedApp(ProfileScreen(userId: screenUserId)));

void main() {
  testWidgets('another player\'s profile shows only public fields', (
    tester,
  ) async {
    await tester.pumpWidget(
      _harness(
        targetId: 'other',
        screenUserId: 'other',
        currentUserId: 'me',
        profile: const PlayerProfile(
          userId: 'other',
          handle: 'bob',
          displayName: null,
          visibility: 'public',
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Public fields: handle + heatmap + songs-played total.
    expect(find.text('@bob'), findsOneWidget);
    expect(find.byType(PlayHeatmap), findsOneWidget);
    expect(find.text('5 songs played'), findsOneWidget);
    // The visibility control is owner-only — never shown for another player.
    expect(find.byKey(const Key('profile-visibility')), findsNothing);
  });

  testWidgets('own profile shows the visibility control + go-public hint', (
    tester,
  ) async {
    await tester.pumpWidget(
      _harness(
        targetId: 'me',
        screenUserId: null, // null = self
        currentUserId: 'me',
        profile: const PlayerProfile(
          userId: 'me',
          handle: 'me',
          displayName: null,
          visibility: 'private',
        ),
      ),
    );
    await tester.pumpAndSettle();

    // The compact visibility toggle sits in the header (right of the pseudo); a
    // private profile shows the "Private" state.
    expect(find.byKey(const Key('profile-visibility')), findsOneWidget);
    expect(find.text('Private'), findsOneWidget);
  });

  testWidgets(
    'own profile shows the usage-analytics consent toggle, grouped with '
    'visibility',
    (tester) async {
      final container = ProviderContainer(
        overrides: [
          currentUserIdProvider.overrideWithValue('me'),
          playerProfileProvider('me').overrideWith(
            (ref) async => const PlayerProfile(
              userId: 'me',
              handle: 'me',
              displayName: null,
              visibility: 'private',
            ),
          ),
          playActivityProvider('me').overrideWith((ref) async => _activity()),
          curatorRewardsServiceProvider.overrideWithValue(
            _FakeCuratorRewards(),
          ),
          achievementsServiceProvider.overrideWithValue(_FakeAchievements()),
          // Keep the own-profile's global standing (change: add-global-
          // leaderboard) off the real gRPC channel, like the shared harness does.
          globalLeaderboardServiceProvider.overrideWithValue(
            FakeGlobalLeaderboardService(),
          ),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(),
          ),
        ],
      );
      addTearDown(container.dispose);
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const ProfileScreen()),
        ),
      );
      await tester.pumpAndSettle();

      // Collection defaults to opt-out (on).
      expect(container.read(usageConsentProvider), isTrue);
      expect(find.byKey(const Key('profile-usage-consent')), findsOneWidget);

      await tester.tap(find.byKey(const Key('profile-usage-consent')));
      await tester.pumpAndSettle();

      expect(container.read(usageConsentProvider), isFalse);
    },
  );

  testWidgets('an unavailable (private/ineligible) profile is refused', (
    tester,
  ) async {
    await tester.pumpWidget(
      _scope([
        currentUserIdProvider.overrideWithValue('me'),
        // Server fail-closed: reading another player's private profile errors.
        playerProfileProvider(
          'other',
        ).overrideWith((ref) async => throw Exception('not found')),
      ], localizedApp(const ProfileScreen(userId: 'other'))),
    );
    await tester.pumpAndSettle();

    expect(find.text("This profile isn't available."), findsOneWidget);
    expect(find.byType(PlayHeatmap), findsNothing);
  });

  testWidgets('choosing Public opens the neutral age gate', (tester) async {
    await tester.pumpWidget(
      _harness(
        targetId: 'me',
        screenUserId: null,
        currentUserId: 'me',
        profile: const PlayerProfile(
          userId: 'me',
          handle: 'me',
          displayName: null,
          visibility: 'private',
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Tapping the header toggle (currently Private) starts going public.
    await tester.tap(find.byKey(const Key('profile-visibility')));
    await tester.pumpAndSettle();

    // The neutral age gate (asks a DOB, used once) appears.
    expect(find.text('Confirm your age'), findsOneWidget);
  });

  group('unresolved own identity (change: fix-session-account-retry)', () {
    /// Drives the REAL [SessionNotifier] into the degraded state (a transient
    /// `GetAccount` failure at launch) rather than stubbing `currentUserId`, so
    /// the test exercises the actual wiring the user hit.
    ProviderContainer degradedContainer(FakeAccountService acct) {
      final container = ProviderContainer(
        overrides: [
          tokenStoreProvider.overrideWithValue(
            FakeTokenStore(
              tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
            ),
          ),
          accountServiceProvider.overrideWithValue(acct),
          authServiceProvider.overrideWithValue(FakeAuthService()),
          oidcTokenSourceProvider.overrideWithValue(FakeOidcTokenSource()),
          playerProfileProvider('me').overrideWith(
            (ref) async => const PlayerProfile(
              userId: 'me',
              handle: 'me',
              displayName: null,
              visibility: 'private',
            ),
          ),
          playActivityProvider('me').overrideWith((ref) async => _activity()),
          curatorRewardsServiceProvider.overrideWithValue(
            _FakeCuratorRewards(),
          ),
          achievementsServiceProvider.overrideWithValue(_FakeAchievements()),
          globalLeaderboardServiceProvider.overrideWithValue(
            FakeGlobalLeaderboardService(),
          ),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(),
          ),
        ],
      );
      addTearDown(container.dispose);
      return container;
    }

    testWidgets('offers a retry instead of claiming the profile is missing', (
      tester,
    ) async {
      final acct = FakeAccountService(
        account: const Account(userId: 'me', version: 1, handle: 'me'),
      )..getErrors.add(const AuthException(AuthError.unavailable, 'offline'));
      final container = degradedContainer(acct);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const ProfileScreen()),
        ),
      );
      await tester.pumpAndSettle();

      expect(container.read(currentUserIdProvider), isNull);
      expect(
        find.byKey(const Key('profile-identity-retry')),
        findsOneWidget,
        reason: 'the session is live — this is recoverable, not a dead profile',
      );
      expect(
        find.text("This profile isn't available."),
        findsNothing,
        reason: 'that would be a false statement about the account',
      );

      // The scheduled backoff attempt is not what this test is about (the
      // notifier tests own it); stop it so no timer outlives the widget test.
      container.read(sessionNotifierProvider.notifier).onBackground();
    });

    testWidgets('tapping retry resolves the account and shows the profile', (
      tester,
    ) async {
      final acct = FakeAccountService(
        account: const Account(userId: 'me', version: 1, handle: 'me'),
      )..getErrors.add(const AuthException(AuthError.unavailable, 'offline'));
      final container = degradedContainer(acct);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const ProfileScreen()),
        ),
      );
      await tester.pumpAndSettle();
      container.read(sessionNotifierProvider.notifier).onBackground();

      await tester.tap(find.byKey(const Key('profile-identity-retry')));
      await tester.pumpAndSettle();

      expect(container.read(currentUserIdProvider), 'me');
      expect(find.byKey(const Key('profile-identity-retry')), findsNothing);
      expect(find.text('@me'), findsOneWidget);
    });

    testWidgets("another player's unavailable profile keeps its message", (
      tester,
    ) async {
      final container = ProviderContainer(
        overrides: [
          currentUserIdProvider.overrideWithValue('me'),
          // Fail-closed on the server: a private/ineligible target reads as an
          // error, which is a genuinely unavailable profile.
          playerProfileProvider('other').overrideWith(
            (ref) async => throw const AuthException(AuthError.notFound),
          ),
          playActivityProvider(
            'other',
          ).overrideWith((ref) async => _activity()),
          curatorRewardsServiceProvider.overrideWithValue(
            _FakeCuratorRewards(),
          ),
          achievementsServiceProvider.overrideWithValue(_FakeAchievements()),
          globalLeaderboardServiceProvider.overrideWithValue(
            FakeGlobalLeaderboardService(),
          ),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(),
          ),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const ProfileScreen(userId: 'other')),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text("This profile isn't available."), findsOneWidget);
      expect(find.byKey(const Key('profile-identity-retry')), findsNothing);
    });
  });
}
