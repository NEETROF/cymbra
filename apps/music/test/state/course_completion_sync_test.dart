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

class _FakeProgress implements CourseProgressService {
  _FakeProgress({Set<String> server = const {}}) : serverIds = {...server};
  Set<String> serverIds;
  final List<String> recorded = [];
  @override
  Future<Set<String>> completedCourseIds() async => serverIds;
  @override
  Future<void> recordCompletion(String courseId) async {
    recorded.add(courseId);
    serverIds = {...serverIds, courseId};
  }
}

ProviderContainer _container({
  required bool online,
  required FakePreferencesService prefs,
  required _FakeProgress progress,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      courseProgressServiceProvider.overrideWithValue(progress),
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
  test('signed in, server completions merge into local state', () async {
    final c = _container(
      online: true,
      prefs: FakePreferencesService(),
      progress: _FakeProgress(server: {'x'}),
    );
    await _settle();
    expect(c.read(courseCompletionProvider).isCompleted('x'), isTrue);
  });

  test('a guest stays local (no server call)', () async {
    final progress = _FakeProgress();
    final c = _container(
      online: false,
      prefs: FakePreferencesService(),
      progress: progress,
    );
    await _settle();
    await c.read(courseCompletionProvider.notifier).markCompleted('z');
    expect(c.read(courseCompletionProvider).isCompleted('z'), isTrue);
    expect(progress.recorded, isEmpty); // never hit the server
  });

  test('signed in, completing a course records it on the server', () async {
    final progress = _FakeProgress();
    final c = _container(
      online: true,
      prefs: FakePreferencesService(),
      progress: progress,
    );
    await _settle();
    await c.read(courseCompletionProvider.notifier).markCompleted('a');
    expect(progress.recorded, contains('a'));
  });

  test(
    'local guest completions are pushed to the server after sign-in',
    () async {
      // The device already has a local completion; the server has none.
      final progress = _FakeProgress();
      final c = _container(
        online: true,
        prefs: FakePreferencesService({'courses.completed.v1': '["y"]'}),
        progress: progress,
      );
      await _settle();
      // The sync pushed the local-only completion up.
      expect(progress.recorded, contains('y'));
      expect(c.read(courseCompletionProvider).isCompleted('y'), isTrue);
    },
  );
}
