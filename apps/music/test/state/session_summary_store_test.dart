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
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/state/session_summary_store.dart';

import '../support/prefs_fakes.dart';

SessionResult _result() => SessionResult.fromJudgments(
  pieceId: 'p1',
  title: 'Clair de Lune',
  hands: 'both',
  judgments: const [
    NoteJudgment(
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
  playedAtMs: 42,
  speed: 1,
);

void main() {
  late FakePreferencesService prefs;
  late SessionSummaryStore store;

  setUp(() {
    prefs = FakePreferencesService();
    final container = ProviderContainer(
      overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
    );
    addTearDown(container.dispose);
    store = container.read(sessionSummaryStoreProvider);
  });

  test('load returns null when nothing was ever saved', () async {
    expect(await store.load(), isNull);
  });

  test(
    'save then load round-trips the result through the fake prefs',
    () async {
      final r = _result();
      await store.save(r);
      // Written through the injectable seam, not native storage.
      expect(prefs.store.containsKey('lastSessionResult'), isTrue);
      final loaded = await store.load();
      expect(loaded, isNotNull);
      expect(loaded!.title, 'Clair de Lune');
      expect(loaded.overallSyncPct, closeTo(r.overallSyncPct, 1e-9));
      expect(loaded.notes.single.verdict, TimingVerdict.perfect);
    },
  );

  test(
    'an unreadable stored value loads as null rather than throwing',
    () async {
      prefs.store['lastSessionResult'] = 'not json';
      expect(await store.load(), isNull);
    },
  );
}
