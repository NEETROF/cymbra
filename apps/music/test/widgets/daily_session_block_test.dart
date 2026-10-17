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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/screens/lesson_player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/streak_service.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/daily_session_core.dart';
import 'package:music/state/daily_session_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';
import 'package:music/state/today_ledger.dart';
import 'package:music/widgets/daily_session_block.dart';

import '../support/daily_fixtures.dart';
import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import 'daily_session_block_test.mocks.dart';

@GenerateNiceMocks([MockSpec<StreakService>()])
final _today = DateTime(2026, 10, 17, 9);

final _midPathPrefs = {
  'courses.completed.v1': jsonEncode(['a1', 'a2', 'b1']),
  'courses.passed.v1': jsonEncode(['a3']),
  'courses.best.v1': jsonEncode({'a3': 91}),
};

ProviderContainer _container({
  Map<String, String> prefs = const {},
  CourseCatalogService? catalog,
  bool online = false,
  StreakView? streak,
}) {
  final midi = FakeMidiService();
  addTearDown(midi.close);
  final streakService = MockStreakService();
  when(streakService.getStreak()).thenAnswer(
    (_) async =>
        streak ??
        const StreakView(
          current: 0,
          longest: 0,
          playedToday: false,
          recoverable: false,
          recoverCost: 0,
          recoverableStreak: 0,
        ),
  );
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      nowFnProvider.overrideWithValue(() => _today),
      courseCatalogServiceProvider.overrideWithValue(
        catalog ?? const DailyFakeCatalog(),
      ),
      midiServiceProvider.overrideWithValue(midi),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      canUseOnlineServicesProvider.overrideWithValue(online),
      streakServiceProvider.overrideWithValue(streakService),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer c) async {
  tester.view.physicalSize = const Size(1000, 900);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        const Scaffold(body: SingleChildScrollView(child: DailySessionBlock())),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

String _ring(WidgetTester tester) => tester
    .widget<Text>(
      find.descendant(
        of: find.byKey(const Key('daily-ring')),
        matching: find.textContaining('/'),
      ),
    )
    .data!;

void main() {
  group('daily session block', () {
    testWidgets('hidden when there are no courses', (tester) async {
      await _pump(tester, _container(catalog: const DailyFakeCatalog([])));
      expect(find.byKey(const Key('daily-session-block')), findsNothing);
    });

    testWidgets('mid-path: lesson, piece, review, the ring over the goal, '
        'the goal choice — and no streak for a guest', (tester) async {
      await _pump(tester, _container(prefs: _midPathPrefs));
      expect(find.byKey(const Key('daily-session-block')), findsOneWidget);
      expect(find.text("TODAY'S SESSION"), findsOneWidget);
      expect(find.text('About 10 minutes'), findsOneWidget);
      expect(find.byKey(const Key('daily-item-lesson')), findsOneWidget);
      expect(find.byKey(const Key('daily-item-piece')), findsOneWidget);
      expect(find.byKey(const Key('daily-item-review')), findsOneWidget);
      expect(find.text('Lesson b2'), findsWidgets);
      expect(find.text('Piece of b2'), findsOneWidget);
      expect(_ring(tester), '0/2');
      expect(find.text('today'), findsOneWidget);
      expect(find.byKey(const Key('daily-goal')), findsOneWidget);
      expect(find.byKey(const Key('daily-streak')), findsNothing);
      expect(find.byKey(const Key('daily-done-lesson')), findsNothing);
    });

    testWidgets('the goal choice resizes the ring and is remembered', (
      tester,
    ) async {
      final c = _container(prefs: _midPathPrefs);
      await _pump(tester, c);
      await tester.tap(find.byKey(const Key('daily-goal-light')));
      await tester.pumpAndSettle();
      expect(_ring(tester), '0/1');
      expect(c.read(dailyGoalSettingProvider), DailyGoal.light);
      await tester.tap(find.byKey(const Key('daily-goal-full')));
      await tester.pumpAndSettle();
      expect(_ring(tester), '0/3');
    });

    testWidgets('signed in, the streak is shown as the server reports it', (
      tester,
    ) async {
      await _pump(
        tester,
        _container(
          prefs: _midPathPrefs,
          online: true,
          streak: const StreakView(
            current: 7,
            longest: 9,
            playedToday: false,
            recoverable: false,
            recoverCost: 0,
            recoverableStreak: 0,
          ),
        ),
      );
      expect(find.byKey(const Key('daily-streak')), findsOneWidget);
      expect(find.textContaining('7-day streak'), findsOneWidget);
      expect(find.textContaining('not yet today'), findsOneWidget);
    });

    testWidgets('a zero streak invites rather than counts', (tester) async {
      await _pump(tester, _container(prefs: _midPathPrefs, online: true));
      expect(find.textContaining('No streak yet'), findsOneWidget);
    });

    testWidgets('an item done today reads done and the ring counts up', (
      tester,
    ) async {
      final c = _container(prefs: _midPathPrefs);
      await _pump(tester, c);
      await c.read(dailyLedgerProvider.notifier).markLesson('b2');
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('daily-done-lesson')), findsOneWidget);
      expect(find.text('done'), findsOneWidget);
      expect(_ring(tester), '1/2');
    });

    testWidgets('the lesson item opens its lesson; finishing it marks the '
        'item', (tester) async {
      // Start of the path: the lesson is a1, a text block then a drill — the
      // drill is skipped through the late escape hatch.
      final c = _container();
      await _pump(tester, c);
      await tester.tap(find.byKey(const Key('daily-item-lesson')));
      await tester.pumpAndSettle();
      expect(find.byType(LessonPlayerScreen), findsOneWidget);
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      await tester.pump(const Duration(seconds: 13));
      await tester.tap(find.byKey(const Key('lesson-skip')));
      await tester.pumpAndSettle();
      // The lesson celebration, then back on the door.
      await tester.tap(find.byKey(const Key('lesson-celebration-close')));
      await tester.pumpAndSettle();
      expect(c.read(dailyLedgerProvider).lessonIds, {'a1'});
      expect(c.read(courseCompletionProvider).isCompleted('a1'), isTrue);
      expect(find.byKey(const Key('daily-done-lesson')), findsOneWidget);
    });

    testWidgets('the review item opens a one-block mini-lesson titled after '
        'its lesson', (tester) async {
      // Only b1 is done: the review is b1's ear drill.
      final c = _container(
        prefs: {
          'courses.completed.v1': jsonEncode(['b1']),
        },
      );
      await _pump(tester, c);
      final session = c.read(dailySessionProvider).requireValue;
      final review = session.items.firstWhere(
        (i) => i.kind == DailyItemKind.review,
      );
      expect(review.courseId, 'b1');
      expect(find.text('Higher or lower?'), findsOneWidget);
      await tester.tap(find.byKey(const Key('daily-item-review')));
      await tester.pumpAndSettle();
      final screen = tester.widget<LessonPlayerScreen>(
        find.byType(LessonPlayerScreen),
      );
      expect(screen.review?.key, review.key);
      expect(screen.review?.manifest.blocks, hasLength(1));
      expect(find.text('Lesson b1'), findsWidgets);
    });

    testWidgets('finishing a review writes the ledger, completes no course '
        'and celebrates nothing', (tester) async {
      final c = _container();
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: c,
          child: localizedApp(
            LessonPlayerScreen(
              courseId: 'review:a1#1',
              review: ReviewRun(
                key: 'review:a1#1',
                manifest: CourseManifest(
                  schemaVersion: kCourseSchemaVersion,
                  id: 'review:a1#1',
                  title: const {'en': 'Lesson a1'},
                  blocks: const [
                    CourseBlock.text(text: {'en': 'A quick one.'}),
                  ],
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('A quick one.'), findsOneWidget);
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      expect(find.byType(LessonPlayerScreen), findsNothing, reason: 'popped');
      expect(find.byKey(const Key('lesson-celebration-close')), findsNothing);
      expect(c.read(dailyLedgerProvider).reviewKeys, {'review:a1#1'});
      expect(c.read(courseCompletionProvider).completed, isEmpty);
    });
  });
}
