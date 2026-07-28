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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/play_outbox_store.dart';
import 'package:music/state/play_session_envelope.dart';

import '../support/prefs_fakes.dart';

PlaySessionEnvelope _entry(String id, {String userId = 'u1'}) =>
    PlaySessionEnvelope(
      sessionId: id,
      userId: userId,
      scoreId: 'piece-$id',
      playedAtMs: 1718494200000,
      tzOffsetMinutes: 60,
      overallSyncPct: 82.5,
      sessionResultJson: '{"pieceId":"piece-$id"}',
    );

void main() {
  test('adds entries and reads them back in order', () async {
    final store = PlayOutboxStore(FakePreferencesService());
    await store.add(_entry('a'));
    await store.add(_entry('b'));
    final all = await store.all();
    expect(all.map((e) => e.sessionId), ['a', 'b']);
    expect(all.first.overallSyncPct, 82.5);
    expect(all.first.tzOffsetMinutes, 60);
  });

  test(
    'add is idempotent by session id (never queues the same session twice)',
    () async {
      final store = PlayOutboxStore(FakePreferencesService());
      await store.add(_entry('a'));
      await store.add(_entry('a'));
      expect(await store.all(), hasLength(1));
    },
  );

  test('remove drops only the acked entry', () async {
    final store = PlayOutboxStore(FakePreferencesService());
    await store.add(_entry('a'));
    await store.add(_entry('b'));
    await store.remove('a');
    expect((await store.all()).map((e) => e.sessionId), ['b']);
    // Removing an absent id is a no-op.
    await store.remove('a');
    expect(await store.all(), hasLength(1));
  });

  test(
    'entries survive an app restart (same backing storage, new store)',
    () async {
      final prefs = FakePreferencesService();
      await PlayOutboxStore(prefs).add(_entry('a'));
      await PlayOutboxStore(prefs).add(_entry('b'));
      // A fresh store over the same persisted storage (models a relaunch).
      final afterRestart = await PlayOutboxStore(prefs).all();
      expect(afterRestart.map((e) => e.sessionId), ['a', 'b']);
    },
  );

  test('unreadable storage reads as empty rather than throwing', () async {
    final prefs = FakePreferencesService({'playOutbox': 'not json'});
    expect(await PlayOutboxStore(prefs).all(), isEmpty);
  });

  test('emptying the outbox clears the backing key', () async {
    final prefs = FakePreferencesService();
    final store = PlayOutboxStore(prefs);
    await store.add(_entry('a'));
    await store.remove('a');
    expect(prefs.store.containsKey('playOutbox'), isFalse);
  });
}
