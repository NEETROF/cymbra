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
import 'package:music/screens/rating_deck_screen.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/state/rating_coach_notifier.dart';
import 'package:music/widgets/swipe_card.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/rating_fakes.dart';

Future<FakeRatingService> _pumpDeck(
  WidgetTester tester, {
  int rows = 3,
  bool coachSeen = true,
}) async {
  await tester.binding.setSurfaceSize(const Size(900, 1400));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  final rating = FakeRatingService();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        catalogServiceProvider.overrideWithValue(
          FakeDeckCatalogService(deckCorpus(rows)),
        ),
        ratingServiceProvider.overrideWithValue(rating),
        preferencesServiceProvider.overrideWithValue(
          FakePreferencesService(
            coachSeen ? {RatingCoachMark.prefsKey: 'true'} : null,
          ),
        ),
      ],
      child: localizedApp(const RatingDeckScreen()),
    ),
  );
  await tester.pumpAndSettle();
  return rating;
}

void main() {
  testWidgets('tapping Like records a rating and advances the deck', (
    tester,
  ) async {
    final rating = await _pumpDeck(tester);
    // The top card is shown.
    expect(find.text('Piece c0'), findsWidgets);
    await tester.tap(find.byKey(const Key('rating-like')));
    await tester.pumpAndSettle();
    expect(rating.submissions.single.catalogId, 'c0');
    expect(rating.submissions.single.verdict, RatingVerdict.like);
    // The next card is now the top card.
    expect(find.text('Piece c1'), findsWidgets);
  });

  testWidgets('the buttons rate the whole deck without swiping', (
    tester,
  ) async {
    final rating = await _pumpDeck(tester);
    for (final key in ['rating-dislike', 'rating-love', 'rating-like']) {
      await tester.tap(find.byKey(Key(key)));
      await tester.pumpAndSettle();
    }
    expect(rating.submissions.map((s) => s.verdict).toList(), [
      RatingVerdict.dislike,
      RatingVerdict.love,
      RatingVerdict.like,
    ]);
    // All three cards consumed → empty state.
    expect(find.byIcon(Icons.done_all), findsOneWidget);
  });

  testWidgets('Skip advances without recording a rating', (tester) async {
    final rating = await _pumpDeck(tester);
    await tester.tap(find.byKey(const Key('rating-skip')));
    await tester.pumpAndSettle();
    expect(rating.submissions, isEmpty);
    expect(find.text('Piece c1'), findsWidgets); // advanced
  });

  testWidgets('the star sheet submits an explicit rating', (tester) async {
    final rating = await _pumpDeck(tester);
    await tester.tap(find.byKey(const Key('rating-stars')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('rating-star-4')));
    await tester.pumpAndSettle();
    expect(rating.submissions.single.stars, 4);
    expect(rating.submissions.single.verdict, RatingVerdict.like); // 4 → like
  });

  testWidgets('the empty state shows once every card is rated', (tester) async {
    await _pumpDeck(tester, rows: 1);
    await tester.tap(find.byKey(const Key('rating-like')));
    await tester.pumpAndSettle();
    expect(find.byIcon(Icons.done_all), findsOneWidget);
  });

  testWidgets('the deck shows the swipeable top card', (tester) async {
    await _pumpDeck(tester);
    // The card stack renders the top card behind the swipe surface. (The swipe
    // gesture itself is covered end-to-end in swipe_card_test.dart, which drives
    // SwipeCard directly — the full-screen deck's overlay chrome swallows the
    // test harness's synthetic pan before it reaches the gesture detector.)
    expect(find.byType(SwipeCard), findsOneWidget);
    expect(find.text('Piece c0'), findsWidgets);
  });

  testWidgets('the card is bounded and visible on a wide desktop viewport', (
    tester,
  ) async {
    // Regression: the card used to fill the whole area, so on a wide viewport its
    // cover overflowed and nothing rendered. It must stay a bounded, centred card
    // (title visible, no overflow).
    await tester.binding.setSurfaceSize(const Size(2000, 1200));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          catalogServiceProvider.overrideWithValue(
            FakeDeckCatalogService(deckCorpus(3)),
          ),
          ratingServiceProvider.overrideWithValue(FakeRatingService()),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService({RatingCoachMark.prefsKey: 'true'}),
          ),
        ],
        child: localizedApp(const RatingDeckScreen()),
      ),
    );
    await tester.pumpAndSettle();
    // The card renders (title visible) and is capped well below the full width.
    expect(find.text('Piece c0'), findsWidgets);
    expect(
      tester.getSize(find.byType(SwipeCard)).width,
      lessThanOrEqualTo(440),
    );
    expect(tester.takeException(), isNull); // no overflow
  });

  testWidgets('the coach mark shows once and dismisses', (tester) async {
    await _pumpDeck(tester, coachSeen: false);
    // The one-time hint is shown over the first card.
    expect(find.byIcon(Icons.swipe), findsOneWidget);
    await tester.tap(find.text('Got it'));
    await tester.pumpAndSettle();
    expect(find.byIcon(Icons.swipe), findsNothing);
  });
}
