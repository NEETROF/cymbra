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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/play_sync_service.dart';
import 'package:music/painters/partition_painter.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/countdown.dart';
import 'package:music/src/rust/api/musicxml.dart' show ScoreDocument;
import 'package:music/state/drum_kit.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/play_session_envelope.dart';
import 'package:music/state/play_sync_notifier.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/practice_settings_store.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

@GenerateNiceMocks([MockSpec<PlaySyncService>()])
import 'practice_range_test.mocks.dart';

/// Connectivity seam that never emits, so no drain fires behind the test.
class _SilentConnectivity implements ConnectivityService {
  @override
  Stream<void> get onOnline => const Stream<void>.empty();
  @override
  Stream<bool> get onlineStatus => const Stream.empty();
  @override
  Future<bool> isOnline() async => true;
  @override
  Future<bool> isDefinitelyOffline() async => !(true);
}

/// No-op retry scheduler so a failed drain doesn't arm a real timer.
class _NoopRetryScheduler implements PlayRetryScheduler {
  @override
  void schedule(Duration delay, void Function() action) {}
  @override
  void cancel() {}
}

/// A [Notation] pinned to a parsed document, so the player derives a real
/// timeline (with a measure table) without touching the byte sources.
/// [apply] pushes a new document, as a real score load would.
class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
  void apply(NotationData next) => state = next;
}

Future<void> _flush() => Future<void>.delayed(Duration.zero);

/// Four 4/4 measures at quarter = 60 (see [sampleFourMeasureDocument]):
/// measure starts 0 / 4000 / 8000 / 12000 ms, song end 16000 ms.
const double _bar = 4000;

/// A container running the player over the four-measure fixture, with the
/// activity-sync seam faked (a selective run reaches it, so it must be
/// overridden even where the test does not assert on it).
ProviderContainer _playerContainer(
  PlaySyncService sync, {
  ScoreDocument? document,
}) => ProviderContainer(
  overrides: [
    midiServiceProvider.overrideWithValue(FakeMidiService()),
    scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
    audioServiceProvider.overrideWithValue(RecordingAudioService()),
    notationProvider.overrideWith(
      () => _FixedNotation(
        NotationData(document: document ?? sampleFourMeasureDocument()),
      ),
    ),
    preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
    playSyncServiceProvider.overrideWithValue(sync),
    connectivityServiceProvider.overrideWithValue(_SilentConnectivity()),
    playRetrySchedulerProvider.overrideWithValue(_NoopRetryScheduler()),
    currentUserIdProvider.overrideWithValue('u1'),
    canUseOnlineServicesProvider.overrideWithValue(true),
  ],
);

