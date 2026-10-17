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

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/score_card.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';

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

CatalogEntry _entry({String? catalogId, String id = 'catalog-x'}) =>
    CatalogEntry(
      id: id,
      title: 'Canon in D',
      composer: 'Pachelbel',
      level: PracticeLevel.beginner,
      catalogId: catalogId,
    );

Future<void> _pump(
  WidgetTester tester, {
  required CatalogEntry entry,
  Map<String, LeaderboardStanding> standings = const {},
  Map<String, String> prefs = const {},
  bool online = true,
}) async {
  final c = ProviderContainer(
    overrides: [
      leaderboardServiceProvider.overrideWithValue(
        _FakeLeaderboardService(standings),
      ),
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      canUseOnlineServicesProvider.overrideWithValue(online),
    ],
  );
  addTearDown(c.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        Scaffold(
          body: Center(
            child: SizedBox(
              width: 220,
              height: 340,
              child: ScoreCard(entry: entry, onTap: () {}),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

int _filled(WidgetTester tester) => tester
    .widgetList<Icon>(find.byType(Icon))
    .where((i) => i.icon == Icons.star_rounded)
    .length;

void main() {
  group('score card stars (change: add-music-piece-stars)', () {
    testWidgets('signed in, the server\'s best fills the stars', (
      tester,
    ) async {
      await _pump(
        tester,
        entry: _entry(catalogId: 'cid'),
        standings: {
          'cid': const LeaderboardStanding(
            scoreId: 'cid',
            rank: 3,
            subscore: 80,
            mode: LeaderboardMode.tempo,
            bestOverallSyncPct: 85,
          ),
        },
      );
      expect(find.byKey(const Key('piece-stars-catalog-x')), findsOneWidget);
      expect(_filled(tester), 2);
    });

    testWidgets('a piece never scored shows three empty stars', (tester) async {
      await _pump(tester, entry: _entry(catalogId: 'cid'));
      expect(find.byKey(const Key('piece-stars-catalog-x')), findsOneWidget);
      expect(_filled(tester), 0);
      expect(find.byIcon(Icons.star_outline_rounded), findsNWidgets(3));
    });

    testWidgets('a bundled piece reads the device\'s record', (tester) async {
      await _pump(
        tester,
        entry: _entry(id: 'ode-to-joy'),
        prefs: {
          'piece.bests.v1': jsonEncode({'ode-to-joy': 97}),
        },
      );
      expect(_filled(tester), 3);
    });

    testWidgets('signed out, no star row at all', (tester) async {
      await _pump(tester, entry: _entry(catalogId: 'cid'), online: false);
      expect(find.byKey(const Key('piece-stars-catalog-x')), findsNothing);
      expect(find.byIcon(Icons.star_outline_rounded), findsNothing);
    });
  });
}
