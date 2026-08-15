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
import 'package:music/services/leaderboard_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/widgets/score_card.dart';

import '../support/localized.dart';

/// The card batches a standings lookup for a catalog entry; an empty map is
/// enough here (this test is about the attribution line, not the badge).
class _NoBoards implements LeaderboardService {
  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};

  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async => Leaderboard.empty;
}

CatalogEntry _catalogEntry({String? credit, String source = 'user-proposal'}) =>
    CatalogEntry(
      id: 'catalog-x',
      title: 'Canon in D',
      composer: 'Pachelbel',
      level: PracticeLevel.beginner,
      catalogId: 'cid',
      source: source,
      contributorCredit: credit,
    );

Future<void> _pump(WidgetTester tester, CatalogEntry entry) async {
  final container = ProviderContainer(
    overrides: [leaderboardServiceProvider.overrideWithValue(_NoBoards())],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        Scaffold(
          body: Center(
            child: SizedBox(
              width: 220,
              height: 320,
              child: ScoreCard(entry: entry, onTap: () {}),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('an accepted user-proposed score credits its proposer', (
    tester,
  ) async {
    await _pump(tester, _catalogEntry(credit: 'alice'));
    expect(find.text('Proposed by @alice'), findsOneWidget);
  });

  testWidgets(
    'no credit (private proposer profile) shows no attribution at all',
    (tester) async {
      await _pump(tester, _catalogEntry());
      expect(find.textContaining('Proposed by'), findsNothing);
      // The `user-proposal` origin sentinel is never surfaced as a dataset source.
      expect(find.textContaining('user-proposal'), findsNothing);
      expect(find.textContaining('via'), findsNothing);
    },
  );

  testWidgets('a crawler-ingested score keeps its dataset origin', (
    tester,
  ) async {
    await _pump(tester, _catalogEntry(source: 'pdmx'));
    expect(find.text('via PDMX'), findsOneWidget);
    expect(find.textContaining('Proposed by'), findsNothing);
  });
}
