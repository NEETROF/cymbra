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
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/daily_session_core.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';
import 'package:music/state/today_ledger.dart';

import '../support/prefs_fakes.dart';
import 'today_ledger_test.mocks.dart';

@GenerateNiceMocks([MockSpec<CourseProgressService>()])
final _today = DateTime(2026, 10, 17, 10, 30);
final _tomorrow = DateTime(2026, 10, 18, 0, 5);
final _yesterday = DateTime(2026, 10, 16, 23, 50);

ProviderContainer _container({
  FakePreferencesService? prefs,
  DateTime Function()? now,
  CourseProgressService? progress,
  bool online = false,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      nowFnProvider.overrideWithValue(now ?? () => _today),
      canUseOnlineServicesProvider.overrideWithValue(online),
      if (progress != null)
        courseProgressServiceProvider.overrideWithValue(progress),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _tick() => Future<void>.delayed(const Duration(milliseconds: 20));

void main() {
  group("today's ledger", () {
    test('a mark lands under today and persists', () async {
      final prefs = FakePreferencesService();
      final c = _container(prefs: prefs);
      await c.read(dailyLedgerProvider.notifier).markLesson('a1');
      final l = c.read(dailyLedgerProvider);
      expect(l.localDay, '2026-10-17');
      expect(l.lessonIds, {'a1'});
      expect(prefs.store['daily.ledger.v1'], contains('"a1"'));
    });

    test("restores today's ledger, including the raised cue", () async {
      final prefs = FakePreferencesService({
        'daily.ledger.v1': jsonEncode({
          'localDay': '2026-10-17',
          'lessonIds': ['a1'],
          'pieceStepKeys': ['a3'],
          'reviewKeys': ['review:a2#1'],
          'cueRaised': true,
        }),
      });
      final c = _container(prefs: prefs);
      c.read(dailyLedgerProvider);
      await _tick();
      final l = c.read(dailyLedgerProvider);
      expect(l.loaded, isTrue);
      expect(l.lessonIds, {'a1'});
      expect(l.pieceStepKeys, {'a3'});
      expect(l.reviewKeys, {'review:a2#1'});
      expect(l.cueRaised, isTrue);
    });

    test("yesterday's ledger is not restored: today starts empty", () async {
      final prefs = FakePreferencesService({
        'daily.ledger.v1': jsonEncode({
          'localDay': '2026-10-16',
          'lessonIds': ['a1'],
          'cueRaised': true,
        }),
      });
      final c = _container(prefs: prefs);
      c.read(dailyLedgerProvider);
      await _tick();
      final l = c.read(dailyLedgerProvider);
      expect(l.loaded, isTrue);
      expect(l.localDay, '2026-10-17');
      expect(l.lessonIds, isEmpty);
      expect(l.cueRaised, isFalse);
    });

    test(
      'rolls over when the local day changes while the app is open',
      () async {
        var now = _today;
        final c = _container(now: () => now);
        final n = c.read(dailyLedgerProvider.notifier);
        await n.markLesson('a1');
        now = _tomorrow;
        await n.markPiece('b2');
        final l = c.read(dailyLedgerProvider);
        expect(l.localDay, '2026-10-18');
        expect(l.lessonIds, isEmpty);
        expect(l.pieceStepKeys, {'b2'});
      },
    );

    test('a clock set back starts a fresh day as well', () async {
      var now = _today;
      final c = _container(now: () => now);
      final n = c.read(dailyLedgerProvider.notifier);
      await n.markLesson('a1');
      now = _yesterday;
      await n.markReview('review:a1#1');
      final l = c.read(dailyLedgerProvider);
      expect(l.localDay, '2026-10-16');
      expect(l.lessonIds, isEmpty);
      expect(l.reviewKeys, {'review:a1#1'});
    });

    test('the cue is raised once a day; a new day forgets it', () async {
      var now = _today;
      final c = _container(now: () => now);
      final n = c.read(dailyLedgerProvider.notifier);
      await n.markCueRaised();
      expect(c.read(dailyLedgerProvider).cueRaised, isTrue);
      now = _tomorrow;
      await n.markLesson('a1');
      expect(c.read(dailyLedgerProvider).cueRaised, isFalse);
    });

    test("the server's today moments count here (another device)", () async {
      final progress = MockCourseProgressService();
      when(progress.progress()).thenAnswer(
        (_) async => [
          CourseProgressRecord(
            courseId: 'c1',
            completed: true,
            passed: false,
            bestSyncPct: null,
            completedAtMs: _today.millisecondsSinceEpoch,
          ),
          CourseProgressRecord(
            courseId: 'c2',
            completed: false,
            passed: true,
            bestSyncPct: 88,
            passedAtMs: _today.millisecondsSinceEpoch,
          ),
          CourseProgressRecord(
            courseId: 'c0',
            completed: true,
            passed: false,
            bestSyncPct: null,
            completedAtMs: _yesterday.millisecondsSinceEpoch,
          ),
        ],
      );
      final c = _container(progress: progress, online: true);
      c.read(dailyLedgerProvider);
      c.read(courseCompletionProvider);
      await _tick();
      await _tick();
      final l = c.read(dailyLedgerProvider);
      expect(l.lessonIds, {'c1'});
      expect(l.pieceStepKeys, {'c2'});
      expect(c.read(courseCompletionProvider).completedAtMs.keys, {'c1', 'c0'});
    });
  });

  group('the daily goal', () {
    test('defaults to normal, persists a change, restores it', () async {
      final prefs = FakePreferencesService();
      final c = _container(prefs: prefs);
      expect(c.read(dailyGoalSettingProvider), DailyGoal.normal);
      await c.read(dailyGoalSettingProvider.notifier).set(DailyGoal.full);
      expect(c.read(dailyGoalSettingProvider), DailyGoal.full);
      expect(prefs.store['daily.goal.v1'], 'full');

      final c2 = _container(
        prefs: FakePreferencesService({'daily.goal.v1': 'light'}),
      );
      c2.read(dailyGoalSettingProvider);
      await _tick();
      expect(c2.read(dailyGoalSettingProvider), DailyGoal.light);
    });
  });

  group('the local day provider', () {
    test('follows the clock when refreshed', () {
      var now = _today;
      final c = _container(now: () => now);
      expect(c.read(localDayProvider), '2026-10-17');
      now = _tomorrow;
      c.read(localDayProvider.notifier).refresh();
      expect(c.read(localDayProvider), '2026-10-18');
    });
  });
}
