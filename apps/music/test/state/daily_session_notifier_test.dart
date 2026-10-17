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
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/daily_session_core.dart';
import 'package:music/state/daily_session_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/today_ledger.dart';

import '../support/daily_fixtures.dart';
import '../support/prefs_fakes.dart';

final _today = DateTime(2026, 10, 17, 9);

ProviderContainer _container({
  Map<String, String> prefs = const {},
  DateTime Function()? now,
  CourseCatalogService? catalog,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      nowFnProvider.overrideWithValue(now ?? () => _today),
      courseCatalogServiceProvider.overrideWithValue(
        catalog ?? const DailyFakeCatalog(),
      ),
    ],
  );
  addTearDown(c.dispose);
  // An auto-dispose provider needs a listener to stay alive between reads.
  final sub = c.listen(dailySessionProvider, (_, _) {});
  addTearDown(sub.close);
  return c;
}

DailyItem? _item(DailySession s, DailyItemKind kind) =>
    s.items.where((i) => i.kind == kind).firstOrNull;

final _midPathPrefs = {
  'courses.completed.v1': jsonEncode(['a1', 'a2', 'b1']),
  'courses.passed.v1': jsonEncode(['a3']),
  'courses.best.v1': jsonEncode({'a3': 91}),
};

void main() {
  group('the daily session provider', () {
    test(
      'composes from the catalogue, the completion state and the ledger',
      () async {
        final c = _container(prefs: _midPathPrefs);
        final s = await c.read(dailySessionProvider.future);
        expect(_item(s, DailyItemKind.lesson)?.courseId, 'b2');
        expect(_item(s, DailyItemKind.piece)?.piece, isNotNull);
        expect(_item(s, DailyItemKind.review)?.block, isNotNull);
        expect(s.doneCount, 0);
      },
    );

    test('a ledger mark recomposes with the item done', () async {
      final c = _container(prefs: _midPathPrefs);
      await c.read(dailySessionProvider.future);
      await c.read(dailyLedgerProvider.notifier).markLesson('b2');
      await Future<void>.delayed(Duration.zero);
      final s = await c.read(dailySessionProvider.future);
      expect(_item(s, DailyItemKind.lesson)?.done, isTrue);
      expect(s.doneCount, 1);
    });

    test('midnight resets the items and recomposes', () async {
      var now = _today;
      final c = _container(prefs: _midPathPrefs, now: () => now);
      await c.read(dailySessionProvider.future);
      await c.read(dailyLedgerProvider.notifier).markLesson('b2');
      await Future<void>.delayed(Duration.zero);
      expect((await c.read(dailySessionProvider.future)).doneCount, 1);

      now = DateTime(2026, 10, 18, 0, 1);
      c.read(localDayProvider.notifier).refresh();
      await Future<void>.delayed(Duration.zero);
      final s = await c.read(dailySessionProvider.future);
      expect(s.doneCount, 0);
      expect(_item(s, DailyItemKind.lesson)?.courseId, 'b2');
      expect(_item(s, DailyItemKind.lesson)?.done, isFalse);
    });

    test('no courses → an empty session', () async {
      final c = _container(catalog: const DailyFakeCatalog([]));
      final s = await c.read(dailySessionProvider.future);
      expect(s.items, isEmpty);
    });
  });
}
