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
import 'package:music/services/preferences_service.dart';
import 'package:music/state/course_completion_notifier.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container(FakePreferencesService prefs) {
  final c = ProviderContainer(
    overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
  );
  final sub = c.listen(courseCompletionProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

Future<void> _settle() => Future<void>.delayed(Duration.zero);

void main() {
  test('restores the completed set from storage', () async {
    final prefs = FakePreferencesService({'courses.completed.v1': '["a","b"]'});
    final c = _container(prefs);
    await _settle();
    final s = c.read(courseCompletionProvider);
    expect(s.loaded, isTrue);
    expect(s.isCompleted('a'), isTrue);
    expect(s.isCompleted('b'), isTrue);
    expect(s.isCompleted('c'), isFalse);
  });

  test('markCompleted persists and is idempotent', () async {
    final prefs = FakePreferencesService();
    final c = _container(prefs);
    await _settle();
    await c.read(courseCompletionProvider.notifier).markCompleted('x');
    expect(c.read(courseCompletionProvider).isCompleted('x'), isTrue);
    expect(prefs.store['courses.completed.v1'], contains('x'));

    // Marking again is a no-op (no churn, still completed).
    await c.read(courseCompletionProvider.notifier).markCompleted('x');
    expect(c.read(courseCompletionProvider).completed, {'x'});
  });

  test('a completion recorded before restore is not clobbered', () async {
    // Storage already has "old"; we mark "new" before the restore resolves.
    final prefs = FakePreferencesService({'courses.completed.v1': '["old"]'});
    final c = _container(prefs);
    // Do not settle first: mark immediately so it races the async restore.
    await c.read(courseCompletionProvider.notifier).markCompleted('new');
    await _settle();
    final s = c.read(courseCompletionProvider);
    expect(s.isCompleted('new'), isTrue);
    expect(s.isCompleted('old'), isTrue);
  });
}
