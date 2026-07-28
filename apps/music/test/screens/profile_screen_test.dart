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
import 'package:music/services/profile_service.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/profile_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/play_heatmap.dart';

import '../support/localized.dart';

PlayActivity _activity() => PlayActivity(
  days: [DayActivity(day: DateTime(2024, 6, 13), count: 3, avgSyncPct: 84)],
  totalSessions: 5,
);

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

    expect(find.byKey(const Key('profile-visibility')), findsOneWidget);
    // A private profile invites going public.
    expect(
      find.text(
        'Make your profile public so other players can see your activity.',
      ),
      findsOneWidget,
    );
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

    await tester.tap(find.text('Public'));
    await tester.pumpAndSettle();

    // The neutral age gate (asks a DOB, used once) appears.
    expect(find.text('Confirm your age'), findsOneWidget);
  });
}
