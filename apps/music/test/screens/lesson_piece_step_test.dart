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
import 'package:music/courses/course_manifest.dart';
import 'package:music/screens/lesson_player_screen.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/state/performance_scoring_core.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

class _FakeService implements CourseCatalogService {
  _FakeService(this.manifest);
  final String manifest;
  @override
  Future<List<CourseListing>> listCourses() async => const [];
  @override
  Future<String?> getCourseManifestJson(String id) async => manifest;
}

/// A lesson with one piece step on the bundled Ode to Joy, right hand, 60 %.
String _manifest({String bundled = 'ode-to-joy'}) =>
    '''
{
  "schemaVersion": 2, "id": "c1",
  "title": {"en": "Fa et sol"},
  "blocks": [
    {"type": "text", "text": {"en": "Your turn."}},
    {"type": "piece", "score": {"bundled": "$bundled"}, "hands": "right",
     "speed": 0.6, "waitMode": true, "passPct": 80,
     "title": {"en": "Ode — right hand"}},
    {"type": "text", "text": {"en": "Well done."}}
  ]
}
''';

/// A lesson chaining two piece steps on the same tune (steps 1 and 3).
const String _chain = '''
{
  "schemaVersion": 2, "id": "c1",
  "title": {"en": "Fa et sol"},
  "blocks": [
    {"type": "text", "text": {"en": "Your turn."}},
    {"type": "piece", "score": {"bundled": "ode-to-joy"}, "hands": "right",
     "speed": 0.6, "waitMode": true, "passPct": 80,
     "title": {"en": "Ode — slowly"}},
    {"type": "text", "text": {"en": "Now in rhythm."}},
    {"type": "piece", "score": {"bundled": "ode-to-joy"}, "hands": "right",
     "speed": 1.0, "waitMode": false, "passPct": 80,
     "title": {"en": "Ode — in rhythm"}},
    {"type": "text", "text": {"en": "Well done."}}
  ]
}
''';

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  String? bundled,
  String? manifest,
  FakePreferencesService? prefs,
}) async {
  final midi = FakeMidiService();
  addTearDown(midi.close);
  final container = ProviderContainer(
    overrides: [
      courseCatalogServiceProvider.overrideWithValue(
        _FakeService(manifest ?? _manifest(bundled: bundled ?? 'ode-to-joy')),
      ),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      midiServiceProvider.overrideWithValue(midi),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
    ],
  );
  addTearDown(container.dispose);
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        const LessonPlayerScreen(courseId: 'c1'),
        locale: const Locale('en'),
      ),
    ),
  );
  await tester.pumpAndSettle();
  // Past the hook, onto the piece step.
  await tester.tap(find.byKey(const Key('lesson-next')));
  await tester.pumpAndSettle();
  return container;
}

