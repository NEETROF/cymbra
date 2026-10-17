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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/my_standings_notifier.dart';
import 'package:music/state/piece_bests.dart';
import 'package:music/state/piece_stars.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';
import 'piece_stars_test.mocks.dart';

@GenerateNiceMocks([MockSpec<LeaderboardService>()])
ProviderContainer _container({
  Map<String, String> prefs = const {},
  bool online = true,
  Map<String, LeaderboardStanding> standings = const {},
}) {
  final service = MockLeaderboardService();
  when(service.getMyStandings(any)).thenAnswer(
    (i) async => {
      for (final id in i.positionalArguments.first as List<String>)
        id: ?standings[id],
    },
  );
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      canUseOnlineServicesProvider.overrideWithValue(online),
      leaderboardServiceProvider.overrideWithValue(service),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

LeaderboardStanding _standing(String id, double? best) => LeaderboardStanding(
  scoreId: id,
  rank: 3,
  subscore: 80,
  mode: LeaderboardMode.tempo,
  bestOverallSyncPct: best,
);

Future<void> _tick() => Future<void>.delayed(const Duration(milliseconds: 20));

void main() {
  group('local piece bests', () {
    test('records a best that only rises, and persists it', () async {
      final prefs = FakePreferencesService();
      final c = _container(prefs: prefs.store);
      final n = c.read(localPieceBestsProvider.notifier);
      await n.record('p', 72);
      await n.record('p', 65);
      expect(n.bestOf('p'), 72);
      await n.record('p', 91);
      expect(c.read(localPieceBestsProvider)['p'], 91);
      expect(n.bestOf('q'), isNull);
    });

    test('restores the stored bests', () async {
      final c = _container(
        prefs: {
          'piece.bests.v1': jsonEncode({'p': 88.5}),
        },
      );
      c.read(localPieceBestsProvider);
      await _tick();
      expect(c.read(localPieceBestsProvider.notifier).bestOf('p'), 88.5);
    });
  });

  group('pieceStars', () {
    test('signed out → null: a guest sees no stars', () async {
      final c = _container(online: false);
      expect(
        c.read(pieceStarsProvider(pieceId: 'p', catalogId: 'cat')),
        isNull,
      );
    });

    test('signed in, nothing known → zero stars (empty, inviting)', () async {
      final c = _container();
      final sub = c.listen(
        pieceStarsProvider(pieceId: 'p', catalogId: 'cat'),
        (_, _) {},
      );
      addTearDown(sub.close);
      await _tick();
      expect(c.read(pieceStarsProvider(pieceId: 'p', catalogId: 'cat')), 0);
    });

    test('the server\'s best overall gives the stars', () async {
      final c = _container(standings: {'cat': _standing('cat', 85)});
      final sub = c.listen(
        pieceStarsProvider(pieceId: 'p', catalogId: 'cat'),
        (_, _) {},
      );
      addTearDown(sub.close);
      await _tick();
      expect(c.read(pieceStarsProvider(pieceId: 'p', catalogId: 'cat')), 2);
    });

    test('the device\'s own record counts too — the greater wins', () async {
      final c = _container(
        prefs: {
          'piece.bests.v1': jsonEncode({'p': 96}),
        },
        standings: {'cat': _standing('cat', 85)},
      );
      final sub = c.listen(
        pieceStarsProvider(pieceId: 'p', catalogId: 'cat'),
        (_, _) {},
      );
      addTearDown(sub.close);
      await _tick();
      expect(c.read(pieceStarsProvider(pieceId: 'p', catalogId: 'cat')), 3);
    });

    test(
      'a bundled piece (no catalogue id) has only the local record',
      () async {
        final c = _container(
          prefs: {
            'piece.bests.v1': jsonEncode({'ode': 61}),
          },
        );
        final sub = c.listen(pieceStarsProvider(pieceId: 'ode'), (_, _) {});
        addTearDown(sub.close);
        await _tick();
        expect(c.read(pieceStarsProvider(pieceId: 'ode')), 1);
      },
    );

    test('the last standing seen survives the delivery reset', () async {
      final c = _container(standings: {'cat': _standing('cat', 85)});
      final sub = c.listen(
        pieceStarsProvider(pieceId: 'p', catalogId: 'cat'),
        (_, _) {},
      );
      addTearDown(sub.close);
      await _tick();
      expect(
        c.read(myStandingsProvider.notifier).known('cat')?.bestOverallSyncPct,
        85,
      );
      expect(c.read(myStandingsProvider.notifier).known('other'), isNull);
    });
  });
}
