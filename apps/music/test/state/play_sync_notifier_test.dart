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

import 'dart:async';
import 'dart:math';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/play_sync_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/play_outbox_store.dart';
import 'package:music/state/play_reward_cue.dart';
import 'package:music/state/play_session_envelope.dart';
import 'package:music/state/play_sync_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_summary.dart';

import '../support/prefs_fakes.dart';

@GenerateNiceMocks([MockSpec<PlaySyncService>()])
import 'play_sync_notifier_test.mocks.dart';

/// A connectivity seam with a manually-pumped "online" event.
class FakeConnectivityService implements ConnectivityService {
  @override
  Future<bool> isOnline() async => true;
  @override
  Future<bool> isDefinitelyOffline() async => !(true);
  final _controller = StreamController<void>.broadcast();
  @override
  Stream<void> get onOnline => _controller.stream;
  @override
  Stream<bool> get onlineStatus => const Stream.empty();
  void goOnline() => _controller.add(null);
  void dispose() => _controller.close();
}

/// No-op retry scheduler so failed drains don't fire real timers mid-test.
class NoopRetryScheduler implements PlayRetryScheduler {
  @override
  void schedule(Duration delay, void Function() action) {}
  @override
  void cancel() {}
}

SessionResult _result({int playedAtMs = 1718494200000, double sync = 80}) =>
    SessionResult(
      pieceId: 'piece-1',
      title: 'Etude',
      hands: 'both',
      overallSyncPct: sync,
      runMode: RunMode.free,
      freeSyncPct: sync,
      waitSyncPct: null,
      freeOnsetCount: 10,
      waitOnsetCount: 0,
      avgFreeOffsetMs: 0,
      avgReactionMs: null,
      timing: 0.8,
      correctness: 0.9,
      sustain: 0.85,
      verdictCounts: const {},
      wrongNotes: 0,
      bestCombo: 8,
      notes: const [],
      playedAtMs: playedAtMs,
      speed: 1,
    );

PlaySessionEnvelope _entry(String id, {required String userId}) =>
    PlaySessionEnvelope(
      sessionId: id,
      userId: userId,
      scoreId: 'p',
      playedAtMs: 1718494200000,
      tzOffsetMinutes: 0,
      overallSyncPct: 75,
      sessionResultJson: '{}',
    );

/// A queued **practice** entry: scoreless, delivered on the practice ingest.
PlaySessionEnvelope _practiceEntry(String id, {required String userId}) =>
    PlaySessionEnvelope(
      sessionId: id,
      userId: userId,
      scoreId: 'p',
      playedAtMs: 1718494200000,
      tzOffsetMinutes: 0,
      overallSyncPct: 0,
      sessionResultJson: '',
      isPractice: true,
    );

