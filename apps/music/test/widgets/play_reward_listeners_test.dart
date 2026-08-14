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
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/state/play_reward_cue.dart';
import 'package:music/widgets/play_reward_listeners.dart';

import '../support/localized.dart';
import 'play_reward_listeners_test.mocks.dart';

@GenerateNiceMocks([MockSpec<CuratorRewardsService>()])
CuratorRewardsView _standing(int level) => CuratorRewardsView(
  lifetimePoints: level * 100,
  spendableBalance: level * 100,
  level: level,
  levelFloor: level * 100,
  nextLevelAt: (level + 1) * 100,
  totalRatings: 0,
  coverageContribution: 0,
  alignmentRate: 0,
  badges: const [],
  recent: const [],
);

/// Mounts the listener over a root container (the repo convention for keepAlive
/// overrides) whose standing seam returns [levels] in order, one per read.
Future<ProviderContainer> _mount(
  WidgetTester tester,
  List<int> levels, {
  bool failFirst = false,
}) async {
  final service = MockCuratorRewardsService();
  var call = 0;
  when(service.getRewards()).thenAnswer((_) async {
    if (failFirst && call++ == 0) throw StateError('offline');
    final i = call.clamp(0, levels.length - 1);
    if (!failFirst) call++;
    return _standing(levels[i]);
  });
  final c = ProviderContainer(
    overrides: [curatorRewardsServiceProvider.overrideWithValue(service)],
  );
  addTearDown(c.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        const Scaffold(body: PlayRewardListeners(child: Text('player'))),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return c;
}

void main() {
  group('play reward listeners', () {
    testWidgets('a level crossed by playing is celebrated', (tester) async {
      // Level 1 loaded while playing; the award pushes the standing to 2.
      final c = await _mount(tester, [1, 2]);
      expect(find.byKey(const Key('reward-celebration')), findsNothing);

      c.read(playRewardCueProvider.notifier)
        ..arm('s1')
        ..report('s1', 12); // the standing refreshes itself off this
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('reward-celebration')), findsOneWidget);
      expect(find.text('Level up!'), findsOneWidget);
      expect(find.textContaining('level 2'), findsOneWidget);
    });

    testWidgets('an award that crosses no level celebrates nothing', (
      tester,
    ) async {
      final c = await _mount(tester, [1, 1]);
      c.read(playRewardCueProvider.notifier)
        ..arm('s1')
        ..report('s1', 3);
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('reward-celebration')), findsNothing);
    });

    testWidgets('a first load is never mistaken for a level-up', (
      tester,
    ) async {
      // Nothing loaded before (a guest, or an offline read that failed): there is
      // no "before" to have crossed, so arriving at level 5 celebrates nothing.
      await _mount(tester, [5, 5], failFirst: true);
      expect(find.byKey(const Key('reward-celebration')), findsNothing);
    });
  });
}
