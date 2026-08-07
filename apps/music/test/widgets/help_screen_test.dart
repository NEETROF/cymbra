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
import 'package:music/screens/help_screen.dart';
import 'package:music/screens/library_screen.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

Future<ProviderContainer> _pumpHelp(WidgetTester tester) async {
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
    ],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const HelpScreen()),
    ),
  );
  await tester.pumpAndSettle();
  return container;
}

void main() {
  testWidgets('help explains the systems, including the one-time hints', (
    tester,
  ) async {
    await _pumpHelp(tester);

    for (final title in const [
      'How it works', // the core loop
      'Swipe or tap the stars', // the rating hint, re-readable here
      'Points',
      'Points, badges and rewards', // shop & badges
      'Your profile',
      'Leaderboards',
      'Public or private profile', // going public
    ]) {
      await tester.scrollUntilVisible(find.text(title), 120);
      expect(find.text(title), findsOneWidget, reason: '$title missing');
    }
  });

  testWidgets('the guided player sequence can be replayed from help', (
    tester,
  ) async {
    final container = await _pumpHelp(tester);
    expect(container.read(coachingProvider).replayArmed, isFalse);

    await tester.scrollUntilVisible(
      find.byKey(const Key('help-replay-player-tour')),
      120,
    );
    await tester.tap(find.byKey(const Key('help-replay-player-tour')));
    await tester.pumpAndSettle();

    // Armed: the sequence runs the next time a score is opened, and the button
    // says so rather than pretending it started here.
    expect(container.read(coachingProvider).replayArmed, isTrue);
    expect(
      find.text('The guide will run the next time you open a score.'),
      findsWidgets,
    );
  });

  testWidgets('help never surfaces moderation to a player', (tester) async {
    await _pumpHelp(tester);
    for (final word in const [
      'Moderation',
      'moderation',
      'Reject',
      'Pending',
    ]) {
      expect(find.textContaining(word), findsNothing, reason: word);
    }
  });

  testWidgets('help is reachable from the library, signed out too', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1200, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final container = ProviderContainer(
      overrides: [
        ...authOverrides(store: FakeTokenStore(guest: true)),
        scoreCatalogProvider.overrideWithValue(const [_entry]),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const LibraryScreen()),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('library-help')));
    await tester.pumpAndSettle();
    expect(find.byType(HelpScreen), findsOneWidget);
  });
}
