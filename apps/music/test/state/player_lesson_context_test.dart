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
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/lesson_context.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/player_preferences.dart';
import 'package:music/state/session_summary.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

LessonContext _lesson() => LessonContext(
  lessonTitle: 'Fa et sol',
  hands: Hand.right,
  speed: 0.6,
  waitMode: false,
  metronome: true,
  readingAid: NoteReadingAid.nameAndRhythm,
  passPct: 80,
  onResult: (_) {},
);

Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  late FakeMidiService midi;
  late ProviderContainer container;
  late FakePreferencesService prefs;

  /// Builds the player; [armed] arms a lesson context BEFORE the player builds,
  /// the way a piece step does right before opening the score.
  Future<void> build({LessonContext? armed}) async {
    midi = FakeMidiService();
    prefs = FakePreferencesService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        preferencesServiceProvider.overrideWithValue(prefs),
      ],
    );
    addTearDown(container.dispose);
    addTearDown(midi.close);
    // The player's own saved preferences, which the lesson must never touch.
    container.read(playerPreferencesProvider.notifier)
      ..setHands(Hand.both)
      ..setSpeed(1.0)
      ..setMetronome(enabled: false)
      ..setNoteReadingAid(NoteReadingAid.name);
    await _flush();
    if (armed != null) {
      container.read(pendingLessonContextProvider.notifier).arm(armed);
    }
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await _flush();
  }

  PlayerData read() => container.read(playerProvider);
  Player notifier() => container.read(playerProvider.notifier);

  // change: add-music-piece-lessons
  group('lesson context', () {
    test('an armed context seeds the run instead of the preferences', () async {
      await build(armed: _lesson());
      final d = read();
      expect(d.lessonContext, isNotNull);
      expect(d.selectedHands, Hand.right);
      expect(d.speed, 0.6);
      expect(d.waitMode, isFalse);
      expect(d.metronomeEnabled, isTrue);
      expect(d.readingAid, NoteReadingAid.nameAndRhythm);
    });

    test('the imposed settings are read-only for the run and the preferences '
        'stay untouched', () async {
      await build(armed: _lesson());
      final before = container.read(playerPreferencesProvider);
      notifier()
        ..setSpeed(1.0)
        ..toggleMetronome()
        ..toggleWaitMode()
        ..setReadingAid(NoteReadingAid.off)
        ..setSelectedHands(Hand.both);
      await _flush();
      final d = read();
      expect(d.speed, 0.6);
      expect(d.metronomeEnabled, isTrue);
      expect(d.waitMode, isFalse);
      expect(d.readingAid, NoteReadingAid.nameAndRhythm);
      expect(d.selectedHands, Hand.right);
      expect(container.read(playerPreferencesProvider), before);
    });

    test(
      'without a context the run seeds from the preferences as always',
      () async {
        await build();
        final d = read();
        expect(d.lessonContext, isNull);
        expect(d.selectedHands, Hand.both);
        expect(d.speed, 1.0);
        expect(d.metronomeEnabled, isFalse);
        expect(d.waitMode, isTrue);
        // And the setters write through.
        notifier().setSpeed(0.5);
        await _flush();
        expect(container.read(playerPreferencesProvider).speed, 0.5);
      },
    );

    test('passes() judges the run against the step\'s bar', () {
      final lesson = _lesson();
      expect(lesson.passPct, 80);
      expect(lesson.passes(_result(84)), isTrue);
      expect(lesson.passes(_result(80)), isTrue);
      expect(lesson.passes(_result(79.9)), isFalse);
    });
  });
}

/// A result with the given overall percentage and nothing else of interest.
SessionResult _result(double pct) => SessionResult(
  pieceId: 'p',
  title: 't',
  hands: 'right',
  overallSyncPct: pct,
  runMode: RunMode.free,
  freeSyncPct: pct,
  waitSyncPct: null,
  freeOnsetCount: 1,
  waitOnsetCount: 0,
  avgFreeOffsetMs: 0,
  avgReactionMs: null,
  timing: pct,
  correctness: pct,
  sustain: null,
  verdictCounts: const {},
  wrongNotes: 0,
  bestCombo: 1,
  notes: const [],
  playedAtMs: 0,
  speed: 1,
  inputSource: 'midi',
);
