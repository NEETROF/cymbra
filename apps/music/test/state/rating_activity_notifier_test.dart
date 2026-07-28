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
import 'package:music/state/rating_activity_notifier.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';

final _now = DateTime(2026, 7, 28, 12);

ProviderContainer _make({
  required FakePreferencesService prefs,
  DateTime? now,
  bool signedIn = true,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      canUseOnlineServicesProvider.overrideWithValue(signedIn),
      nowFnProvider.overrideWithValue(() => now ?? _now),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

String _millis(DateTime d) => d.millisecondsSinceEpoch.toString();

void main() {
  group('shouldInviteToRate', () {
    test('never rated → invited', () {
      expect(shouldInviteToRate(null, _now), isTrue);
    });
    test('rated long ago → invited', () {
      expect(
        shouldInviteToRate(_now.subtract(const Duration(days: 5)), _now),
        isTrue,
      );
    });
    test('rated recently → not invited', () {
      expect(
        shouldInviteToRate(_now.subtract(const Duration(days: 1)), _now),
        isFalse,
      );
    });
    test('exactly at the threshold → invited (inclusive)', () {
      expect(
        shouldInviteToRate(_now.subtract(RatingActivity.inviteAfter), _now),
        isTrue,
      );
    });
  });

  test('signed-out never shows the invite', () async {
    final c = _make(prefs: FakePreferencesService(), signedIn: false);
    await c.read(ratingActivityProvider.future);
    expect(c.read(ratingInviteVisibleProvider), isFalse);
  });

  test('signed-in first-run (never rated) shows the invite', () async {
    final c = _make(prefs: FakePreferencesService());
    await c.read(ratingActivityProvider.future);
    expect(c.read(ratingInviteVisibleProvider), isTrue);
  });

  test('a recent stored rating suppresses the invite', () async {
    final prefs = FakePreferencesService({
      RatingActivity.prefsKey: _millis(_now.subtract(const Duration(hours: 6))),
    });
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    expect(c.read(ratingInviteVisibleProvider), isFalse);
  });

  test('a stale stored rating re-shows the invite', () async {
    final prefs = FakePreferencesService({
      RatingActivity.prefsKey: _millis(_now.subtract(const Duration(days: 10))),
    });
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    expect(c.read(ratingInviteVisibleProvider), isTrue);
  });

  test('markRatedNow hides the invite and persists the timestamp', () async {
    final prefs = FakePreferencesService();
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    expect(c.read(ratingInviteVisibleProvider), isTrue); // never rated yet

    await c.read(ratingActivityProvider.notifier).markRatedNow();
    expect(c.read(ratingInviteVisibleProvider), isFalse);
    expect(prefs.store[RatingActivity.prefsKey], _millis(_now));
  });

  test('snooze suppresses the invite for the snooze window', () async {
    final prefs = FakePreferencesService();
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    await c.read(ratingActivityProvider.notifier).snooze();
    expect(c.read(ratingInviteVisibleProvider), isFalse);
    expect(prefs.store[RatingActivity.prefsKey], _millis(_now));
  });
}
