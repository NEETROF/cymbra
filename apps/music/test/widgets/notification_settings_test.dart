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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/screens/profile_screen.dart';
import 'package:music/services/achievements_service.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/global_leaderboard_service.dart';
import 'package:music/services/notification_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/profile_notifier.dart';
import 'package:music/state/push_categories.dart';
import 'package:music/state/session_notifier.dart';

import '../support/global_leaderboard_fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import 'notification_settings_test.mocks.dart';

@GenerateNiceMocks([MockSpec<NotificationRegistryService>()])
/// Stand-in for the category a feature will declare — the platform ships none.
final _streak = PushCategory(
  id: 'practice_streak',
  label: (_) => 'Streak reminder',
);

/// The per-category notification switches live on the OWN profile, next to the
/// other profile-level preferences (change: add-push-notifications; they follow
/// the usage-analytics toggle that moved off the account menu).

const _me = 'me';

PlayActivity _activity() => PlayActivity(
  days: [DayActivity(day: DateTime(2024, 6, 13), count: 3, avgSyncPct: 84)],
  totalSessions: 5,
);

/// A no-op curator-rewards seam so the own-profile's rewards section resolves.
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
/// badges); an empty registry keeps this test off the real gRPC channel.
class _FakeAchievements implements AchievementsService {
  @override
  Future<List<AchievementBadgeView>> getAchievements() async => const [];
}

void main() {
  Future<MockNotificationRegistryService> pumpProfile(
    WidgetTester tester, {
    required List<PushCategory> categories,
    Map<String, bool> stored = const {},
    String? currentUserId = _me,
    String? screenUserId,
  }) async {
    final registry = MockNotificationRegistryService();
    when(
      registry.settings(),
    ).thenAnswer((_) async => NotificationSettings(prefs: stored));
    final container = ProviderContainer(
      overrides: [
        currentUserIdProvider.overrideWithValue(currentUserId),
        playerProfileProvider(_me).overrideWith(
          (ref) async => const PlayerProfile(
            userId: _me,
            handle: 'bob',
            displayName: 'Bob',
            visibility: 'private',
          ),
        ),
        playActivityProvider(_me).overrideWith((ref) async => _activity()),
        curatorRewardsServiceProvider.overrideWithValue(_FakeCuratorRewards()),
        achievementsServiceProvider.overrideWithValue(_FakeAchievements()),
        globalLeaderboardServiceProvider.overrideWithValue(
          FakeGlobalLeaderboardService(),
        ),
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        notificationRegistryServiceProvider.overrideWithValue(registry),
        pushCategoriesProvider.overrideWithValue(categories),
      ],
    );
    addTearDown(container.dispose);
    await tester.binding.setSurfaceSize(const Size(900, 1400));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(ProfileScreen(userId: screenUserId)),
      ),
    );
    await tester.pumpAndSettle();
    return registry;
  }

  testWidgets('no declared category renders no notification switch', (
    tester,
  ) async {
    await pumpProfile(tester, categories: const []);
    expect(
      find.byKey(const Key('profile-notification-practice_streak')),
      findsNothing,
    );
    // The other profile preferences are still there — the section is simply empty.
    expect(find.byKey(const Key('profile-usage-consent')), findsOneWidget);
  });

  testWidgets('a declared category renders a switch on by its default', (
    tester,
  ) async {
    await pumpProfile(tester, categories: [_streak]);
    final tile = tester.widget<SwitchListTile>(
      find.byKey(const Key('profile-notification-practice_streak')),
    );
    expect(tile.value, isTrue); // defaultEnabled
    expect(find.text('Streak reminder'), findsOneWidget);
  });

  testWidgets('the stored preference wins over the default', (tester) async {
    await pumpProfile(
      tester,
      categories: [_streak],
      stored: const {'practice_streak': false},
    );
    final tile = tester.widget<SwitchListTile>(
      find.byKey(const Key('profile-notification-practice_streak')),
    );
    expect(tile.value, isFalse);
  });

  testWidgets('flipping the switch records the opposite choice', (
    tester,
  ) async {
    final registry = await pumpProfile(tester, categories: [_streak]);
    await tester.tap(
      find.byKey(const Key('profile-notification-practice_streak')),
    );
    await tester.pumpAndSettle();

    verify(
      registry.setPref(category: 'practice_streak', enabled: false),
    ).called(1);
    final tile = tester.widget<SwitchListTile>(
      find.byKey(const Key('profile-notification-practice_streak')),
    );
    expect(tile.value, isFalse);
  });

  testWidgets("another player's profile shows no notification switch", (
    tester,
  ) async {
    // The switches are self-only, like every other profile preference: viewing
    // someone else's profile exposes none of the viewer's settings.
    await pumpProfile(
      tester,
      categories: [_streak],
      currentUserId: 'someone-else',
      screenUserId: _me,
    );
    expect(
      find.byKey(const Key('profile-notification-practice_streak')),
      findsNothing,
    );
  });
}
