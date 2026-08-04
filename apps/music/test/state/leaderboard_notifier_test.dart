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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/leaderboard_notifier.dart';
import 'package:music/state/play_sync_notifier.dart';

/// A leaderboard service that returns a fixed sequence of boards on successive
/// calls, so a test can detect a *re-fetch* (each call advances the sequence).
class _SeqLeaderboardService implements LeaderboardService {
  _SeqLeaderboardService(this._responses);

  final List<Leaderboard> _responses;
  int calls = 0;

  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async {
    final board = _responses[calls.clamp(0, _responses.length - 1)];
    calls++;
    return board;
  }

  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};
}

/// A play-sync notifier whose pending count is set directly, so a test can
/// simulate a delivery (the count dropping) without the outbox machinery.
class _FakePlaySync extends PlaySyncNotifier {
  _FakePlaySync(this._seed);

  final int _seed;

  @override
  int build() => _seed;

  void setCount(int value) => state = value;
}

LeaderboardEntry _own() => const LeaderboardEntry(
  rank: 1,
  userId: 'me',
  handle: '@me',
  displayName: null,
  subscore: 88,
  tiebreakMetric: 10,
  achievedAtMs: 1,
);

void main() {
  test(
    're-fetches the board when the play-sync outbox delivers (count drops)',
    () async {
      // First fetch: no own standing (the just-played session is not delivered
      // yet); after delivery the re-fetch surfaces the caller's own best.
      final service = _SeqLeaderboardService([
        Leaderboard.empty,
        Leaderboard(entries: const [], total: 0, own: _own()),
      ]);
      final container = ProviderContainer(
        overrides: [
          leaderboardServiceProvider.overrideWithValue(service),
          // One entry pending delivery.
          playSyncNotifierProvider.overrideWith(() => _FakePlaySync(1)),
        ],
      );
      addTearDown(container.dispose);

      // Keep the provider alive so its internal `ref.listen` stays active.
      container.listen(
        leaderboardProvider('p', LeaderboardMode.tempo),
        (_, _) {},
      );

      final first = await container.read(
        leaderboardProvider('p', LeaderboardMode.tempo).future,
      );
      expect(first.own, isNull);
      expect(service.calls, 1);

      // Deliver: the pending count drops 1 -> 0, which must refresh the board.
      (container.read(playSyncNotifierProvider.notifier) as _FakePlaySync)
          .setCount(0);
      await pumpEventQueue();

      final second = await container.read(
        leaderboardProvider('p', LeaderboardMode.tempo).future,
      );
      expect(second.own, isNotNull);
      expect(service.calls, 2);
    },
  );

  test(
    'does not re-fetch when the pending count only rises (capture)',
    () async {
      final service = _SeqLeaderboardService([Leaderboard.empty]);
      final container = ProviderContainer(
        overrides: [
          leaderboardServiceProvider.overrideWithValue(service),
          playSyncNotifierProvider.overrideWith(() => _FakePlaySync(0)),
        ],
      );
      addTearDown(container.dispose);
      container.listen(
        leaderboardProvider('p', LeaderboardMode.tempo),
        (_, _) {},
      );
      await container.read(
        leaderboardProvider('p', LeaderboardMode.tempo).future,
      );
      expect(service.calls, 1);

      // Capturing a session raises the count (0 -> 1): nothing delivered yet, so
      // the board must NOT re-fetch.
      (container.read(playSyncNotifierProvider.notifier) as _FakePlaySync)
          .setCount(1);
      await pumpEventQueue();

      await container.read(
        leaderboardProvider('p', LeaderboardMode.tempo).future,
      );
      expect(service.calls, 1);
    },
  );
}