/// Unmounts the tree before the container is disposed, so the player's and the
/// setup modal's timers are cancelled before the test ends.
Future<void> _teardown(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

/// Bounded frames: the player runs a Ticker, so pumpAndSettle never settles.
Future<void> _frames(WidgetTester tester, [int n = 12]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

SessionResult _result(double pct) => SessionResult(
  pieceId: 'ode-to-joy',
  title: 'Ode to Joy',
  hands: 'right',
  overallSyncPct: pct,
  runMode: RunMode.wait,
  freeSyncPct: null,
  waitSyncPct: pct,
  freeOnsetCount: 0,
  waitOnsetCount: 10,
  avgFreeOffsetMs: null,
  avgReactionMs: 100,
  timing: pct,
  correctness: pct,
  sustain: null,
  verdictCounts: const {},
  wrongNotes: 0,
  bestCombo: 3,
  notes: const [],
  playedAtMs: 0,
  speed: 0.6,
  inputSource: 'midi',
);

void main() {
  // change: add-music-piece-lessons
  testWidgets('the piece step shows the imposed settings and the bar, and '
      'gates Next until a run', (tester) async {
    await _pump(tester);
    expect(find.text('Ode — right hand'), findsOneWidget);
    expect(find.text('Right hand'), findsOneWidget);
    expect(find.text('Speed 60 %'), findsOneWidget);
    expect(find.text('Wait Mode'), findsOneWidget);
    expect(find.text('80 % overall sync to pass this step'), findsOneWidget);
    expect(find.text('Not played yet'), findsOneWidget);
    expect(find.byKey(const Key('piece-start')), findsOneWidget);
    expect(find.byKey(const Key('lesson-next')), findsNothing);
    await _teardown(tester);
  });

  testWidgets('Play opens the real player with the lesson in charge: no '
      'setup modal, the lesson in the header, settings seeded; the result '
      'comes back as passed and opens the gate', (tester) async {
    final container = await _pump(tester);
    await tester.tap(find.byKey(const Key('piece-start')));
    await _frames(tester, 30);

    expect(find.byType(PlayerScreen), findsOneWidget);
    // No pre-play setup: the lesson imposed everything.
    expect(find.widgetWithText(FilledButton, 'Play'), findsNothing);
    expect(
      find.text('Lesson · goal 80 % · Fa et sol'),
      findsOneWidget,
      reason: 'the header names the lesson and its bar',
    );
    final data = container.read(playerProvider);
    expect(data.lessonContext, isNotNull);
    expect(data.speed, 0.6);
    expect(data.waitMode, isTrue);

    // A finished run reaches the step through the context, as the player's
    // end-of-run path does; here delivered directly.
    data.lessonContext!.onResult(_result(84));
    await _frames(tester);
    // Leave the player: back on the lesson, the step is passed and Next is back.
    Navigator.of(tester.element(find.byType(PlayerScreen))).pop();
    await _frames(tester, 20);
    expect(find.byType(LessonPlayerScreen), findsOneWidget);
    // The pass is celebrated back on the lesson, never over the player
    // (change: add-music-motion-and-celebrations).
    expect(find.byKey(const Key('reward-celebration')), findsOneWidget);
    expect(find.text('Piece passed!'), findsOneWidget);
    expect(find.text('Ode — right hand · 84 %'), findsOneWidget);
    await _frames(tester, 24);
    await tester.tap(find.text('Nice!'));
    await _frames(tester, 10);
    expect(find.byKey(const Key('reward-celebration')), findsNothing);
    expect(find.text('Step passed · 84 %'), findsOneWidget);
    expect(find.byKey(const Key('piece-retry')), findsOneWidget);
    expect(find.byKey(const Key('lesson-next')), findsOneWidget);
    // Recorded against the course: passed, best 84.
    final completion = container.read(courseCompletionProvider);
    expect(completion.isPassed('c1'), isTrue);
    expect(completion.bestPct['c1'], 84);
    // The player's own preferences were never written by the lesson run.
    expect(
      (prefsOf(container)).store.containsKey('player_preferences'),
      isFalse,
    );
    await _teardown(tester);
  });

  testWidgets('the play settings opened from the gear show the imposed '
      'settings locked by the lesson', (tester) async {
    await _pump(tester);
    await tester.tap(find.byKey(const Key('piece-start')));
    await _frames(tester, 30);
    await tester.tap(find.byTooltip('Settings'));
    await _frames(tester, 10);
    expect(find.byKey(const Key('pre-play-lesson-lock')), findsOneWidget);
    expect(find.text('Set by the lesson “Fa et sol”'), findsOneWidget);
    // Close the modal, then leave the player so its auto-dispose state (and
    // the MIDI poll it runs) is gone before the tree is torn down.
    await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
    await _frames(tester, 10);
    Navigator.of(tester.element(find.byType(PlayerScreen))).pop();
    await _frames(tester, 20);
    await _teardown(tester);
  });

  testWidgets('a run below the bar leaves the step visited, with Try again '
      'and Next both offered', (tester) async {
    final container = await _pump(tester);
    await tester.tap(find.byKey(const Key('piece-start')));
    await _frames(tester, 30);
    container.read(playerProvider).lessonContext!.onResult(_result(71));
    await _frames(tester);
    Navigator.of(tester.element(find.byType(PlayerScreen))).pop();
    await _frames(tester, 20);
    expect(find.text('Visited · best 71 %'), findsOneWidget);
    expect(find.byKey(const Key('piece-retry')), findsOneWidget);
    expect(find.byKey(const Key('lesson-next')), findsOneWidget);
    expect(container.read(courseCompletionProvider).isPassed('c1'), isFalse);
    // A run below the bar is no celebration.
    expect(find.byKey(const Key('reward-celebration')), findsNothing);
    await _teardown(tester);
  });

  testWidgets('an unknown piece never traps: the step says so and Next '
      'returns', (tester) async {
    await _pump(tester, bundled: 'nope');
    await tester.tap(find.byKey(const Key('piece-start')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('piece-unavailable')), findsOneWidget);
    expect(find.text('This piece is not available.'), findsOneWidget);
    expect(find.byKey(const Key('lesson-next')), findsOneWidget);
    expect(find.byType(PlayerScreen), findsNothing);
    await _teardown(tester);
  });

  testWidgets('a learner coming back sees the course\'s standing before '
      'pressing Play', (tester) async {
    await _pump(
      tester,
      prefs: FakePreferencesService({
        'courses.passed.v1': '["c1"]',
        'courses.best.v1': '{"c1": 88}',
      }),
    );
    expect(find.text('Step passed · 88 %'), findsOneWidget);
    expect(find.byKey(const Key('piece-retry')), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('in a chain, each piece step keeps its own standing: the '
      'second step is not passed by the first (found while dogfooding)', (
    tester,
  ) async {
    await _pump(
      tester,
      manifest: _chain,
      prefs: FakePreferencesService({
        'courses.passed.v1': jsonEncode(['c1']),
        'courses.best.v1': jsonEncode({'c1': 99}),
        'courses.stepbest.v1': jsonEncode({'c1#1': 99}),
      }),
    );
    // Step 1 shows its own pass.
    expect(find.text('Step passed · 99 %'), findsOneWidget);
    expect(find.byKey(const Key('piece-retry')), findsOneWidget);
    // Past it (the late skip: no run this session), then the bridging text.
    await tester.pump(const Duration(seconds: 13));
    await tester.tap(find.byKey(const Key('lesson-skip')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('lesson-next')));
    await tester.pumpAndSettle();
    // Step 3 has never been run: not passed, Play, not Try again.
    expect(find.text('Ode — in rhythm'), findsOneWidget);
    expect(find.text('Step passed · 99 %'), findsNothing);
    expect(find.byKey(const Key('piece-start')), findsOneWidget);
    expect(find.byKey(const Key('piece-retry')), findsNothing);
    await _teardown(tester);
  });
}

FakePreferencesService prefsOf(ProviderContainer c) =>
    c.read(preferencesServiceProvider) as FakePreferencesService;
