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

import 'dart:ui' show Locale;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/achievements_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/achievements_notifier.dart';
import 'package:music/state/app_locale.dart';

import '../support/prefs_fakes.dart';
import 'achievements_test.mocks.dart';

@GenerateNiceMocks([MockSpec<AchievementsService>()])
/// One badge as the server would send it.
AchievementBadgeView _badge(
  String key, {
  String family = 'play',
  String metric = 'session_count',
  int threshold = 10,
  String track = '',
  int tier = 0,
  bool earned = false,
  int? value,
  DateTime? earnedAt,
  Map<String, String> label = const {'en': 'Badge', 'fr': 'Insigne'},
  Map<String, String> description = const {'en': 'Do the thing.'},
}) => AchievementBadgeView(
  key: key,
  family: family,
  metric: metric,
  threshold: threshold,
  track: track,
  tier: tier,
  earned: earned,
  // The server clamps an earned badge to its threshold.
  value: value ?? (earned ? threshold : 0),
  earnedAt: earnedAt,
  label: label,
  description: description,
);

ProviderContainer _container(
  AchievementsService service, {
  Map<String, String>? prefs,
  Locale locale = const Locale('en'),
}) {
  final c = ProviderContainer(
    overrides: [
      achievementsServiceProvider.overrideWithValue(service),
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      deviceLocaleProvider.overrideWithValue(locale),
    ],
  );
  addTearDown(c.dispose);
  // Keep the auto-dispose notifier alive across its own async gaps: `build`
  // awaits the seen timestamp before the service call, and an unlistened
  // provider would be reaped in between. In the app the section is the listener.
  c.listen(achievementsProvider, (_, _) {});
  return c;
}

MockAchievementsService _serving(List<AchievementBadgeView> badges) {
  final service = MockAchievementsService();
  when(service.getAchievements()).thenAnswer((_) async => badges);
  return service;
}

