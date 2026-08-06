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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/global_leaderboard_service.dart';
import 'package:music/state/global_leaderboard.dart';
import 'package:music/state/global_leaderboard_notifier.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/play_sync_notifier.dart';

import '../support/global_leaderboard_fakes.dart';

/// A play-sync notifier whose pending count is set directly, so a test can
/// simulate a delivery (the count dropping) without the outbox machinery.
class _FakePlaySync extends PlaySyncNotifier {
  _FakePlaySync(this._seed);

  final int _seed;

  @override
  int build() => _seed;

  void setCount(int value) => state = value;
}

ProviderContainer _container(
  FakeGlobalLeaderboardService service, {
  int pending = 0,
}) {
  final container = ProviderContainer(
    overrides: [
      globalLeaderboardServiceProvider.overrideWithValue(service),
      playSyncNotifierProvider.overrideWith(() => _FakePlaySync(pending)),
    ],
  );
  addTearDown(container.dispose);
  return container;
}

void main() {
  test('the empty season key reads the CURRENT season', () async {
    final service = FakeGlobalLeaderboardService();
    final container = _container(service);
    await container.read(
      globalLeaderboardProvider(LeaderboardMode.tempo, '').future,
    );
    // An empty family key maps to `null` on the wire = the current season.
    expect(service.requests, [(LeaderboardMode.tempo, '')]);
  });

  test('a season key is passed through to the service', () async {
    final service = FakeGlobalLeaderboardService();
    final container = _container(service);
    await container.read(
      globalLeaderboardProvider(LeaderboardMode.reaction, '2026-01-01').future,
    );
    expect(service.requests, [(LeaderboardMode.reaction, '2026-01-01')]);
  });

  test('re-fetches when the play-sync outbox delivers (count drops)', () async {
    final service = FakeGlobalLeaderboardService();
    final container = _container(service, pending: 1);
    // Keep the provider alive so its internal `ref.listen` stays active.
    container.listen(
      globalLeaderboardProvider(LeaderboardMode.tempo, ''),
      (_, _) {},
    );
    await container.read(
      globalLeaderboardProvider(LeaderboardMode.tempo, '').future,
    );
    expect(service.requests.length, 1);

    // Deliver: the pending count drops 1 -> 0, which must refresh the board.
    (container.read(playSyncNotifierProvider.notifier) as _FakePlaySync)
        .setCount(0);
    await pumpEventQueue();
    await container.read(
      globalLeaderboardProvider(LeaderboardMode.tempo, '').future,
    );
    expect(service.requests.length, 2);
  });

  test(
    'does not re-fetch when the pending count only rises (capture)',
    () async {
      final service = FakeGlobalLeaderboardService();
      final container = _container(service);
      container.listen(
        globalLeaderboardProvider(LeaderboardMode.tempo, ''),
        (_, _) {},
      );
      await container.read(
        globalLeaderboardProvider(LeaderboardMode.tempo, '').future,
      );
      expect(service.requests.length, 1);

      (container.read(playSyncNotifierProvider.notifier) as _FakePlaySync)
          .setCount(1);
      await pumpEventQueue();
      await container.read(
        globalLeaderboardProvider(LeaderboardMode.tempo, '').future,
      );
      expect(service.requests.length, 1);
    },
  );

  test('the seasons provider surfaces the current + past seasons', () async {
    final service = FakeGlobalLeaderboardService(
      seasons: const GlobalSeasons(
        currentSeasonId: '2026-01-31',
        pastSeasonIds: ['2026-01-01'],
      ),
    );
    final container = _container(service);
    final seasons = await container.read(globalSeasonsProvider.future);
    expect(seasons.currentSeasonId, '2026-01-31');
    expect(seasons.pastSeasonIds, ['2026-01-01']);
    // `all` is the selector's list, current first.
    expect(seasons.all, ['2026-01-31', '2026-01-01']);
  });
}
