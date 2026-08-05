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

/// A leaderboard service whose only job here is the batch standings map.
class _FakeLeaderboardService implements LeaderboardService {
  _FakeLeaderboardService(this.standings);

  final Map<String, LeaderboardStanding> standings;

  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => standings;

  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async => Leaderboard.empty;
}

CatalogEntry _entry({String? catalogId}) => CatalogEntry(
  id: 'catalog-x',
  title: 'Canon in D',
  composer: 'Pachelbel',
  level: PracticeLevel.beginner,
  catalogId: catalogId,
);

Future<void> _pump(
  WidgetTester tester, {
  required CatalogEntry entry,
  required LeaderboardService service,
}) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: ProviderContainer(
        overrides: [leaderboardServiceProvider.overrideWithValue(service)],
      ),
      child: localizedApp(
        // Constrain like a grid cell — the card is a tile, not a full page.
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
  await tester.pumpAndSettle(); // let the coalesced batch flush + rebuild
}

void main() {
  testWidgets('a ranked catalog card shows the trophy + best rank', (
    tester,
  ) async {
    final service = _FakeLeaderboardService({
      'cid': const LeaderboardStanding(
        scoreId: 'cid',
        rank: 3,
        subscore: 80,
        mode: LeaderboardMode.tempo,
      ),
    });
    await _pump(
      tester,
      entry: _entry(catalogId: 'cid'),
      service: service,
    );
    expect(find.byIcon(Icons.emoji_events), findsOneWidget);
    expect(find.text('#3'), findsOneWidget);
  });

  testWidgets('a populated board the player is not on shows a bare trophy', (
    tester,
  ) async {
    // rank 0 = the caller is not ranked, but the board has entries.
    final service = _FakeLeaderboardService({
      'cid': const LeaderboardStanding(
        scoreId: 'cid',
        rank: 0,
        subscore: 0,
        mode: LeaderboardMode.tempo,
      ),
    });
    await _pump(
      tester,
      entry: _entry(catalogId: 'cid'),
      service: service,
    );
    expect(find.byIcon(Icons.emoji_events), findsOneWidget);
    expect(find.textContaining('#'), findsNothing);
  });

  testWidgets('an empty board (no standing) shows no badge', (tester) async {
    await _pump(
      tester,
      entry: _entry(catalogId: 'cid'),
      service: _FakeLeaderboardService(const {}),
    );
    expect(find.byIcon(Icons.emoji_events), findsNothing);
  });

  testWidgets('a non-catalog card has no leaderboard badge', (tester) async {
    await _pump(
      tester,
      entry: _entry(catalogId: null),
      service: _FakeLeaderboardService(const {}),
    );
    expect(find.byIcon(Icons.emoji_events), findsNothing);
  });
}