ProviderContainer _container({
  required PlaySyncService service,
  required FakePreferencesService prefs,
  FakeConnectivityService? connectivity,
  String? userId = 'u1',
  bool online = true,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      playSyncServiceProvider.overrideWithValue(service),
      connectivityServiceProvider.overrideWithValue(
        connectivity ?? FakeConnectivityService(),
      ),
      playRetrySchedulerProvider.overrideWithValue(NoopRetryScheduler()),
      currentUserIdProvider.overrideWithValue(userId),
      canUseOnlineServicesProvider.overrideWithValue(online),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('captures a session and removes it on the server ack', () async {
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    final prefs = FakePreferencesService();
    final c = _container(service: service, prefs: prefs);

    await c.read(playSyncNotifierProvider.notifier).captureSession(_result());

    verify(service.recordSession(any)).called(1);
    expect(await c.read(playOutboxStoreProvider).all(), isEmpty);
    expect(c.read(playSyncNotifierProvider), 0);
  });

  test(
    'a captured session is safe offline and delivered on a later drain',
    () async {
      final service = MockPlaySyncService();
      when(service.recordSession(any)).thenThrow(Exception('offline'));
      final prefs = FakePreferencesService();
      final c = _container(service: service, prefs: prefs);
      final notifier = c.read(playSyncNotifierProvider.notifier);

      // Offline: capture persists, delivery fails, the entry is KEPT (no loss).
      await notifier.captureSession(_result());
      expect(await c.read(playOutboxStoreProvider).all(), hasLength(1));

      // Connectivity/server recovers: the SAME entry is delivered and removed.
      reset(service);
      when(service.recordSession(any)).thenAnswer((_) async => 0);
      await notifier.drain();
      expect(await c.read(playOutboxStoreProvider).all(), isEmpty);
      verify(service.recordSession(any)).called(1);
    },
  );

  // --- reward cue (change: add-play-rewards) ------------------------------

  test("the ack's award reaches the reward cue", () async {
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 12);
    final c = _container(service: service, prefs: FakePreferencesService());

    await c.read(playSyncNotifierProvider.notifier).captureSession(_result());

    expect(c.read(playRewardCueProvider).points, 12);
    expect(c.read(playRewardCueProvider).seq, 1);
  });

  test('a session that earned nothing publishes no cue', () async {
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    final c = _container(service: service, prefs: FakePreferencesService());

    await c.read(playSyncNotifierProvider.notifier).captureSession(_result());

    expect(c.read(playRewardCueProvider).points, 0);
    expect(c.read(playRewardCueProvider).seq, 0);
  });

  test('a new run clears the previous run\'s award before delivery', () async {
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 7);
    final c = _container(service: service, prefs: FakePreferencesService());
    final notifier = c.read(playSyncNotifierProvider.notifier);

    await notifier.captureSession(_result());
    expect(c.read(playRewardCueProvider).points, 7);

    // The next run earns nothing: its summary must show NO cue rather than the
    // previous run's number.
    reset(service);
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    await notifier.captureSession(_result());
    expect(c.read(playRewardCueProvider).points, 0);
  });

  test('a backlogged entry never claims the current run\'s cue', () async {
    // A spell offline left an older session queued. When connectivity returns
    // mid-run, its award must not be shown as what the run on screen earned.
    final prefs = FakePreferencesService();
    await PlayOutboxStore(prefs).add(_entry('old', userId: 'u1'));
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((invocation) async {
      final e = invocation.positionalArguments.first as PlaySessionEnvelope;
      return e.sessionId == 'old' ? 40 : 6;
    });
    final c = _container(service: service, prefs: prefs);

    await c.read(playSyncNotifierProvider.notifier).captureSession(_result());

    // Both were delivered; only the armed (current) run set the cue.
    expect(await c.read(playOutboxStoreProvider).all(), isEmpty);
    expect(c.read(playRewardCueProvider).points, 6);
  });

  test('an acked entry is never sent again (no double-count)', () async {
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    final c = _container(service: service, prefs: FakePreferencesService());
    final notifier = c.read(playSyncNotifierProvider.notifier);

    await notifier.captureSession(_result());
    await notifier.drain(); // a second drain finds nothing to send
    verify(service.recordSession(any)).called(1);
  });

  test('delivers only the signed-in user\'s entries (per-user)', () async {
    final prefs = FakePreferencesService();
    // Two accounts' sessions queued on a shared device.
    await PlayOutboxStore(prefs).add(_entry('a', userId: 'u1'));
    await PlayOutboxStore(prefs).add(_entry('b', userId: 'u2'));

    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    final c = _container(service: service, prefs: prefs, userId: 'u1');

    await c.read(playSyncNotifierProvider.notifier).drain();

    final sent = verify(
      service.recordSession(captureAny),
    ).captured.cast<PlaySessionEnvelope>();
    expect(sent.map((e) => e.sessionId), ['a']); // only u1's
    // u2's entry stays until u2 signs in.
    final remaining = await PlayOutboxStore(prefs).all();
    expect(remaining.map((e) => e.sessionId), ['b']);
  });

  test('pending entries flush on the next launch', () async {
    final prefs = FakePreferencesService();
    await PlayOutboxStore(prefs).add(_entry('a', userId: 'u1'));

    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    final c = _container(service: service, prefs: prefs);

    // Reading the notifier runs build(), whose launch drain flushes the backlog.
    c.read(playSyncNotifierProvider);
    await pumpEventQueue();

    verify(service.recordSession(any)).called(1);
    expect(await PlayOutboxStore(prefs).all(), isEmpty);
  });

  test(
    'does not deliver while signed out, then flushes once signed in',
    () async {
      final prefs = FakePreferencesService();
      await PlayOutboxStore(prefs).add(_entry('a', userId: 'u1'));
      final service = MockPlaySyncService();
      when(service.recordSession(any)).thenAnswer((_) async => 0);

      // Signed out (no user / offline): the entry is retained, nothing is sent.
      final offline = _container(
        service: service,
        prefs: prefs,
        userId: null,
        online: false,
      );
      await offline.read(playSyncNotifierProvider.notifier).drain();
      verifyNever(service.recordSession(any));
      expect(await PlayOutboxStore(prefs).all(), hasLength(1));

      // Signed in on a fresh launch: the backlog flushes.
      final online = _container(service: service, prefs: prefs);
      online.read(playSyncNotifierProvider);
      await pumpEventQueue();
      verify(service.recordSession(any)).called(1);
    },
  );

  test('resumes delivery when connectivity returns', () async {
    final prefs = FakePreferencesService();
    final service = MockPlaySyncService();
    when(service.recordSession(any)).thenThrow(Exception('offline'));
    final conn = FakeConnectivityService();
    addTearDown(conn.dispose);
    final c = _container(service: service, prefs: prefs, connectivity: conn);
    final notifier = c.read(playSyncNotifierProvider.notifier);

    await notifier.captureSession(_result()); // fails, kept
    expect(await c.read(playOutboxStoreProvider).all(), hasLength(1));

    // Connectivity returns → the sender re-drains and delivers.
    reset(service);
    when(service.recordSession(any)).thenAnswer((_) async => 0);
    conn.goOnline();
    await pumpEventQueue();
    expect(await c.read(playOutboxStoreProvider).all(), isEmpty);
  });

  test(
    'playActivity provider reads the aggregate through the service',
    () async {
      final service = MockPlaySyncService();
      when(
        service.getPlayActivity('u1'),
      ).thenAnswer((_) async => const PlayActivity(days: [], totalSessions: 3));
      final c = ProviderContainer(
        overrides: [playSyncServiceProvider.overrideWithValue(service)],
      );
      addTearDown(c.dispose);
      final a = await c.read(playActivityProvider('u1').future);
      expect(a.totalSessions, 3);
    },
  );

  group('practice (change: add-measure-range-practice)', () {
    test(
      'capture enqueues a SCORELESS record on the practice ingest',
      () async {
        final service = MockPlaySyncService();
        when(service.recordPractice(any)).thenAnswer((_) async => 0);
        final c = _container(service: service, prefs: FakePreferencesService());

        await c
            .read(playSyncNotifierProvider.notifier)
            .capturePractice(scoreId: 'piece-1');

        final sent = verify(
          service.recordPractice(captureAny),
        ).captured.cast<PlaySessionEnvelope>();
        expect(sent, hasLength(1));
        expect(sent.single.isPractice, isTrue);
        expect(sent.single.scoreId, 'piece-1');
        // No grade whatsoever.
        expect(sent.single.overallSyncPct, 0);
        expect(sent.single.sessionResultJson, isEmpty);
        // ...and it never touches the scored ingest.
        verifyNever(service.recordSession(any));
        expect(await c.read(playOutboxStoreProvider).all(), isEmpty);
      },
    );

    test('a practice survives a restart and is retried until acked', () async {
      final prefs = FakePreferencesService();
      final service = MockPlaySyncService();
      when(service.recordPractice(any)).thenThrow(Exception('offline'));
      final c = _container(service: service, prefs: prefs);

      await c.read(playSyncNotifierProvider.notifier).capturePractice();
      // Delivery failed → the durable entry is KEPT (no loss).
      final pending = await PlayOutboxStore(prefs).all();
      expect(pending, hasLength(1));
      expect(pending.single.isPractice, isTrue);

      // A fresh launch on the same storage flushes the backlog.
      reset(service);
      when(service.recordPractice(any)).thenAnswer((_) async => 0);
      final relaunched = _container(service: service, prefs: prefs);
      relaunched.read(playSyncNotifierProvider);
      await pumpEventQueue();
      verify(service.recordPractice(any)).called(1);
      expect(await PlayOutboxStore(prefs).all(), isEmpty);
    });

    test('practice and scored entries are routed separately', () async {
      final prefs = FakePreferencesService();
      await PlayOutboxStore(prefs).add(_entry('scored', userId: 'u1'));
      await PlayOutboxStore(
        prefs,
      ).add(_practiceEntry('practice', userId: 'u1'));

      final service = MockPlaySyncService();
      when(service.recordSession(any)).thenAnswer((_) async => 0);
      when(service.recordPractice(any)).thenAnswer((_) async => 0);
      final c = _container(service: service, prefs: prefs);

      await c.read(playSyncNotifierProvider.notifier).drain();

      expect(
        verify(
          service.recordSession(captureAny),
        ).captured.cast<PlaySessionEnvelope>().map((e) => e.sessionId),
        ['scored'],
      );
      expect(
        verify(
          service.recordPractice(captureAny),
        ).captured.cast<PlaySessionEnvelope>().map((e) => e.sessionId),
        ['practice'],
      );
      expect(await PlayOutboxStore(prefs).all(), isEmpty);
    });

    test('an acked practice is never sent again (no double-count)', () async {
      final service = MockPlaySyncService();
      when(service.recordPractice(any)).thenAnswer((_) async => 0);
      final c = _container(service: service, prefs: FakePreferencesService());
      final notifier = c.read(playSyncNotifierProvider.notifier);

      await notifier.capturePractice();
      await notifier.drain();
      verify(service.recordPractice(any)).called(1);
    });

    test('a guest practice is not captured', () async {
      final service = MockPlaySyncService();
      final prefs = FakePreferencesService();
      final c = _container(service: service, prefs: prefs, userId: null);

      await c.read(playSyncNotifierProvider.notifier).capturePractice();

      verifyNever(service.recordPractice(any));
      expect(await PlayOutboxStore(prefs).all(), isEmpty);
    });

    test('the envelope round-trips its practice flag through storage', () {
      final e = _practiceEntry('p', userId: 'u1');
      final back = PlaySessionEnvelope.fromJson(e.toJson());
      expect(back.isPractice, isTrue);
      // A pre-change entry (no flag stored) decodes as a scored session.
      final legacy = Map<String, dynamic>.from(
        _entry('s', userId: 'u1').toJson(),
      )..remove('isPractice');
      expect(PlaySessionEnvelope.fromJson(legacy).isPractice, isFalse);
    });
  });

  test('backoff grows exponentially and is bounded', () {
    // Deterministic (zero jitter) → the capped base delay.
    Duration at(int n) => playOutboxBackoff(n, random: _ZeroRandom());
    expect(at(1), const Duration(seconds: 1));
    expect(at(2), const Duration(seconds: 2));
    expect(at(3), const Duration(seconds: 4));
    // Capped at 5 minutes however large the attempt.
    expect(at(20), const Duration(minutes: 5));
  });
}

/// A `Random` whose `nextDouble` is 1.0, so full-jitter backoff yields its cap.
class _ZeroRandom implements Random {
  @override
  double nextDouble() => 1.0;
  @override
  bool nextBool() => false;
  @override
  int nextInt(int max) => 0;
}
