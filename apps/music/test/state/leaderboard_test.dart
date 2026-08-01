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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/leaderboard.dart';

LeaderboardEntry _entry(
  String userId, {
  int rank = 1,
  String? handle,
  String? displayName,
  double subscore = 90,
}) => LeaderboardEntry(
  rank: rank,
  userId: userId,
  handle: handle,
  displayName: displayName,
  subscore: subscore,
  tiebreakMetric: 10,
  achievedAtMs: 1,
);

void main() {
  group('LeaderboardMode', () {
    test('maps to the wire value the RPC expects', () {
      expect(LeaderboardMode.tempo.wire, 'tempo');
      expect(LeaderboardMode.reaction.wire, 'reaction');
    });
  });

  group('LeaderboardEntry.label', () {
    test('prefers the handle, falls back to the display name, else null', () {
      expect(_entry('u', handle: '@ana', displayName: 'Ana').label, '@ana');
      expect(_entry('u', displayName: 'Ana').label, 'Ana');
      expect(_entry('u').label, isNull);
    });
  });

  group('Leaderboard.isViewer', () {
    test('true only for the entry matching the viewer own row', () {
      final own = _entry('me', rank: 2);
      final board = Leaderboard(
        entries: [_entry('a'), _entry('me', rank: 2)],
        total: 2,
        own: own,
      );
      expect(board.isViewer(_entry('me', rank: 2)), isTrue);
      expect(board.isViewer(_entry('a')), isFalse);
    });

    test('false for every entry when there is no own standing', () {
      const board = Leaderboard.empty;
      expect(board.isViewer(_entry('a')), isFalse);
    });
  });
}
