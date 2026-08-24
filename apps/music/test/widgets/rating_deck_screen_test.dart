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
import 'package:music/services/audio_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/selected_kit.dart';
import 'package:music/state/selected_piano.dart';
import 'package:music/state/rating_deck_notifier.dart';
import 'package:music/widgets/swipe_card.dart';
import 'package:music/widgets/sound_selector_field.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/rating_fakes.dart';

/// The top card auto-plays a looping preview (a continuous ticker), so the deck
/// widget tree never "settles" — pump explicitly instead of `pumpAndSettle`.
Future<void> _settle(WidgetTester tester, {int frames = 8}) async {
  for (var i = 0; i < frames; i++) {
    await tester.pump(const Duration(milliseconds: 16));
  }
}

Future<FakeRatingService> _pumpDeck(
  WidgetTester tester, {
  int rows = 3,
  bool coachSeen = true,
  List<CatalogHit>? corpus,
  bool drums = false,
}) async {
  await tester.binding.setSurfaceSize(const Size(900, 1400));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  final rating = FakeRatingService();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        drumsEnabledProvider.overrideWithValue(drums),
        catalogServiceProvider.overrideWithValue(
          FakeDeckCatalogService(corpus ?? deckCorpus(rows)),
        ),
        ratingServiceProvider.overrideWithValue(rating),
        // The card auto-plays its preview: load it through the fake notation +
        // audio seams so it renders (no native library).
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        preferencesServiceProvider.overrideWithValue(
          FakePreferencesService(
            coachSeen ? {CoachHint.ratingDeck.prefsKey: 'true'} : null,
          ),
        ),
      ],
      child: localizedApp(const RatingDeckScreen()),
    ),
  );
  await _settle(tester);
  return rating;
}

ProviderContainer _container(WidgetTester tester) =>
    ProviderScope.containerOf(tester.element(find.byType(RatingDeckScreen)));

/// Simulate the top card's preview having played enough to unlock its rating.
Future<void> _unlockTop(WidgetTester tester) async {
  final c = _container(tester);
  final id = c.read(ratingDeckProvider).topCard?.catalogId;
  if (id != null) c.read(ratingDeckProvider.notifier).markPreviewed(id, 1.0);
  await tester.pump();
}

