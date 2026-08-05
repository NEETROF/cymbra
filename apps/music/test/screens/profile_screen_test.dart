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
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/profile_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/play_heatmap.dart';

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
}