MockPlaySyncService _sync() {
  final service = MockPlaySyncService();
  when(service.recordPractice(any)).thenAnswer((_) async => 0);
  when(service.recordSession(any)).thenAnswer((_) async => 0);
  return service;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('normalizePracticeRange', () {
    test('keeps an in-bounds ordered range', () {
      expect(normalizePracticeRange(start: 1, end: 3, measureCount: 8), (
        start: 1,
        end: 3,
      ));
    });

    test('allows a single-measure range', () {
      expect(normalizePracticeRange(start: 2, end: 2, measureCount: 8), (
        start: 2,
        end: 2,
      ));
    });

    test('clamps out-of-bounds bounds to the piece', () {
      expect(normalizePracticeRange(start: -5, end: 99, measureCount: 4), (
        start: 0,
        end: 3,
      ));
    });

    test('reorders a reversed range', () {
      expect(normalizePracticeRange(start: 6, end: 2, measureCount: 8), (
        start: 2,
        end: 6,
      ));
    });

    test('is null when the piece has no measure table', () {
      expect(normalizePracticeRange(start: 0, end: 1, measureCount: 0), isNull);
    });
  });

  group('PlayerData range-aware bounds', () {
    // Three 1s measures; notes fill them so the whole-piece case is unaffected
    // by the leading/trailing trim.
    const data = PlayerData(
      measureStartMs: [0, 1000, 2000],
      songEndMs: 3000,
      notes: [
        TimedNote(pitch: 60, startMs: 0, durationMs: 1000),
        TimedNote(pitch: 62, startMs: 1000, durationMs: 1000),
        TimedNote(pitch: 64, startMs: 2000, durationMs: 1000),
      ],
    );

    test('measureEndMs is the next measure start, songEndMs for the last', () {
      expect(data.measureEndMs(0), 1000);
      expect(data.measureEndMs(1), 2000);
      expect(data.measureEndMs(2), 3000);
    });

    test('no range set is a full run with today\'s bounds', () {
      expect(data.isSelectiveRun, isFalse);
      expect(data.hasPracticeRange, isFalse);
      expect(data.startMs, effectiveStartMs(data.visibleNotes));
      expect(data.endMs, effectiveEndMs(data.visibleNotes, songEndMs: 3000));
    });

    test('an explicit whole-piece range is still a full run', () {
      final whole = data.copyWith(
        practiceStartMeasure: 0,
        practiceEndMeasure: 2,
      );
      expect(whole.hasPracticeRange, isTrue);
      expect(whole.isSelectiveRun, isFalse);
      // Byte-for-byte the untouched behaviour.
      expect(whole.startMs, data.startMs);
      expect(whole.endMs, data.endMs);
    });

    test('a narrower range is selective and maps to the measure bounds', () {
      final sel = data.copyWith(practiceStartMeasure: 1, practiceEndMeasure: 1);
      expect(sel.isSelectiveRun, isTrue);
      expect(sel.startMs, 1000);
      expect(sel.endMs, 2000);
    });

    test('a range ending on the last measure ends at songEndMs', () {
      final sel = data.copyWith(practiceStartMeasure: 1, practiceEndMeasure: 2);
      expect(sel.isSelectiveRun, isTrue);
      expect(sel.startMs, 1000);
      expect(sel.endMs, 3000);
    });

    test('a selective run shows ONLY the passage\'s notes', () {
      // Seeing the whole piece during a passage hides where the loop ends, and
      // would let the Wait-Mode gate hold at an onset the loop never reaches.
      expect(data.visibleNotes.map((n) => n.pitch), [60, 62, 64]);
      final sel = data.copyWith(practiceStartMeasure: 1, practiceEndMeasure: 1);
      expect(sel.visibleNotes.map((n) => n.pitch), [62]);
      // The gate and the expected-key preview follow, since both derive from it.
      expect(sel.nextOnsetAfter(0), 1000);
      expect(sel.nextOnsetAfter(1000), isNull);
    });

    test('a range on a piece with no measure table is inert', () {
      const demo = PlayerData(
        songEndMs: 1000,
        practiceStartMeasure: 0,
        practiceEndMeasure: 0,
      );
      expect(demo.hasPracticeRange, isFalse);
      expect(demo.isSelectiveRun, isFalse);
    });
  });

  group('Player practice range', () {
    late ProviderContainer container;

    Player notifier() => container.read(playerProvider.notifier);
    PlayerData read() => container.read(playerProvider);
    ScoringData scoring() => container.read(performanceScorerProvider);

    Future<void> build() async {
      container = _playerContainer(_sync());
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();
    }

    test('the fixture has the expected measure table', () async {
      await build();
      expect(read().measureStartMs, [0, 4000, 8000, 12000]);
      expect(read().songEndMs, 16000);
      expect(read().lastMeasureIndex, 3);
    });

    test(
      'setPracticeRange normalizes, clamps and seeks the range start',
      () async {
        await build();
        // Reversed and out of bounds → normalized to [1, 3].
        notifier().setPracticeRange(9, 1);
        expect(read().practiceStartMeasure, 1);
        expect(read().practiceEndMeasure, 3);
        expect(read().isSelectiveRun, isTrue);
        expect(read().elapsedMs, _bar);
        expect(read().startMs, _bar);
        expect(read().endMs, 16000);
      },
    );

    test(
      'clearPracticeRange returns to a full run at the piece start',
      () async {
        await build();
        notifier().setPracticeRange(2, 2);
        expect(read().isSelectiveRun, isTrue);
        notifier().clearPracticeRange();
        expect(read().practiceStartMeasure, isNull);
        expect(read().practiceEndMeasure, isNull);
        expect(read().isSelectiveRun, isFalse);
        expect(read().elapsedMs, read().startMs);
      },
    );

    test('a full run still arms the scorer (no regression)', () async {
      await build();
      notifier().setPlaying(true);
      expect(scoring().active, isTrue);
    });

    test('a selective run never arms the scorer', () async {
      await build();
      notifier().setPracticeRange(1, 2);
      notifier().setPlaying(true);
      expect(scoring().active, isFalse);
    });

    test('starting a selective run cancels an in-flight scored run', () async {
      await build();
      notifier().setPlaying(true);
      expect(scoring().active, isTrue);
      notifier().setPracticeRange(1, 2);
      expect(scoring().active, isFalse);
    });

    test(
      'a looping selective run wraps to the range start, not the top',
      () async {
        await build();
        notifier().setPracticeRange(1, 1); // bar 2 only: [4000, 8000)
        notifier().setPlaying(true);
        // Free run so the Wait-Mode gate does not freeze the playhead.
        if (read().waitMode) notifier().toggleWaitMode();
        notifier().advance(3999);
        expect(read().elapsedMs, closeTo(7999, 0.001));
        notifier().advance(2); // crosses the range end
        expect(read().elapsedMs, _bar);
        expect(read().isPlaying, isTrue);
      },
    );

    test(
      'a full unscored run still loops to the top (no regression)',
      () async {
        await build();
        notifier().setPlaying(true);
        if (read().waitMode) notifier().toggleWaitMode();
        // Cancel the scored run so the unscored loop path is taken, as today.
        container.read(performanceScorerProvider.notifier).cancelRun();
        notifier().advance(16001);
        expect(read().elapsedMs, read().startMs);
        expect(read().isPlaying, isTrue);
      },
    );

    test(
      'a selective run does not advance the playthrough high-water mark',
      () async {
        // Cross-feature guard (change: add-post-play-rating-prompt): drilling the
        // LAST bars must not make the piece look played end-to-end. The rating
        // prompt counts notes BEFORE `furthestElapsedMs`, so crediting the range
        // end would mark every earlier, never-played note as heard.
        await build();
        notifier().setPracticeRange(3, 3); // bar 4 only: [12000, 16000)
        notifier().setPlaying(true);
        if (read().waitMode) notifier().toggleWaitMode();
        final before = read().furthestElapsedMs;
        // Several laps of the last bar — each one reaches the piece's end time.
        for (var i = 0; i < 3; i++) {
          notifier().advance(_bar + 1);
        }
        expect(read().elapsedMs, 12000); // wrapped back to the range start
        expect(
          read().furthestElapsedMs,
          before,
          reason: 'practice is not a playthrough',
        );

        // A FULL run still credits it, exactly as before.
        notifier().clearPracticeRange();
        notifier().setPlaying(true);
        if (read().waitMode) notifier().toggleWaitMode();
        notifier().advance(_bar + 1);
        expect(read().furthestElapsedMs, greaterThan(before));
      },
    );

    test('loading a new document drops a stale range', () async {
      await build();
      notifier().setPracticeRange(1, 2);
      expect(read().isSelectiveRun, isTrue);
      // A different score arrives (here: the same fixture re-parsed, so a new
      // document instance the player treats as a fresh load).
      (container.read(notationProvider.notifier) as _FixedNotation).apply(
        NotationData(document: sampleFourMeasureDocument()),
      );
      await _flush();
      expect(read().practiceStartMeasure, isNull);
      expect(read().isSelectiveRun, isFalse);
    });
  });

  // Task 2.5 (change: add-practice-focus-controls): the focus selection is
  // session-only for the same reason a range is — it describes the passage
  // being worked on, not a preference. A selection carried into another score
  // would hand it a kit with holes in it, and the ids need not even exist there.
  group('per-piece focus is session-only', () {
    late ProviderContainer container;
    Player notifier() => container.read(playerProvider.notifier);
    PlayerData read() => container.read(playerProvider);

    late FakePreferencesService prefs;

    Future<void> build() async {
      prefs = FakePreferencesService();
      container = ProviderContainer(
        overrides: [
          midiServiceProvider.overrideWithValue(FakeMidiService()),
          scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
          audioServiceProvider.overrideWithValue(RecordingAudioService()),
          notationProvider.overrideWith(
            () => _FixedNotation(
              NotationData(document: sampleOpenGrooveDocument()),
            ),
          ),
          preferencesServiceProvider.overrideWithValue(prefs),
          playSyncServiceProvider.overrideWithValue(_sync()),
          connectivityServiceProvider.overrideWithValue(_SilentConnectivity()),
          playRetrySchedulerProvider.overrideWithValue(_NoopRetryScheduler()),
          currentUserIdProvider.overrideWithValue('u1'),
          canUseOnlineServicesProvider.overrideWithValue(true),
        ],
      );
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();
    }

    test('every piece is in focus on load', () async {
      await build();
      expect(read().isPercussion, isTrue);
      expect(read().mutedDrumPieces, isEmpty);
      expect(read().focusedDrumPieces, read().kitPieceIds.toSet());
    });

    test('loading a new document drops a stale selection', () async {
      await build();
      notifier().muteDrumPiece(kKickPieceId);
      expect(read().mutedDrumPieces, isNotEmpty);
      (container.read(notationProvider.notifier) as _FixedNotation).apply(
        NotationData(document: sampleOpenGrooveDocument()),
      );
      await _flush();
      expect(read().mutedDrumPieces, isEmpty);
    });

    test('nothing is written to the preferences seam', () async {
      await build();
      final before = {...prefs.store};
      notifier()
        ..muteDrumPiece(kKickPieceId)
        ..soloDrumPiece('kitPieceSnare')
        ..clearDrumFocus();
      await _flush();
      expect(prefs.store, before);
    });
  });

  group('PracticeSettings', () {
    const saved = PracticeSettings(startMeasure: 4, endMeasure: 7);

    test('round-trips through JSON', () {
      final back = PracticeSettings.fromJson(saved.toJson());
      expect(back.startMeasure, 4);
      expect(back.endMeasure, 7);
    });

    test('is clamped to a score that shrank', () {
      final c = saved.clampedTo(6)!; // last measure is now 5
      expect(c.startMeasure, 4);
      expect(c.endMeasure, 5);
    });

    test('falls back to the whole piece when it cannot be salvaged', () {
      // A single-measure score: the clamped range would BE the whole piece,
      // which is a full run, not a practice selection.
      expect(saved.clampedTo(1), isNull);
      // No measure table at all.
      expect(saved.clampedTo(0), isNull);
      // A range that already spans everything is likewise not a selection.
      expect(
        const PracticeSettings(startMeasure: 0, endMeasure: 9).clampedTo(4),
        isNull,
      );
    });
  });

  group('Player loop count and tempo ramp', () {
    late ProviderContainer container;

    Player notifier() => container.read(playerProvider.notifier);
    PlayerData read() => container.read(playerProvider);

    Future<void> build() async {
      container = _playerContainer(_sync());
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();
    }

    /// Runs one full pass of a one-bar range (4000 ms) in free mode.
    void lap() => notifier().advance(_bar + 1);

    Future<void> startSelective() async {
      await build();
      notifier().setPracticeRange(1, 1);
      notifier().setPlaying(true);
      if (read().waitMode) notifier().toggleWaitMode();
    }

    test('an infinite loop keeps wrapping', () async {
      await startSelective();
      for (var i = 0; i < 5; i++) {
        lap();
      }
      expect(read().isPlaying, isTrue);
      expect(read().elapsedMs, _bar);
    });

    test('every lap re-arms the 3-2-1 countdown', () async {
      // A practice loop throws the player back to the start of the passage, so
      // each lap opens with the same get-ready beat as the first one.
      await startSelective();
      expect(read().countdownMs, 0);
      lap(); // wraps
      expect(read().elapsedMs, _bar);
      expect(read().countdownMs, kCountdownStartMs);
      expect(read().isPlaying, isTrue);
    });

    test('starting a passage always opens on the countdown', () async {
      await build();
      notifier().setPracticeRange(1, 1);
      // Wait Mode on — a full run would skip the countdown there, a passage
      // never does.
      if (!read().waitMode) notifier().toggleWaitMode();
      notifier().startPlayback();
      expect(read().countdownMs, kCountdownStartMs);
    });
  });

  group('per-score practice persistence', () {
    test('interleaved writes do not clobber each other', () async {
      final store = PracticeSettingsStore(FakePreferencesService());
      // The player fires these back-to-back without awaiting (loop settings then
      // the range); the last one requested must be the one that survives.
      unawaited(store.clear('score-1'));
      unawaited(
        store.save(
          'score-1',
          const PracticeSettings(startMeasure: 0, endMeasure: 1),
        ),
      );
      unawaited(store.clear('score-1'));
      await store.save(
        'score-1',
        const PracticeSettings(startMeasure: 2, endMeasure: 3),
      );
      final saved = await store.load('score-1');
      expect(saved, isNotNull);
      expect(saved!.startMeasure, 2);
      expect(saved.endMeasure, 3);
    });

    test('settings are kept per score', () async {
      final store = PracticeSettingsStore(FakePreferencesService());
      await store.save(
        'a',
        const PracticeSettings(startMeasure: 0, endMeasure: 1),
      );
      await store.save(
        'b',
        const PracticeSettings(startMeasure: 5, endMeasure: 6),
      );
      await store.clear('a');
      expect(await store.load('a'), isNull);
      expect((await store.load('b'))!.startMeasure, 5);
    });

    test('a chosen range is saved and clamped back on load', () async {
      final container = _playerContainer(_sync());
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();

      final notifier = container.read(playerProvider.notifier);
      notifier.setPracticeRange(1, 2);
      await pumpEventQueue();

      final store = container.read(practiceSettingsStoreProvider);
      final key = pieceIdentityOf(
        container.read(selectedScoreProvider),
        container.read(playerProvider).title,
      );
      final saved = await store.load(key);
      expect(saved, isNotNull);
      expect(saved!.startMeasure, 1);
      expect(saved.endMeasure, 2);

      // Returning to a full run forgets the selection.
      notifier.clearPracticeRange();
      await pumpEventQueue();
      expect(await store.load(key), isNull);
    });
  });

  group('measureAtPosition', () {
    const rects = <MeasureHit>[
      (measure: 0, rect: Rect.fromLTRB(0, 0, 100, 50)),
      (measure: 1, rect: Rect.fromLTRB(100, 0, 200, 50)),
      (measure: 2, rect: Rect.fromLTRB(0, 60, 100, 110)),
    ];

    test('maps a point to the measure whose rectangle contains it', () {
      expect(measureAtPosition(rects, const Offset(50, 25)), 0);
      expect(measureAtPosition(rects, const Offset(150, 25)), 1);
      expect(measureAtPosition(rects, const Offset(50, 80)), 2);
    });

    test('is null outside every measure (header, gutter, gap)', () {
      expect(measureAtPosition(rects, const Offset(300, 25)), isNull);
      expect(measureAtPosition(rects, const Offset(50, 55)), isNull);
      expect(measureAtPosition(const [], Offset.zero), isNull);
    });
  });

  group('practice activity capture', () {
    late MockPlaySyncService service;
    late ProviderContainer container;

    Player notifier() => container.read(playerProvider.notifier);
    PlayerData read() => container.read(playerProvider);

    late ProviderSubscription<PlayerData> keepAlive;

    Future<void> build() async {
      service = _sync();
      container = _playerContainer(service);
      addTearDown(container.dispose);
      keepAlive = container.listen(
        playerProvider,
        (_, _) {},
        fireImmediately: true,
      );
      await _flush();
    }

    /// Leaves the player screen: the last listener goes, so the auto-dispose
    /// player notifier is torn down while the app container (and the sync
    /// notifier) stays alive — exactly as when the screen unmounts.
    Future<void> leavePlayer() async {
      keepAlive.close();
      await pumpEventQueue();
    }

    /// Plays a selective run long enough to cross several range ends (laps).
    void playLaps(int laps) {
      notifier().setPlaying(true);
      if (read().waitMode) notifier().toggleWaitMode();
      for (var i = 0; i < laps; i++) {
        notifier().advance(_bar + 1);
      }
    }

    List<PlaySessionEnvelope> practicesSent() => verify(
      service.recordPractice(captureAny),
    ).captured.cast<PlaySessionEnvelope>();

    test('a looping practice is recorded ONCE, as soon as it sounds', () async {
      await build();
      notifier().setPracticeRange(1, 1);
      playLaps(4);
      await pumpEventQueue();
      // Durable from the first onset rather than held until the session ends
      // (change: add-practice-streak) — see the interrupted-practice test below.
      final sent = practicesSent();
      expect(sent, hasLength(1));
      expect(sent.single.isPractice, isTrue);
      expect(sent.single.overallSyncPct, 0);
      // Laps do not add records: the capture above is now marked verified, so
      // `verifyNever` passes only if no FURTHER capture happened.
      playLaps(3);
      await pumpEventQueue();
      verifyNever(service.recordPractice(any));
      // A practice never enters the scored path.
      verifyNever(service.recordSession(any));
    });

    test('an interrupted practice is already recorded', () async {
      // THE point of capturing at the first onset: no range change, no return to
      // a full run, no leaving the screen — the app is simply killed mid-loop.
      // The record must already be in the durable outbox, because a lost practice
      // now also costs the player a streak day they actually earned.
      await build();
      notifier().setPracticeRange(1, 2);
      playLaps(2);
      await pumpEventQueue();
      expect(practicesSent(), hasLength(1));
    });

    test('a new range opens a fresh record', () async {
      await build();
      notifier().setPracticeRange(1, 1);
      playLaps(1);
      await pumpEventQueue();
      expect(practicesSent(), hasLength(1));
      // Closing the session re-arms the guard: drilling a different passage is a
      // second practice session, not a continuation of the first.
      notifier().setPracticeRange(2, 2);
      playLaps(1);
      await pumpEventQueue();
      expect(practicesSent(), hasLength(1));
    });

    test('returning to a full run closes the practice session', () async {
      await build();
      notifier().setPracticeRange(1, 2);
      playLaps(1);
      notifier().clearPracticeRange();
      await pumpEventQueue();
      expect(practicesSent(), hasLength(1));
    });

    test('leaving the player does not re-record the practice', () async {
      await build();
      notifier().setPracticeRange(1, 2);
      playLaps(1);
      await leavePlayer();
      expect(practicesSent(), hasLength(1));
    });

    test('a range opened but never played is not counted', () async {
      await build();
      notifier().setPracticeRange(1, 2);
      // No advance at all — the player quit before playing a note.
      notifier().clearPracticeRange();
      await pumpEventQueue();
      verifyNever(service.recordPractice(any));
    });

    test('a full run never produces a practice record', () async {
      await build();
      notifier().setPlaying(true);
      if (read().waitMode) notifier().toggleWaitMode();
      notifier().advance(_bar + 1);
      notifier().clearPracticeRange();
      await pumpEventQueue();
      verifyNever(service.recordPractice(any));
    });
  });

  group('rewindTargetMs', () {
    const starts = [0, 4000, 8000, 12000];

    test('mid-measure lands on the top of that measure', () {
      expect(
        rewindTargetMs(elapsedMs: 6000, measureStartMs: starts, minMs: 0),
        4000,
      );
    });

    test('at (or within epsilon of) a start lands on the previous bar', () {
      expect(
        rewindTargetMs(elapsedMs: 4000, measureStartMs: starts, minMs: 0),
        0,
      );
      expect(
        rewindTargetMs(elapsedMs: 4200, measureStartMs: starts, minMs: 0),
        0,
      );
    });

    test('just past epsilon means the current bar again', () {
      expect(
        rewindTargetMs(elapsedMs: 4301, measureStartMs: starts, minMs: 0),
        4000,
      );
    });

    test('clamps at the effective start', () {
      // Already at the piece's top.
      expect(rewindTargetMs(elapsedMs: 0, measureStartMs: starts, minMs: 0), 0);
      // Selective run from bar 2: stepping back from its first bar stays put.
      expect(
        rewindTargetMs(elapsedMs: 4100, measureStartMs: starts, minMs: 4000),
        4000,
      );
    });

    test('lands on the last measure from beyond it', () {
      expect(
        rewindTargetMs(elapsedMs: 15000, measureStartMs: starts, minMs: 0),
        12000,
      );
    });

    test('is null without a measure table', () {
      expect(
        rewindTargetMs(elapsedMs: 1000, measureStartMs: const [], minMs: 0),
        isNull,
      );
    });
  });

  group('Player measure rewind', () {
    late ProviderContainer container;

    Player notifier() => container.read(playerProvider.notifier);
    PlayerData read() => container.read(playerProvider);
    ScoringData scoring() => container.read(performanceScorerProvider);

    Future<void> build() async {
      container = _playerContainer(_sync());
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();
    }

    test('taps stack back one bar at a time, preserving playback', () async {
      await build();
      notifier().setPlaying(true);
      if (read().waitMode) notifier().toggleWaitMode();
      notifier().advance(6000); // mid bar 2
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, _bar, reason: 'first tap: top of the bar');
      expect(read().isPlaying, isTrue);
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, 0, reason: 'second tap: the bar before');
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, 0, reason: 'clamped at the piece top');
      expect(read().isPlaying, isTrue);
    });

    test('discards the scored run and never re-arms mid-piece', () async {
      await build();
      notifier().setPlaying(true);
      expect(scoring().active, isTrue);
      if (read().waitMode) notifier().toggleWaitMode();
      notifier().advance(6000);
      notifier().rewindOneMeasure();
      expect(scoring().active, isFalse);
      // Playing on from mid-piece must not silently re-open a scored run.
      notifier().advance(1000);
      expect(scoring().active, isFalse);
      // The explicit restart-from-top does re-arm, as today.
      notifier().restartFromTop();
      expect(scoring().active, isTrue);
    });

    test('clamps at the range start on a selective run', () async {
      await build();
      notifier().setPracticeRange(1, 2); // bars 2–3, playhead at 4000
      notifier().setPlaying(true);
      if (read().waitMode) notifier().toggleWaitMode();
      notifier().advance(4500); // mid bar 3
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, 8000);
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, _bar);
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, _bar, reason: 'never below the range start');
    });

    test('clears the Wait-Mode latches', () async {
      await build();
      notifier().setPlaying(true);
      notifier().noteOn(72); // latch bar 1's C5 onset
      expect(read().gateSatisfied, contains(72));
      notifier().rewindOneMeasure();
      expect(read().gateSatisfied, isEmpty);
      expect(read().consumedHeld, isEmpty);
    });

    test('is a no-op without a measure table (the demo score)', () async {
      // No document → the built-in demo, which has no measure table.
      container = ProviderContainer(
        overrides: [
          midiServiceProvider.overrideWithValue(FakeMidiService()),
          scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
          audioServiceProvider.overrideWithValue(RecordingAudioService()),
          notationProvider.overrideWith(
            () => _FixedNotation(const NotationData()),
          ),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(),
          ),
          playSyncServiceProvider.overrideWithValue(_sync()),
          connectivityServiceProvider.overrideWithValue(_SilentConnectivity()),
          playRetrySchedulerProvider.overrideWithValue(_NoopRetryScheduler()),
          currentUserIdProvider.overrideWithValue('u1'),
          canUseOnlineServicesProvider.overrideWithValue(true),
        ],
      );
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();
      expect(read().measureStartMs, isEmpty);
      final before = read().elapsedMs;
      notifier().rewindOneMeasure();
      expect(read().elapsedMs, before);
    });
  });

  // With repeats, a full run follows the unrolled playback order while a
  // selective practice run deliberately stays linear over the WRITTEN
  // measures (spec: measure-range-practice delta) — the notifier swaps the
  // timeline on range set/clear.
  group('practice on a repeat-carrying piece', () {
    late ProviderContainer container;

    Player notifier() => container.read(playerProvider.notifier);
    PlayerData read() => container.read(playerProvider);

    Future<void> build() async {
      container = _playerContainer(_sync(), document: sampleRepeatDocument());
      addTearDown(container.dispose);
      container.listen(playerProvider, (_, _) {}, fireImmediately: true);
      await _flush();
    }

    test(
      'a full run is unrolled; a range swaps to the written order',
      () async {
        await build();
        // Unrolled: the repeated bar has two played slots.
        expect(read().measureStartMs, [0, 4000, 8000]);
        expect(read().writtenMeasureOf, [0, 0, 1]);
        expect(read().writtenMeasureCount, 2);
        expect(read().songEndMs, 12000);

        // The range is picked in written measures (2 of them) and the timeline
        // becomes linear so the loop means "these bars, once".
        notifier().setPracticeRange(1, 1);
        expect(read().practiceStartMeasure, 1);
        expect(read().measureStartMs, [0, 4000]);
        expect(read().songEndMs, 8000);
        expect(read().startMs, 4000);
        expect(read().endMs, 8000);

        // Clearing returns to the unrolled performance route.
        notifier().clearPracticeRange();
        expect(read().measureStartMs, [0, 4000, 8000]);
        expect(read().songEndMs, 12000);
      },
    );
  });
}
