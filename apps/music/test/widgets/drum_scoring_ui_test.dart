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
import 'package:music/painters/drum_cascade_painter.dart';
import 'package:music/painters/hit_effects_painter.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/play_sync_notifier.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/widgets/mistake_replay.dart';
import 'package:music/widgets/score_chip.dart';
import 'package:music/widgets/session_summary_modal.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

// The gauge, the summary and the replay over a percussion run (change:
// add-drum-scoring).

const _entry = CatalogEntry(
  id: 'drums-ui',
  title: 'Groove ouvert',
  composer: 'Cymbra',
  assetPath: 'assets/scores/groove.musicxml',
  level: PracticeLevel.beginner,
);

class _EmptyLeaderboardService implements LeaderboardService {
  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async => Leaderboard.empty;

  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};
}

Future<void> _frames(WidgetTester tester, {int count = 1}) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<ProviderContainer> _pumpPlayer(WidgetTester tester) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: sampleOpenGrooveDocument()),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
    ],
  );
  container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const PlayerScreen()),
    ),
  );
  await _frames(tester, count: 12);
  final validate = find.widgetWithText(FilledButton, 'Play');
  if (validate.evaluate().isNotEmpty) {
    await tester.tap(validate);
    await _frames(tester, count: 6);
  }
  return container;
}

/// Records which submission path a finished run took (change:
/// add-practice-focus-controls, design D7), so the decision is asserted on the
/// **seam** rather than on the absence of something in the UI.
///
/// A hand double rather than a mockito mock: `PlaySyncNotifier` is a Riverpod
/// notifier, so the override has to construct the generated base class — one of
/// the documented exceptions in the `flutter-testing` skill.
class _RecordingPlaySync extends PlaySyncNotifier {
  _RecordingPlaySync(this._calls);

  /// The test's own sink. Held privately: a notifier exposes its API through
  /// `state`, and a public field here trips `avoid_public_notifier_properties`.
  final List<String> _calls;

  @override
  int build() => 0;

  @override
  Future<void> captureSession(SessionResult result) async {
    _calls.add('session');
  }

  @override
  Future<void> capturePractice({String? scoreId}) async {
    _calls.add('practice');
  }
}

Future<({ProviderContainer container, List<String> calls})> _pumpPlayerWithSync(
  WidgetTester tester,
) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final calls = <String>[];
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: sampleOpenGrooveDocument()),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
      playSyncNotifierProvider.overrideWith(() => _RecordingPlaySync(calls)),
    ],
  );
  container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const PlayerScreen()),
    ),
  );
  await _frames(tester, count: 12);
  final validate = find.widgetWithText(FilledButton, 'Play');
  if (validate.evaluate().isNotEmpty) {
    await tester.tap(validate);
    await _frames(tester, count: 6);
  }
  return (container: container, calls: calls);
}

