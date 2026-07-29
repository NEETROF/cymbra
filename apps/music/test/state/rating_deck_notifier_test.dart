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
import 'package:music/services/rating_service.dart';
import 'package:music/state/rating_activity_notifier.dart';
import 'package:music/state/rating_deck_notifier.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';
import '../support/rating_fakes.dart';

List<CatalogHit> _corpus([int n = 3]) => deckCorpus(n);

ProviderContainer _container(
  FakeDeckCatalogService catalog,
  FakeRatingService rating,
) {
  final c = ProviderContainer(
    overrides: [
      catalogServiceProvider.overrideWithValue(catalog),
      ratingServiceProvider.overrideWithValue(rating),
    ],
  );
  // Hold a subscription so the autoDispose notifier survives across reads.
  final sub = c.listen(ratingDeckProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

/// Wait until the deck settles (initial or paged load done).
Future<RatingDeckState> _settled(ProviderContainer c) async {
  for (var i = 0; i < 40; i++) {
    await Future<void>.delayed(const Duration(milliseconds: 10));
    final s = c.read(ratingDeckProvider);
    if (!s.loading && !s.loadingMore) return s;
  }
  return c.read(ratingDeckProvider);
}

String? _topId(ProviderContainer c) =>
    c.read(ratingDeckProvider).topCard?.catalogId;

/// Unlock the current top card's rating (simulate its preview having played past
/// the threshold), since rating is gated on "listened enough".
void _unlockTop(ProviderContainer c) {
  final id = c.read(ratingDeckProvider).topCard?.catalogId;
  if (id != null) {
    c.read(ratingDeckProvider.notifier).markPreviewed(id, 1.0);
  }
}

void main() {
  test('deck sources accepted cards, top card first', () async {
    final c = _container(
      FakeDeckCatalogService(_corpus()),
      FakeRatingService(),
    );
    final s = await _settled(c);
    expect(s.cards.map((e) => e.catalogId).toList(), ['c0', 'c1', 'c2']);
    expect(s.topCard?.catalogId, 'c0');
  });

  test(
    'a pending card surfaces as a candidate (change: rate-pending-scores)',
    () async {
      // The deck now sources pending candidates too; a pending card carries its
      // status so the UI can label it a "potential new score".
      final c = _container(
        FakeDeckCatalogService([
          deckHit('c0', moderationStatus: 'pending'),
          deckHit('c1'), // accepted
        ]),
        FakeRatingService(),
      );
      final s = await _settled(c);
      final pending = s.cards.firstWhere((e) => e.catalogId == 'c0');
      final accepted = s.cards.firstWhere((e) => e.catalogId == 'c1');
      expect(pending.isPending, isTrue);
      expect(accepted.isPending, isFalse);
    },
  );

  test('a swipe/button rating is submitted and the deck advances', () async {
    final rating = FakeRatingService();
    final c = _container(FakeDeckCatalogService(_corpus()), rating);
    await _settled(c);
    _unlockTop(c);
    await c.read(ratingDeckProvider.notifier).rate(RatingVerdict.like);
    // The verdict was submitted for the top card…
    expect(rating.submissions, [
      (catalogId: 'c0', verdict: RatingVerdict.like, stars: null),
    ]);
    // …and the next card is now on top.
    expect(_topId(c), 'c1');
  });

  test(
    'buttons mirror swipes: dislike/love record the same verdicts',
    () async {
      final rating = FakeRatingService();
      final c = _container(FakeDeckCatalogService(_corpus()), rating);
      await _settled(c);
      _unlockTop(c);
      await c.read(ratingDeckProvider.notifier).rate(RatingVerdict.dislike);
      _unlockTop(c);
      await c.read(ratingDeckProvider.notifier).rate(RatingVerdict.love);
      expect(rating.submissions.map((s) => s.verdict).toList(), [
        RatingVerdict.dislike,
        RatingVerdict.love,
      ]);
      expect(rating.submissions.map((s) => s.catalogId).toList(), ['c0', 'c1']);
    },
  );

  test('skip advances without recording a rating', () async {
    final rating = FakeRatingService();
    final c = _container(FakeDeckCatalogService(_corpus()), rating);
    await _settled(c);
    c.read(ratingDeckProvider.notifier).skip();
    expect(_topId(c), 'c1'); // advanced
    expect(rating.submissions, isEmpty); // nothing recorded
  });

  test('star rating derives the verdict and submits explicit stars', () async {
    final rating = FakeRatingService();
    final c = _container(FakeDeckCatalogService(_corpus()), rating);
    await _settled(c);
    _unlockTop(c);
    await c.read(ratingDeckProvider.notifier).rateStars(5); // → love
    _unlockTop(c);
    await c.read(ratingDeckProvider.notifier).rateStars(4); // → like
    _unlockTop(c);
    await c.read(ratingDeckProvider.notifier).rateStars(1); // → dislike
    expect(rating.submissions, [
      (catalogId: 'c0', verdict: RatingVerdict.love, stars: 5),
      (catalogId: 'c1', verdict: RatingVerdict.like, stars: 4),
      (catalogId: 'c2', verdict: RatingVerdict.dislike, stars: 1),
    ]);
  });

  test('deck empties once every sourced card is rated', () async {
    final c = _container(
      FakeDeckCatalogService(_corpus()),
      FakeRatingService(),
    );
    await _settled(c);
    for (var i = 0; i < 3; i++) {
      _unlockTop(c);
      await c.read(ratingDeckProvider.notifier).rate(RatingVerdict.like);
    }
    final s = c.read(ratingDeckProvider);
    expect(s.topCard, isNull);
    expect(s.isExhausted, isTrue); // last-card/empty state, not repeating cards
  });

  test(
    'a failed submit reverts the advance so the card can be re-rated',
    () async {
      final c = _container(
        FakeDeckCatalogService(_corpus()),
        FakeRatingService(fail: true),
      );
      await _settled(c);
      _unlockTop(c);
      await c.read(ratingDeckProvider.notifier).rate(RatingVerdict.like);
      final s = c.read(ratingDeckProvider);
      expect(s.topCard?.catalogId, 'c0'); // reverted to the same card
      expect(s.error, isNotNull); // surfaced for the snackbar listener
    },
  );

  test('loadMore appends the next page without repeating cards', () async {
    // 22 rows → first page of 20, second page of 2.
    final c = _container(
      FakeDeckCatalogService(_corpus(22)),
      FakeRatingService(),
    );
    final first = await _settled(c);
    expect(first.cards.length, 20);
    expect(first.hasMore, isTrue);
    await c.read(ratingDeckProvider.notifier).loadMore();
    final more = await _settled(c);
    expect(more.cards.length, 22);
    // No duplicate ids across the two pages.
    final ids = more.cards.map((e) => e.catalogId).toList();
    expect(ids.toSet().length, ids.length);
    expect(more.hasMore, isFalse);
  });

  // --- listen-before-rating gate ------------------------------------------

  test(
    'rating is ignored until the preview passes the unlock threshold',
    () async {
      final rating = FakeRatingService();
      final c = _container(FakeDeckCatalogService(_corpus()), rating);
      await _settled(c);
      final notifier = c.read(ratingDeckProvider.notifier);
      // Locked at first: a rate is a no-op (no submit, no advance).
      expect(c.read(ratingDeckProvider).topUnlocked, isFalse);
      await notifier.rate(RatingVerdict.like);
      expect(rating.submissions, isEmpty);
      expect(_topId(c), 'c0');
      // Below the threshold stays locked…
      notifier.markPreviewed('c0', RatingDeck.previewUnlockProgress - 0.05);
      expect(c.read(ratingDeckProvider).topUnlocked, isFalse);
      await notifier.rate(RatingVerdict.like);
      expect(rating.submissions, isEmpty);
      // …reaching the threshold unlocks it, and the rating goes through.
      notifier.markPreviewed('c0', RatingDeck.previewUnlockProgress);
      expect(c.read(ratingDeckProvider).topUnlocked, isTrue);
      await notifier.rate(RatingVerdict.like);
      expect(rating.submissions.single.catalogId, 'c0');
      expect(_topId(c), 'c1');
    },
  );

  test('skip is allowed even while the card is locked', () async {
    final rating = FakeRatingService();
    final c = _container(FakeDeckCatalogService(_corpus()), rating);
    await _settled(c);
    expect(c.read(ratingDeckProvider).topUnlocked, isFalse);
    c.read(ratingDeckProvider.notifier).skip();
    expect(_topId(c), 'c1'); // advanced without previewing
    expect(rating.submissions, isEmpty);
  });

  ProviderContainer engagementContainer(FakePreferencesService prefs) {
    final c = ProviderContainer(
      overrides: [
        catalogServiceProvider.overrideWithValue(
          FakeDeckCatalogService(_corpus()),
        ),
        ratingServiceProvider.overrideWithValue(FakeRatingService()),
        preferencesServiceProvider.overrideWithValue(prefs),
        canUseOnlineServicesProvider.overrideWithValue(true),
        nowFnProvider.overrideWithValue(() => DateTime(2026, 7, 28)),
      ],
    );
    final sub = c.listen(ratingDeckProvider, (_, _) {});
    addTearDown(sub.close);
    addTearDown(c.dispose);
    return c;
  }

  test('rating a card records activity so the library invite hides', () async {
    final c = engagementContainer(FakePreferencesService());
    await _settled(c);
    await c.read(ratingActivityProvider.future);
    expect(
      await c.read(ratingInviteVisibleProvider.future),
      isTrue,
    ); // never rated → due
    _unlockTop(c);
    await c.read(ratingDeckProvider.notifier).rate(RatingVerdict.like);
    expect(await c.read(ratingInviteVisibleProvider.future), isFalse);
  });

  test(
    'going through cards (even skipping) hides the library invite',
    () async {
      final c = engagementContainer(FakePreferencesService());
      await _settled(c);
      await c.read(ratingActivityProvider.future);
      expect(await c.read(ratingInviteVisibleProvider.future), isTrue);
      // Just advancing the deck — a skip, no rating — counts as engagement.
      c.read(ratingDeckProvider.notifier).skip();
      expect(await c.read(ratingInviteVisibleProvider.future), isFalse);
    },
  );
}
