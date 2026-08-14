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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/achievements_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/achievements_notifier.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/widgets/achievements_section.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import 'achievements_section_test.mocks.dart';

@GenerateNiceMocks([MockSpec<AchievementsService>()])
AchievementBadgeView _badge(
  String key, {
  String family = 'play',
  int threshold = 10,
  String track = '',
  int tier = 0,
  bool earned = false,
  int? value,
  DateTime? earnedAt,
  String label = 'Badge',
  String description = 'Do the thing.',
}) => AchievementBadgeView(
  key: key,
  family: family,
  metric: 'session_count',
  threshold: threshold,
  track: track,
  tier: tier,
  earned: earned,
  value: value ?? (earned ? threshold : 0),
  earnedAt: earnedAt,
  label: {'en': label},
  description: {'en': description},
);

/// Host the section in a scroll view, like the profile does.
Widget _host(
  AchievementsService service, {
  Map<String, String>? prefs,
  FakePreferencesService? store,
}) {
  final container = ProviderContainer(
    overrides: [
      achievementsServiceProvider.overrideWithValue(service),
      preferencesServiceProvider.overrideWithValue(
        store ?? FakePreferencesService(prefs),
      ),
      deviceLocaleProvider.overrideWithValue(const Locale('en')),
    ],
  );
  addTearDown(container.dispose);
  return UncontrolledProviderScope(
    container: container,
    child: localizedApp(
      const Scaffold(body: SingleChildScrollView(child: AchievementsSection())),
    ),
  );
}

MockAchievementsService _serving(List<AchievementBadgeView> badges) {
  final service = MockAchievementsService();
  when(service.getAchievements()).thenAnswer((_) async => badges);
  return service;
}

