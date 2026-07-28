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
import 'package:music/services/catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/rating_activity_notifier.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';
import '../support/rating_fakes.dart';

final _now = DateTime(2026, 7, 28, 12);

ProviderContainer _make({
  required FakePreferencesService prefs,
  DateTime? now,
  bool signedIn = true,
  int scoresToRate = 3,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      canUseOnlineServicesProvider.overrideWithValue(signedIn),
      nowFnProvider.overrideWithValue(() => now ?? _now),
      // The invite also probes the deck source; default to some un-rated scores.
      catalogServiceProvider.overrideWithValue(
        FakeDeckCatalogService(deckCorpus(scoresToRate)),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

/// Reads the (now async) invite visibility.
Future<bool> _visible(ProviderContainer c) =>
    c.read(ratingInviteVisibleProvider.future);

String _millis(DateTime d) => d.millisecondsSinceEpoch.toString();

void main() {
  RatingActivityData at(DateTime? lastAt, {int dismissals = 0}) =>
      RatingActivityData(lastAt: lastAt, dismissals: dismissals);

  group('shouldInviteToRate', () {
    test('never rated → invited', () {
      expect(shouldInviteToRate(at(null), _now), isTrue);
    });
    test('rated long ago → invited', () {
      expect(
        shouldInviteToRate(at(_now.subtract(const Duration(days: 5))), _now),
        isTrue,
      );
    });
    test('rated recently → not invited', () {
      expect(
        shouldInviteToRate(at(_now.subtract(const Duration(days: 1))), _now),
        isFalse,
      );
    });
    test('exactly at the threshold → invited (inclusive)', () {
      expect(
        shouldInviteToRate(at(_now.subtract(RatingActivity.inviteAfter)), _now),
        isTrue,
      );
    });
    test('stops for good after enough dismissals', () {
      // Even long-stale, once dismissed the max times it never shows again.
      final old = _now.subtract(const Duration(days: 30));
      expect(
        shouldInviteToRate(
          at(old, dismissals: RatingActivity.maxDismissals),
          _now,
        ),
        isFalse,
      );
      // One below the threshold still shows when stale.
      expect(
        shouldInviteToRate(
          at(old, dismissals: RatingActivity.maxDismissals - 1),
          _now,
        ),
        isTrue,
      );
    });
  });

  test('signed-out never shows the invite', () async {
    final c = _make(prefs: FakePreferencesService(), signedIn: false);
    await c.read(ratingActivityProvider.future);
    expect(await _visible(c), isFalse);
  });

  test('signed-in first-run (never rated) shows the invite', () async {
    final c = _make(prefs: FakePreferencesService());
    await c.read(ratingActivityProvider.future);
    expect(await _visible(c), isTrue);
  });

  test('no invite when there is nothing left to rate', () async {
    // Due to be nudged, but the deck source returns no un-rated scores.
    final c = _make(prefs: FakePreferencesService(), scoresToRate: 0);
    await c.read(ratingActivityProvider.future);
    expect(await _visible(c), isFalse);
  });

  test('a recent stored rating suppresses the invite', () async {
    final prefs = FakePreferencesService({
      RatingActivity.prefsKey: _millis(_now.subtract(const Duration(hours: 6))),
    });
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    expect(await _visible(c), isFalse);
  });

  test('a stale stored rating re-shows the invite', () async {
    final prefs = FakePreferencesService({
      RatingActivity.prefsKey: _millis(_now.subtract(const Duration(days: 10))),
    });
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    expect(await _visible(c), isTrue);
  });

  test('markRatedNow hides the invite and persists the timestamp', () async {
    final prefs = FakePreferencesService();
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    expect(await _visible(c), isTrue); // never rated yet

    await c.read(ratingActivityProvider.notifier).markRatedNow();
    expect(await _visible(c), isFalse);
    expect(prefs.store[RatingActivity.prefsKey], _millis(_now));
  });

  test('snooze suppresses the invite for the snooze window', () async {
    final prefs = FakePreferencesService();
    final c = _make(prefs: prefs);
    await c.read(ratingActivityProvider.future);
    await c.read(ratingActivityProvider.notifier).snooze();
    expect(await _visible(c), isFalse);
    expect(prefs.store[RatingActivity.prefsKey], _millis(_now));
  });

  test('repeated dismissals stop the invite for good', () async {
    final prefs = FakePreferencesService();
    // A different "now" per read so staleness alone would re-show it.
    var day = 0;
    final c = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(prefs),
        canUseOnlineServicesProvider.overrideWithValue(true),
        nowFnProvider.overrideWithValue(
          () => _now.add(Duration(days: 30 * day)),
        ),
      ],
    );
    addTearDown(c.dispose);
    await c.read(ratingActivityProvider.future);
    final notifier = c.read(ratingActivityProvider.notifier);
    for (var i = 0; i < RatingActivity.maxDismissals; i++) {
      day = i + 1;
      await notifier.snooze();
    }
    expect(
      prefs.store[RatingActivity.dismissKey],
      '${RatingActivity.maxDismissals}',
    );
    // Far in the future (well past the snooze window) it still never shows again.
    day = 100;
    expect(await _visible(c), isFalse);
  });
}
