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
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';

/// The server's view: completion ids, and per-course piece outcome.
class _FakeProgress implements CourseProgressService {
  _FakeProgress({
    Set<String> completed = const {},
    Map<String, (bool, double)> pieces = const {},
  }) : serverIds = {...completed},
       pieces = {...pieces};
  Set<String> serverIds;
  final Map<String, (bool, double)> pieces;
  final List<(String, bool, double)> pieceRecorded = [];
  final List<String> recorded = [];

  @override
  Future<Set<String>> completedCourseIds() async => serverIds;
  @override
  Future<void> recordCompletion(String courseId) async {
    recorded.add(courseId);
    serverIds = {...serverIds, courseId};
  }

  @override
  Future<List<CourseProgressRecord>> progress() async => [
    for (final id in {...serverIds, ...pieces.keys})
      CourseProgressRecord(
        courseId: id,
        completed: serverIds.contains(id),
        passed: pieces[id]?.$1 ?? false,
        bestSyncPct: pieces[id]?.$2,
      ),
  ];

  @override
  Future<void> recordPieceResult(
    String courseId, {
    required bool passed,
    required double syncPct,
  }) async {
    pieceRecorded.add((courseId, passed, syncPct));
    final prev = pieces[courseId];
    pieces[courseId] = (
      (prev?.$1 ?? false) || passed,
      prev == null || syncPct > prev.$2 ? syncPct : prev.$2,
    );
  }
}

ProviderContainer _container({
  required bool online,
  FakePreferencesService? prefs,
  _FakeProgress? progress,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      courseProgressServiceProvider.overrideWithValue(
        progress ?? _FakeProgress(),
      ),
      canUseOnlineServicesProvider.overrideWithValue(online),
    ],
  );
  final sub = c.listen(courseCompletionProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

Future<void> _settle() =>
    Future<void>.delayed(const Duration(milliseconds: 10));

void main() {
  // change: add-music-piece-lessons
  test('a run below the bar is visited once the course is completed; a pass '
      'is kept and the best only rises', () async {
    final prefs = FakePreferencesService();
    final c = _container(online: false, prefs: prefs);
    await _settle();
    final n = c.read(courseCompletionProvider.notifier);

    await n.recordPieceResult('c1', passed: false, syncPct: 71);
    var s = c.read(courseCompletionProvider);
    expect(s.isPassed('c1'), isFalse);
    expect(s.bestPct['c1'], 71);
    expect(s.isVisited('c1'), isFalse, reason: 'not completed yet');
    expect(s.isDone('c1'), isFalse);

    await n.markCompleted('c1');
    s = c.read(courseCompletionProvider);
    expect(s.isVisited('c1'), isTrue);
    expect(s.isDone('c1'), isTrue);

    await n.recordPieceResult('c1', passed: true, syncPct: 84);
    await n.recordPieceResult('c1', passed: false, syncPct: 60);
    s = c.read(courseCompletionProvider);
    expect(s.isPassed('c1'), isTrue);
    expect(s.isVisited('c1'), isFalse);
    expect(s.bestPct['c1'], 84, reason: 'a lower run never lowers the best');
    expect(prefs.store['courses.passed.v1'], contains('c1'));
    expect(prefs.store['courses.best.v1'], contains('84'));
  });

  test('a pass without completion counts as done for the path', () async {
    final c = _container(online: false);
    await _settle();
    await c
        .read(courseCompletionProvider.notifier)
        .recordPieceResult('c2', passed: true, syncPct: 90);
    final s = c.read(courseCompletionProvider);
    expect(s.isCompleted('c2'), isFalse);
    expect(s.isDone('c2'), isTrue);
  });

  test('restores passed and best from storage', () async {
    final c = _container(
      online: false,
      prefs: FakePreferencesService({
        'courses.passed.v1': '["a"]',
        'courses.best.v1': '{"a": 88.5, "b": 40}',
      }),
    );
    await _settle();
    final s = c.read(courseCompletionProvider);
    expect(s.isPassed('a'), isTrue);
    expect(s.bestPct, {'a': 88.5, 'b': 40});
  });

  test('signed in, the server\'s passed and best merge in (greatest wins) and '
      'local-only results are pushed up', () async {
    final progress = _FakeProgress(pieces: {'a': (true, 70), 'b': (false, 95)});
    final c = _container(
      online: true,
      progress: progress,
      prefs: FakePreferencesService({
        'courses.passed.v1': '["c"]',
        'courses.best.v1': '{"a": 82, "b": 50, "c": 85}',
      }),
    );
    await _settle();
    final s = c.read(courseCompletionProvider);
    expect(s.isPassed('a'), isTrue);
    expect(s.isPassed('b'), isFalse);
    expect(s.isPassed('c'), isTrue);
    expect(s.bestPct['a'], 82, reason: 'local 82 beats server 70');
    expect(s.bestPct['b'], 95, reason: 'server 95 beats local 50');
    // Pushed: a (better best), c (server lacked it); b was not (server is ahead).
    final pushed = progress.pieceRecorded.map((r) => r.$1).toSet();
    expect(pushed, {'a', 'c'});
    expect(progress.pieces['c'], (true, 85.0));
  });

  test('signed in, a recorded run reaches the server at once', () async {
    final progress = _FakeProgress();
    final c = _container(online: true, progress: progress);
    await _settle();
    await c
        .read(courseCompletionProvider.notifier)
        .recordPieceResult('d', passed: true, syncPct: 81);
    expect(progress.pieceRecorded, [('d', true, 81.0)]);
  });
}