void main() {
  testWidgets('groups tiles by family, earned before locked', (tester) async {
    await tester.binding.setSurfaceSize(const Size(900, 1600));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _host(
        _serving([
          _badge('locked_play', label: 'Repertoire', threshold: 50),
          _badge(
            'earned_play',
            label: 'First Performance',
            threshold: 1,
            earned: true,
          ),
          _badge('first_note', family: 'curation', label: 'First Note'),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Achievements'), findsOneWidget);
    expect(find.byKey(const Key('achievement-family-play')), findsOneWidget);
    expect(
      find.byKey(const Key('achievement-family-curation')),
      findsOneWidget,
    );
    // Earned first within the family: the play grid's first tile is the earned one.
    final tiles = tester
        .widgetList<InkWell>(
          find.descendant(
            of: find.byKey(const Key('achievement-family-play')),
            matching: find.byType(InkWell),
          ),
        )
        .toList();
    expect(tiles.first.key, const Key('achievement-tile-earned_play'));
    expect(tiles.last.key, const Key('achievement-tile-locked_play'));
    // The family header counts what is earned out of what exists.
    expect(find.text('1/2'), findsOneWidget); // play
  });

  testWidgets('a family with no badges is not rendered', (tester) async {
    await tester.pumpWidget(
      _host(_serving([_badge('a', family: 'play', label: 'Only Play')])),
    );
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('achievement-family-play')), findsOneWidget);
    // The server declared `learning` but shipped nothing under it.
    expect(find.byKey(const Key('achievement-family-learning')), findsNothing);
    expect(find.text('Learning'), findsNothing);
  });

  testWidgets('a locked tile shows progress toward its threshold', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        _serving([
          _badge('performer_1', label: 'Performer I', threshold: 25, value: 12),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('12/25'), findsOneWidget);
    final bar = tester.widget<LinearProgressIndicator>(
      find.byKey(const Key('achievement-progress-performer_1')),
    );
    expect(bar.value, closeTo(12 / 25, 1e-9));
  });

  testWidgets('a track renders as ONE tile at its current tier', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        _serving([
          _badge(
            'curator_1',
            family: 'curation',
            label: 'Curator I',
            track: 'curator',
            tier: 1,
            threshold: 10,
            earned: true,
          ),
          _badge(
            'curator_2',
            family: 'curation',
            label: 'Curator II',
            track: 'curator',
            tier: 2,
            threshold: 100,
            earned: true,
          ),
          _badge(
            'curator_3',
            family: 'curation',
            label: 'Curator III',
            track: 'curator',
            tier: 3,
            threshold: 500,
            value: 140,
          ),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    // One tile for the whole ladder, wearing the highest earned tier...
    expect(find.byKey(const Key('achievement-tile-curator')), findsOneWidget);
    expect(find.text('Curator II'), findsOneWidget);
    expect(find.text('Curator I'), findsNothing);
    // ...with progress toward the next rung.
    expect(find.text('140/500'), findsOneWidget);
  });

  testWidgets('the detail sheet fits the landscape viewport of a phone', (
    tester,
  ) async {
    // REGRESSION: the sheet was a plain Column in a bottom sheet capped at 9/16
    // of the viewport height. The app is landscape-ONLY (main.dart pins it), so
    // on an iPhone that budget is 9/16 of 402pt — and a long ladder blew through
    // it ("BOTTOM OVERFLOWED BY 103 PIXELS" on the real device). Sized here to
    // the actual iPhone 17 landscape frame rather than a comfortable test
    // surface, which is exactly why the original test missed it.
    await tester.binding.setSurfaceSize(const Size(874, 402));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _host(
        _serving([
          for (var tier = 1; tier <= 6; tier++)
            _badge(
              'streak_$tier',
              family: 'consistency',
              label: 'Streak $tier',
              description: 'Play $tier days in a row, and then some more.',
              track: 'streak',
              tier: tier,
              threshold: tier * 10,
              earned: tier == 1,
              earnedAt: tier == 1 ? DateTime(2026, 8, 2) : null,
            ),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('achievement-tile-streak')));
    await tester.pumpAndSettle();

    // No RenderFlex overflow — the content scrolls instead.
    expect(tester.takeException(), isNull);
    // ...and every rung is still reachable, including the deepest one.
    expect(
      find.byKey(const Key('achievement-ladder-streak_6')),
      findsOneWidget,
    );
    await tester.drag(
      find.byKey(const Key('achievement-ladder-streak_1')),
      const Offset(0, -200),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });

  testWidgets('the detail sheet shows the whole ladder, earned tiers marked', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(600, 1200));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      _host(
        _serving([
          _badge(
            'streak_1',
            family: 'consistency',
            label: 'Streak I',
            description: 'Play 3 days in a row.',
            track: 'streak',
            tier: 1,
            threshold: 3,
            earned: true,
            earnedAt: DateTime(2026, 8, 2),
          ),
          _badge(
            'streak_2',
            family: 'consistency',
            label: 'Streak II',
            description: 'Play 7 days in a row.',
            track: 'streak',
            tier: 2,
            threshold: 7,
            value: 4,
          ),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('achievement-tile-streak')));
    await tester.pumpAndSettle();

    // What it takes comes from the SERVER copy, not an app switch.
    expect(find.text('Play 7 days in a row.'), findsOneWidget);
    // Every rung of the ladder is listed.
    expect(
      find.byKey(const Key('achievement-ladder-streak_1')),
      findsOneWidget,
    );
    expect(
      find.byKey(const Key('achievement-ladder-streak_2')),
      findsOneWidget,
    );
    // Earned rungs are ticked, locked ones are not.
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(find.byIcon(Icons.radio_button_unchecked), findsOneWidget);
    // The earned date is shown.
    expect(find.textContaining('Earned on'), findsOneWidget);
  });

  testWidgets('a standalone badge sheet has no tier ladder', (tester) async {
    await tester.pumpWidget(
      _host(
        _serving([
          _badge(
            'trailblazer',
            family: 'curation',
            label: 'Trailblazer',
            threshold: 20,
            value: 5,
          ),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('achievement-tile-trailblazer')));
    await tester.pumpAndSettle();

    expect(find.text('Tiers'), findsNothing);
    expect(find.text('Not earned yet'), findsOneWidget);
  });

  testWidgets('a badge earned since the last visit is marked new, then clears', (
    tester,
  ) async {
    final store = FakePreferencesService({
      AchievementsSeen.prefsKey: DateTime(
        2026,
        8,
        1,
      ).millisecondsSinceEpoch.toString(),
    });
    final earnedAt = DateTime(2026, 8, 10);
    await tester.pumpWidget(
      _host(
        _serving([
          _badge(
            'fresh',
            label: 'Fresh',
            threshold: 1,
            earned: true,
            earnedAt: earnedAt,
          ),
        ]),
        store: store,
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('achievement-new-marker')), findsOneWidget);
    expect(find.text('New'), findsOneWidget);
    // Opening the section records the moment, so the marker clears next visit.
    expect(
      store.store[AchievementsSeen.prefsKey],
      earnedAt.millisecondsSinceEpoch.toString(),
    );
  });

  testWidgets('nothing is marked new on a first ever visit', (tester) async {
    await tester.pumpWidget(
      _host(
        _serving([
          _badge(
            'old',
            label: 'Old',
            threshold: 1,
            earned: true,
            earnedAt: DateTime(2026, 8, 10),
          ),
        ]),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('achievement-new-marker')), findsNothing);
  });

  testWidgets('shows a spinner while loading', (tester) async {
    final service = MockAchievementsService();
    when(service.getAchievements()).thenAnswer(
      (_) => Future.delayed(const Duration(seconds: 1), () => const []),
    );
    await tester.pumpWidget(_host(service));
    await tester.pump(); // one frame: still loading

    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    await tester.pumpAndSettle(const Duration(seconds: 2));
  });

  testWidgets('a read failure degrades to a retry, not a broken profile', (
    tester,
  ) async {
    final service = MockAchievementsService();
    var calls = 0;
    when(service.getAchievements()).thenAnswer((_) async {
      calls++;
      if (calls == 1) throw StateError('offline');
      return [_badge('a', label: 'Recovered', threshold: 1, earned: true)];
    });
    await tester.pumpWidget(_host(service));
    await tester.pumpAndSettle();

    expect(
      find.textContaining("Couldn't load your achievements"),
      findsOneWidget,
    );
    expect(find.byIcon(Icons.cloud_off), findsOneWidget);

    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();
    expect(find.text('Recovered'), findsOneWidget);
  });
}