void main() {
  group('AchievementBadgeView', () {
    test('progress is the fraction of the threshold, clamped to 0..1', () {
      expect(
        _badge('a', threshold: 25, value: 12).progress,
        closeTo(0.48, 1e-9),
      );
      expect(_badge('a', threshold: 25, value: 0).progress, 0);
      // The server clamps, but a defensive over-value never overflows the bar.
      expect(_badge('a', threshold: 25, value: 900).progress, 1);
    });

    test('a degenerate threshold reads as complete, not a divide by zero', () {
      expect(_badge('a', threshold: 0, value: 0).progress, 1);
    });

    test('label falls back to English when the language is missing', () {
      final b = _badge('a', label: const {'en': 'Streak', 'fr': 'Série'});
      expect(b.labelIn('fr'), 'Série');
      expect(b.labelIn('it'), 'Streak'); // no Italian → English
      expect(b.descriptionIn('es'), 'Do the thing.');
    });

    test('isTracked distinguishes a ladder rung from a standalone badge', () {
      expect(_badge('a').isTracked, isFalse);
      expect(_badge('a', track: 'streak', tier: 1).isTracked, isTrue);
    });
  });

  group('groupAchievements', () {
    test('groups by family in the order the server sent them', () {
      final view = groupAchievements([
        _badge('p1', family: 'play'),
        _badge('c1', family: 'curation'),
        _badge('p2', family: 'play'),
        _badge('l1', family: 'learning'),
      ], languageCode: 'en');
      // The app holds no family list of its own — registry order wins.
      expect(view.families.map((f) => f.family), [
        'play',
        'curation',
        'learning',
      ]);
      expect(view.families.first.tiles.length, 2);
    });

    test('a family with no badges does not render', () {
      // The server declared `learning` but shipped nothing under it.
      final view = groupAchievements([
        _badge('p1', family: 'play'),
      ], languageCode: 'en');
      expect(view.families.map((f) => f.family), ['play']);
      expect(view.families.any((f) => f.family == 'learning'), isFalse);
    });

    test('earned tiles come before locked ones within a family', () {
      final view = groupAchievements([
        _badge('locked_a', threshold: 50),
        _badge('earned_a', threshold: 1, earned: true),
        _badge('locked_b', threshold: 90),
        _badge('earned_b', threshold: 2, earned: true),
      ], languageCode: 'en');
      final keys = view.families.single.tiles.map((t) => t.badge.key).toList();
      expect(keys, ['earned_a', 'earned_b', 'locked_a', 'locked_b']);
      expect(view.families.single.earnedCount, 2);
      expect(view.families.single.totalCount, 4);
    });

    test('a track collapses to ONE tile at the highest earned tier', () {
      final view = groupAchievements([
        _badge(
          'curator_1',
          family: 'curation',
          track: 'curator',
          tier: 1,
          threshold: 10,
          earned: true,
        ),
        _badge(
          'curator_2',
          family: 'curation',
          track: 'curator',
          tier: 2,
          threshold: 100,
          earned: true,
        ),
        _badge(
          'curator_3',
          family: 'curation',
          track: 'curator',
          tier: 3,
          threshold: 500,
          value: 140,
        ),
      ], languageCode: 'en');
      final tile = view.families.single.tiles.single; // not three tiles
      expect(tile.badge.key, 'curator_2'); // highest earned
      expect(tile.next?.key, 'curator_3'); // progress points at the next rung
      expect(tile.target.value, 140);
      expect(tile.earned, isTrue);
      expect(tile.id, 'curator');
      // The whole ladder travels with the tile for the detail sheet.
      expect(tile.ladder.map((b) => b.key), [
        'curator_1',
        'curator_2',
        'curator_3',
      ]);
    });

    test('an untouched track shows its ENTRY rung, not its top', () {
      final view = groupAchievements([
        _badge('streak_1', track: 'streak', tier: 1, threshold: 3),
        _badge('streak_2', track: 'streak', tier: 2, threshold: 7),
        _badge('streak_3', track: 'streak', tier: 3, threshold: 30),
      ], languageCode: 'en');
      final tile = view.families.single.tiles.single;
      expect(tile.badge.key, 'streak_1');
      expect(tile.next?.key, 'streak_1');
      expect(tile.earned, isFalse);
    });

    test('a completed track has no next rung', () {
      final view = groupAchievements([
        _badge('s1', track: 's', tier: 1, threshold: 3, earned: true),
        _badge('s2', track: 's', tier: 2, threshold: 7, earned: true),
      ], languageCode: 'en');
      final tile = view.families.single.tiles.single;
      expect(tile.badge.key, 's2');
      expect(tile.next, isNull);
      expect(tile.target.key, 's2');
    });

    test(
      'a standalone badge is a one-rung ladder, so the sheet needs no case',
      () {
        final view = groupAchievements([
          _badge('trailblazer', threshold: 20, earned: true),
        ], languageCode: 'en');
        final tile = view.families.single.tiles.single;
        expect(tile.ladder.map((b) => b.key), ['trailblazer']);
        expect(tile.next, isNull);
        expect(tile.id, 'trailblazer');
      },
    );

    test('a badge earned since the last visit is marked new', () {
      final view = groupAchievements(
        [
          _badge(
            'fresh',
            threshold: 1,
            earned: true,
            earnedAt: DateTime(2026, 8, 10),
          ),
          _badge(
            'old',
            threshold: 2,
            earned: true,
            earnedAt: DateTime(2026, 7, 1),
          ),
        ],
        languageCode: 'en',
        lastSeen: DateTime(2026, 8, 1),
      );
      final tiles = {
        for (final t in view.families.single.tiles) t.badge.key: t,
      };
      expect(tiles['fresh']!.isNew, isTrue);
      expect(tiles['old']!.isNew, isFalse);
      // The listener records this so the marks clear next visit.
      expect(view.newestEarnedAt, DateTime(2026, 8, 10));
    });

    test('a never-visited section marks nothing new', () {
      // The first visit sets the baseline — an existing user must not be shown a
      // lifetime of badges all flagged at once.
      final view = groupAchievements([
        _badge('a', earned: true, earnedAt: DateTime(2026, 8, 10)),
      ], languageCode: 'en');
      expect(view.families.single.tiles.single.isNew, isFalse);
    });

    test('newestEarnedAt is null when nothing has been earned', () {
      final view = groupAchievements([_badge('a')], languageCode: 'en');
      expect(view.newestEarnedAt, isNull);
      expect(view.isEmpty, isFalse);
    });

    test('an empty projection yields an empty surface', () {
      final view = groupAchievements(const [], languageCode: 'en');
      expect(view.isEmpty, isTrue);
      expect(view.newestEarnedAt, isNull);
    });
  });

  group('Achievements notifier', () {
    test('loads and groups the registry from the service', () async {
      final c = _container(
        _serving([
          _badge('first_performance', threshold: 1, earned: true),
          _badge(
            'performer_1',
            track: 'performer',
            tier: 1,
            threshold: 25,
            value: 12,
          ),
          _badge('first_note', family: 'curation', threshold: 1),
        ]),
      );
      final view = await c.read(achievementsProvider.future);
      expect(view.families.map((f) => f.family), ['play', 'curation']);
      expect(view.languageCode, 'en');
      // A player who never rates still has an earned badge on the surface.
      expect(view.families.first.earnedCount, 1);
    });

    test('resolves labels against the ACTIVE DISPLAY language', () async {
      final badges = [
        _badge(
          'streak_1',
          label: const {'en': 'Streak I', 'fr': 'Série I'},
          description: const {
            'en': 'Three in a row.',
            'fr': 'Trois d\'affilée.',
          },
        ),
      ];
      final fr = _container(_serving(badges), locale: const Locale('fr'));
      final frView = await fr.read(achievementsProvider.future);
      final badge = frView.families.single.tiles.single.badge;
      expect(badge.labelIn(frView.languageCode), 'Série I');
      expect(badge.descriptionIn(frView.languageCode), 'Trois d\'affilée.');

      // A language the registry has no entry for falls back to English.
      final it = _container(_serving(badges), locale: const Locale('it'));
      final itView = await it.read(achievementsProvider.future);
      expect(
        itView.families.single.tiles.single.badge.labelIn(itView.languageCode),
        'Streak I',
      );
    });

    test('marks new against the persisted seen timestamp', () async {
      final c = _container(
        _serving([
          _badge(
            'fresh',
            threshold: 1,
            earned: true,
            earnedAt: DateTime(2026, 8, 10),
          ),
        ]),
        prefs: {
          AchievementsSeen.prefsKey: DateTime(
            2026,
            8,
            1,
          ).millisecondsSinceEpoch.toString(),
        },
      );
      final view = await c.read(achievementsProvider.future);
      expect(view.families.single.tiles.single.isNew, isTrue);
    });

    test('refresh reloads from the service', () async {
      final service = MockAchievementsService();
      var earned = false;
      when(service.getAchievements()).thenAnswer(
        (_) async => [
          _badge('first_performance', threshold: 1, earned: earned),
        ],
      );
      final c = _container(service);
      final before = await c.read(achievementsProvider.future);
      expect(before.families.single.earnedCount, 0);

      earned = true;
      await c.read(achievementsProvider.notifier).refresh();
      expect(
        c.read(achievementsProvider).value!.families.single.earnedCount,
        1,
      );
    });

    test('a service failure lands in the state rather than throwing', () async {
      final service = MockAchievementsService();
      when(
        service.getAchievements(),
      ).thenAnswer((_) async => throw StateError('offline'));
      final c = _container(service);
      await expectLater(
        c.read(achievementsProvider.future),
        throwsA(isA<StateError>()),
      );
      expect(c.read(achievementsProvider).hasError, isTrue);
    });
  });

  group('AchievementsSeen', () {
    test('reads back the persisted moment', () async {
      final at = DateTime(2026, 8, 5);
      final c = _container(
        _serving(const []),
        prefs: {
          AchievementsSeen.prefsKey: at.millisecondsSinceEpoch.toString(),
        },
      );
      expect(await c.read(achievementsSeenProvider.future), at);
    });

    test('is null when nothing was ever seen', () async {
      final c = _container(_serving(const []));
      expect(await c.read(achievementsSeenProvider.future), isNull);
    });

    test('markSeen persists under its OWN key, not the curator one', () async {
      final prefs = FakePreferencesService();
      final c = ProviderContainer(
        overrides: [
          achievementsServiceProvider.overrideWithValue(_serving(const [])),
          preferencesServiceProvider.overrideWithValue(prefs),
          deviceLocaleProvider.overrideWithValue(const Locale('en')),
        ],
      );
      addTearDown(c.dispose);
      await c.read(achievementsSeenProvider.future);

      final at = DateTime(2026, 8, 12);
      await c.read(achievementsSeenProvider.notifier).markSeen(at);
      expect(
        prefs.store[AchievementsSeen.prefsKey],
        at.millisecondsSinceEpoch.toString(),
      );
      expect(prefs.store.containsKey('curator_activity_seen'), isFalse);
      expect(c.read(achievementsSeenProvider).value, at);
    });

    test('markSeen never moves the mark backwards', () async {
      final c = _container(_serving(const []));
      await c.read(achievementsSeenProvider.future);
      final notifier = c.read(achievementsSeenProvider.notifier);
      await notifier.markSeen(DateTime(2026, 8, 12));
      // An out-of-order call must not resurrect already-cleared markers.
      await notifier.markSeen(DateTime(2026, 8, 1));
      expect(c.read(achievementsSeenProvider).value, DateTime(2026, 8, 12));
    });
  });
}