/// Runs the loaded percussion score from the top to past its end, free-run, so
/// the scorer finalises and the screen routes the result.
Future<void> _runToEnd(WidgetTester tester, ProviderContainer c) async {
  final player = c.read(playerProvider.notifier);
  if (c.read(playerProvider).waitMode) player.toggleWaitMode();
  player.setPlaying(true);
  await _frames(tester, count: 4);
  player.advance(c.read(playerProvider).scoredRunEndMs + 1000);
  await _frames(tester, count: 8);
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

// --- summary / replay fixtures ------------------------------------------

NoteJudgment _stroke(
  int i,
  TimingVerdict v, {
  int pitch = 38,
  bool wrong = false,
  int startMs = 0,
}) => NoteJudgment(
  noteIndex: wrong ? -1 : i,
  pitch: pitch,
  startMs: startMs,
  waitMode: false,
  verdict: v,
  timingOffsetMs: wrong ? null : 0,
  sustainRatio: null, // percussion: absent, never zero
  wrong: wrong,
);

SessionResult _drumResult() => SessionResult.fromJudgments(
  pieceId: 'drums-ui',
  title: 'Groove ouvert',
  hands: 'both',
  judgments: [
    _stroke(0, TimingVerdict.perfect, pitch: 42),
    _stroke(1, TimingVerdict.late, pitch: 38, startMs: 600),
    _stroke(2, TimingVerdict.missed, pitch: 36, startMs: 1200),
    _stroke(3, TimingVerdict.perfect, pitch: 47, wrong: true, startMs: 900),
  ],
  bestCombo: 1,
  playedAtMs: 0,
  speed: 1,
  percussion: true,
);

SessionResult _keyboardResult() => SessionResult.fromJudgments(
  pieceId: 'p',
  title: 'Sonata',
  hands: 'both',
  judgments: [
    const NoteJudgment(
      noteIndex: 0,
      pitch: 60,
      startMs: 0,
      waitMode: false,
      verdict: TimingVerdict.perfect,
      timingOffsetMs: 0,
      sustainRatio: 1,
    ),
  ],
  bestCombo: 1,
  playedAtMs: 0,
  speed: 1,
);

Widget _scoped(Widget home) => UncontrolledProviderScope(
  container: ProviderContainer(
    overrides: [
      leaderboardServiceProvider.overrideWithValue(_EmptyLeaderboardService()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  ),
  child: localizedApp(home),
);

Future<void> _openSummary(WidgetTester tester, SessionResult r) async {
  await tester.pumpWidget(
    _scoped(
      Scaffold(
        body: Builder(
          builder: (context) => ElevatedButton(
            onPressed: () => showSessionSummary(context, r),
            child: const Text('go'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('go'));
  await tester.pumpAndSettle();
}

void main() {
  group('the score chip is the only live gauge', () {
    testWidgets('hidden before a run, shown in the cascade during one', (
      tester,
    ) async {
      final c = await _pumpPlayer(tester);
      // The chip is mounted but takes no space at all while no run is active.
      expect(find.byType(ScoreChip), findsOneWidget);
      expect(tester.getSize(find.byType(ScoreChip)), Size.zero);

      c.read(playerProvider.notifier)
        ..toggleWaitMode()
        ..setPlaying(true);
      await _frames(tester, count: 3);
      expect(c.read(performanceScorerProvider).active, isTrue);
      // The cascade is on screen and the chip now occupies the top bar —
      // never the lanes.
      expect(
        find.byWidgetPredicate(
          (w) => w is CustomPaint && w.painter is DrumCascadePainter,
        ),
        findsOneWidget,
      );
      expect(tester.getSize(find.byType(ScoreChip)).height, greaterThan(0));
      expect(
        tester.getRect(find.byType(ScoreChip)).bottom,
        lessThan(tester.getRect(find.byKey(const Key('render-area'))).top),
      );
      expect(find.text('100%'), findsOneWidget);
      await _teardown(tester, c);
    });

    testWidgets('shown in a percussion notation mode too', (tester) async {
      final c = await _pumpPlayer(tester);
      c.read(playerProvider.notifier)
        ..toggleWaitMode()
        ..setPlaying(true);
      await _frames(tester, count: 3);
      await tester.tap(find.text('Staff'));
      await _frames(tester, count: 3);
      expect(
        find.byWidgetPredicate(
          (w) => w is CustomPaint && w.painter is StaffPainter,
        ),
        findsWidgets,
      );
      expect(tester.getSize(find.byType(ScoreChip)).height, greaterThan(0));
      await _teardown(tester, c);
    });

    testWidgets('nothing floats over the play surface: the feedback is IN it', (
      tester,
    ) async {
      final c = await _pumpPlayer(tester);
      c.read(playerProvider.notifier)
        ..toggleWaitMode()
        ..setPlaying(true);
      await _frames(tester, count: 3);
      // A percussion run carries no spark layer at all: the surface lights the
      // note the stroke answered and flashes the piece that was struck, both
      // inside the painter that drew them. The separate layer that used to
      // float over the lanes anchored its sparks to the bottom of the screen —
      // under the drawn kit, where nothing happens — and the keyboard's own
      // layer has never applied here.
      expect(
        find.byWidgetPredicate(
          (w) => w is CustomPaint && w.painter is HitEffectsPainter,
        ),
        findsNothing,
      );
      await _teardown(tester, c);
    });

    testWidgets(
      'a percussion notation mode draws no keyboard-anchored sparks',
      (tester) async {
        final c = await _pumpPlayer(tester);
        c.read(playerProvider.notifier)
          ..toggleWaitMode()
          ..setPlaying(true);
        await _frames(tester, count: 3);
        await tester.tap(find.text('Staff'));
        await _frames(tester, count: 3);
        // The staff shows the pad strip, not a keyboard: piano-keyed sparks
        // would land on coordinates that mean nothing here.
        expect(
          find.byWidgetPredicate(
            (w) => w is CustomPaint && w.painter is HitEffectsPainter,
          ),
          findsNothing,
        );
        await _teardown(tester, c);
      },
    );

    testWidgets(
      'the combo climbs on landed strokes and resets on a wrong one',
      (tester) async {
        final c = await _pumpPlayer(tester);
        final player = c.read(playerProvider.notifier)
          ..toggleWaitMode()
          ..setPlaying(true);
        await _frames(tester, count: 3);
        player
          ..noteOn(42)
          ..noteOn(49)
          ..noteOn(36);
        await _frames(tester, count: 2);
        expect(c.read(performanceScorerProvider).combo, 3);
        expect(c.read(performanceScorerProvider).recentHits, hasLength(3));

        player.noteOn(47); // a tom nobody asked for
        await _frames(tester, count: 2);
        expect(c.read(performanceScorerProvider).combo, 0);
        expect(c.read(performanceScorerProvider).bestCombo, 3);
        await _teardown(tester, c);
      },
    );
  });

  group('the percussion summary shows two dimensions', () {
    testWidgets('no sustain row — absent, not an empty bar', (tester) async {
      await _openSummary(tester, _drumResult());
      expect(find.text('Timing'), findsOneWidget);
      expect(find.text('Correct notes'), findsOneWidget);
      expect(find.text('Sustain'), findsNothing);
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
    });

    testWidgets('a keyboard summary keeps its three dimensions', (
      tester,
    ) async {
      await _openSummary(tester, _keyboardResult());
      expect(find.text('Timing'), findsOneWidget);
      expect(find.text('Correct notes'), findsOneWidget);
      expect(find.text('Sustain'), findsOneWidget);
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
    });

    testWidgets('the per-verdict counts, best combo and sub-score are there', (
      tester,
    ) async {
      final result = _drumResult();
      await _openSummary(tester, result);
      expect(find.textContaining('perfect'), findsOneWidget);
      expect(find.textContaining('off'), findsOneWidget);
      expect(find.textContaining('missed'), findsOneWidget);
      expect(find.textContaining('Best ×'), findsOneWidget);
      // A pure free run shows its one sub-score.
      expect(find.text('Tempo'), findsOneWidget);
      expect(find.text('Reaction'), findsNothing);
      // The selection is reported in the drummer's own vocabulary.
      // A percussion run reports the keyboard's `both` since the hands/feet
      // reading was removed (change: add-practice-focus-controls).
      expect(find.textContaining('Both hands'), findsOneWidget);
      // The explicit-choice contract is unchanged.
      expect(find.text('Replay mistakes'), findsOneWidget);
      expect(find.text('Retry'), findsOneWidget);
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
    });
  });

  group('the mistake replay is percussion end to end', () {
    final drumScore = ReplayScore(
      notes: const [
        TimedNote(
          pitch: 42,
          startMs: 0,
          durationMs: 300,
          clefSign: 'percussion',
          voice: 1,
        ),
        TimedNote(
          pitch: 38,
          startMs: 600,
          durationMs: 300,
          clefSign: 'percussion',
          voice: 1,
        ),
        TimedNote(
          pitch: 36,
          startMs: 1200,
          durationMs: 300,
          clefSign: 'percussion',
          voice: 2,
        ),
      ],
      bpm: 100,
      songEndMs: 2400,
      keyFifths: 0,
      beats: 4,
      beatType: 4,
      measureStartMs: const [0],
      isPercussion: true,
    );

    testWidgets('no stroke is ever flagged for a short sustain', (
      tester,
    ) async {
      // A landed stroke carries no ratio at all, so the keyboard-only sustain
      // category cannot fire on it — the absence decides, not a zero.
      expect(markFor(_stroke(0, TimingVerdict.perfect)), ReplayMark.correct);
      expect(markFor(_stroke(1, TimingVerdict.good)), ReplayMark.correct);
      // …while a keyboard note with a genuinely short hold still is.
      expect(
        markFor(
          const NoteJudgment(
            noteIndex: 0,
            pitch: 60,
            startMs: 0,
            waitMode: false,
            verdict: TimingVerdict.perfect,
            sustainRatio: 0.1,
          ),
        ),
        ReplayMark.shortSustain,
      );
    });

    testWidgets('the run sounds through the percussion channel, and missed '
        'strokes stay silent', (tester) async {
      final audio = RecordingAudioService();
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: ProviderContainer(
            overrides: [audioServiceProvider.overrideWithValue(audio)],
          ),
          child: localizedApp(
            Scaffold(
              body: Builder(
                builder: (context) => ElevatedButton(
                  onPressed: () => showMistakeReplay(
                    context,
                    drumScore,
                    SessionResult.fromJudgments(
                      pieceId: 'drums-ui',
                      title: 'Groove ouvert',
                      hands: 'both',
                      judgments: [
                        _stroke(0, TimingVerdict.perfect, pitch: 42),
                        _stroke(1, TimingVerdict.late, pitch: 38, startMs: 600),
                        _stroke(
                          2,
                          TimingVerdict.missed,
                          pitch: 36,
                          startMs: 1200,
                        ),
                      ],
                      bestCombo: 2,
                      playedAtMs: 0,
                      speed: 1,
                      percussion: true,
                    ),
                  ),
                  child: const Text('go'),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('go'));
      await tester.pumpAndSettle();

      // The engraving is the percussion scrolling staff (the notes carry the
      // percussion clef, which is what the staff painter keys on).
      expect(
        find.byWidgetPredicate(
          (w) => w is CustomPaint && w.painter is StaffPainter,
        ),
        findsWidgets,
      );

      await tester.tap(find.byIcon(Icons.play_circle));
      for (var i = 0; i < 40; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      // Every sounded stroke went through the drum entry points…
      expect(audio.drumOns, isNotEmpty);
      expect(audio.noteOns, isEmpty);
      // …and the missed kick never sounded.
      expect(audio.drumOns.map((e) => e.key), isNot(contains(36)));
      expect(audio.drumOns.map((e) => e.key), containsAll([42, 38]));

      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
    });
  });

  group('a focus-restricted run is scored but not submitted (design D7)', () {
    testWidgets('a full-kit run is submitted as a session', (tester) async {
      final r = await _pumpPlayerWithSync(tester);
      expect(r.container.read(playerProvider).isFocusRestrictedRun, isFalse);
      await _runToEnd(tester, r.container);
      expect(r.calls, ['session']);
      await _teardown(tester, r.container);
    });

    testWidgets('a run with a piece muted is captured as practice instead', (
      tester,
    ) async {
      final r = await _pumpPlayerWithSync(tester);
      r.container.read(playerProvider.notifier).muteDrumPiece(kKickPieceId);
      expect(r.container.read(playerProvider).isFocusRestrictedRun, isTrue);
      await _runToEnd(tester, r.container);
      // Never ranked — a clean groove with a piece muted is not the same
      // achievement, and the boards carry the same piece id either way…
      expect(r.calls, isNot(contains('session')));
      // …but it still holds the streak: an isolation drill is practice.
      expect(r.calls, ['practice']);
      await _teardown(tester, r.container);
    });
  });
}