void main() {
  testWidgets('the rater chooses which instrument to be dealt', (tester) async {
    await _pumpDeck(
      tester,
      drums: true,
      corpus: [
        deckHit('k0', instrument: ScoreInstrument.keyboard),
        deckHit('d0', instrument: ScoreInstrument.percussion),
        deckHit('d1', instrument: ScoreInstrument.percussion),
      ],
    );
    final c = _container(tester);
    expect(c.read(ratingDeckProvider).cards, hasLength(3));

    // Picking a family re-sources the deck at once: the cards already in hand
    // were dealt under the old choice.
    await tester.tap(find.widgetWithText(ChoiceChip, 'Drums'));
    await _settle(tester);
    expect(c.read(ratingDeckProvider).instrument, ScoreInstrument.percussion);
    expect(c.read(ratingDeckProvider).cards.map((e) => e.catalogId), [
      'd0',
      'd1',
    ]);

    await tester.tap(find.widgetWithText(ChoiceChip, 'Piano'));
    await _settle(tester);
    expect(c.read(ratingDeckProvider).cards.map((e) => e.catalogId), ['k0']);

    await tester.tap(find.widgetWithText(ChoiceChip, 'Any'));
    await _settle(tester);
    expect(c.read(ratingDeckProvider).instrument, isNull);
    expect(c.read(ratingDeckProvider).cards, hasLength(3));
  });

  testWidgets('no instrument chooser without the drum audience', (
    tester,
  ) async {
    await _pumpDeck(tester);
    expect(find.byType(ChoiceChip), findsNothing);
  });

  testWidgets('the sound selector follows the card being auditioned', (
    tester,
  ) async {
    // The deck is drum-compatible end to end — the backend serves percussion
    // cards to eligible callers, the card engraves them, the preview sounds
    // them on the drum channel — but its sound control was bound to the piano
    // memory alone: on a drum card it offered a knob that changes nothing you
    // are hearing, and moved the piano while you listened to a groove.
    await _pumpDeck(
      tester,
      corpus: [
        deckHit('d0', instrument: ScoreInstrument.percussion),
        deckHit('c1'),
      ],
    );
    final c = _container(tester);
    expect(
      tester.widget<SoundSelectorField>(find.byType(SoundSelectorField)).family,
      SoundFamily.percussion,
    );
    expect(
      tester.widget<SoundSelectorField>(find.byType(SoundSelectorField)).value,
      c.read(selectedKitProvider),
    );

    // Past the drum card, the control goes back to the piano memory.
    await _unlockTop(tester);
    c.read(ratingDeckProvider.notifier).skip();
    await _settle(tester);
    expect(
      tester.widget<SoundSelectorField>(find.byType(SoundSelectorField)).family,
      SoundFamily.keyboard,
    );
    expect(
      tester.widget<SoundSelectorField>(find.byType(SoundSelectorField)).value,
      c.read(selectedPianoProvider),
    );
  });

  testWidgets('tapping Like records a rating and advances the deck', (
    tester,
  ) async {
    final rating = await _pumpDeck(tester);
    expect(find.text('Piece c0'), findsWidgets);
    await _unlockTop(tester);
    await tester.tap(find.byKey(const Key('rating-like')));
    await _settle(tester);
    expect(rating.submissions.single.catalogId, 'c0');
    expect(rating.submissions.single.verdict, RatingVerdict.like);
    expect(find.text('Piece c1'), findsWidgets);
  });

  testWidgets('the buttons rate the whole deck without swiping', (
    tester,
  ) async {
    final rating = await _pumpDeck(tester);
    for (final key in ['rating-dislike', 'rating-love', 'rating-like']) {
      await _unlockTop(tester);
      await tester.tap(find.byKey(Key(key)));
      await _settle(tester);
    }
    expect(rating.submissions.map((s) => s.verdict).toList(), [
      RatingVerdict.dislike,
      RatingVerdict.love,
      RatingVerdict.like,
    ]);
    expect(find.byIcon(Icons.done_all), findsOneWidget); // empty state
  });

  testWidgets('Skip advances without recording a rating (even locked)', (
    tester,
  ) async {
    final rating = await _pumpDeck(tester);
    // No unlock: Skip stays available while the card is locked.
    await tester.tap(find.byKey(const Key('rating-skip')));
    await _settle(tester);
    expect(rating.submissions, isEmpty);
    expect(find.text('Piece c1'), findsWidgets);
  });

  testWidgets('rating is locked until the card is previewed', (tester) async {
    final rating = await _pumpDeck(tester);
    // Locked: the rating buttons are disabled and a hint is shown.
    expect(find.byKey(const Key('rating-locked-hint')), findsOneWidget);
    final likeButton = tester.widget<InkWell>(
      find.descendant(
        of: find.byKey(const Key('rating-like')),
        matching: find.byType(InkWell),
      ),
    );
    expect(likeButton.onTap, isNull); // disabled
    // Once previewed enough, the controls unlock.
    await _unlockTop(tester);
    expect(find.byKey(const Key('rating-locked-hint')), findsNothing);
    expect(find.byKey(const Key('rating-stars')), findsOneWidget);
    expect(rating.submissions, isEmpty);
  });

  testWidgets('the star sheet submits an explicit rating', (tester) async {
    final rating = await _pumpDeck(tester);
    await _unlockTop(tester);
    await tester.tap(find.byKey(const Key('rating-stars')));
    await _settle(tester); // open the sheet
    await tester.tap(find.byKey(const Key('rating-star-4')));
    await _settle(tester);
    expect(rating.submissions.single.stars, 4);
    expect(rating.submissions.single.verdict, RatingVerdict.like); // 4 → like
  });

  testWidgets('the empty state shows once every card is rated', (tester) async {
    await _pumpDeck(tester, rows: 1);
    await _unlockTop(tester);
    await tester.tap(find.byKey(const Key('rating-like')));
    await _settle(tester);
    expect(find.byIcon(Icons.done_all), findsOneWidget);
  });

  testWidgets('the deck shows the swipeable top card', (tester) async {
    await _pumpDeck(tester);
    expect(find.byType(SwipeCard), findsOneWidget);
    expect(find.text('Piece c0'), findsWidgets);
  });

  testWidgets('the top card shows a listen-to-unlock progress bar', (
    tester,
  ) async {
    await _pumpDeck(tester);
    // The thin bar (a LinearProgressIndicator at the notation/title seam) is
    // shown while the required listening time is still elapsing.
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
  });

  testWidgets('the card is bounded and visible on a wide desktop viewport', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(2000, 1200));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          catalogServiceProvider.overrideWithValue(
            FakeDeckCatalogService(deckCorpus(3)),
          ),
          ratingServiceProvider.overrideWithValue(FakeRatingService()),
          notationEngineProvider.overrideWithValue(FakeNotationEngine()),
          audioServiceProvider.overrideWithValue(RecordingAudioService()),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService({CoachHint.ratingDeck.prefsKey: 'true'}),
          ),
        ],
        child: localizedApp(const RatingDeckScreen()),
      ),
    );
    await _settle(tester);
    expect(find.text('Piece c0'), findsWidgets);
    expect(
      tester.getSize(find.byType(SwipeCard)).width,
      lessThanOrEqualTo(440),
    );
    expect(tester.takeException(), isNull); // no overflow
  });

  testWidgets('the coach mark shows once and dismisses', (tester) async {
    await _pumpDeck(tester, coachSeen: false);
    expect(find.byKey(const Key('coach-mark-bubble')), findsOneWidget);
    await tester.tap(find.text('Got it'));
    await _settle(tester);
    expect(find.byKey(const Key('coach-mark-bubble')), findsNothing);
  });

  testWidgets('the coach mark fits a short landscape viewport (no overflow)', (
    tester,
  ) async {
    // Regression: on a short landscape viewport the coach mark's column
    // overflowed; it must now fit (scroll) rather than overflow.
    await tester.binding.setSurfaceSize(const Size(900, 400));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          catalogServiceProvider.overrideWithValue(
            FakeDeckCatalogService(deckCorpus(2)),
          ),
          ratingServiceProvider.overrideWithValue(FakeRatingService()),
          notationEngineProvider.overrideWithValue(FakeNotationEngine()),
          audioServiceProvider.overrideWithValue(RecordingAudioService()),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(null), // never seen → coach mark shows
          ),
        ],
        child: localizedApp(const RatingDeckScreen()),
      ),
    );
    await _settle(tester);
    expect(find.byKey(const Key('coach-mark-bubble')), findsOneWidget);
    expect(tester.takeException(), isNull); // no RenderFlex overflow
  });

  testWidgets('phone-landscape uses a side rail so the card keeps its height', (
    tester,
  ) async {
    // Short phone-landscape: the controls move to a side rail so the card gets
    // the full height (a bottom bar would squeeze the score into a sliver).
    await tester.binding.setSurfaceSize(const Size(760, 360));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          catalogServiceProvider.overrideWithValue(
            FakeDeckCatalogService(deckCorpus(2)),
          ),
          ratingServiceProvider.overrideWithValue(FakeRatingService()),
          notationEngineProvider.overrideWithValue(FakeNotationEngine()),
          audioServiceProvider.overrideWithValue(RecordingAudioService()),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService({CoachHint.ratingDeck.prefsKey: 'true'}),
          ),
        ],
        child: localizedApp(const RatingDeckScreen()),
      ),
    );
    await _settle(tester);
    await _unlockTop(tester); // clears the locked hint → card takes full height
    expect(find.byKey(const Key('rating-love')), findsOneWidget);
    // Side rail → the card is tall (well beyond the ~170px a bottom bar leaves).
    expect(tester.getSize(find.byType(SwipeCard)).height, greaterThan(240));
    expect(tester.takeException(), isNull);
  });

  testWidgets('the listen-before-rating caption is visible while locked', (
    tester,
  ) async {
    // The gate must be explained on screen (not just a tooltip): a visible
    // caption with the hint text while locked, gone once unlocked.
    final rating = await _pumpDeck(tester);
    expect(find.byKey(const Key('rating-locked-hint')), findsOneWidget);
    expect(find.byIcon(Icons.headphones), findsOneWidget);
    await _unlockTop(tester);
    expect(find.byKey(const Key('rating-locked-hint')), findsNothing);
    expect(rating.submissions, isEmpty);
  });
}
