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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/post_play_rating_core.dart';
import 'package:music/state/post_play_rating_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';
import 'post_play_rating_notifier_test.mocks.dart';

@GenerateNiceMocks([MockSpec<RatingService>()])
/// A catalog score (rateable) and a bundled one (not).
const _catalogEntry = CatalogEntry(
  id: 'catalog:c1',
  title: 'Piece',
  composer: 'Composer',
  assetPath: '',
  level: PracticeLevel.beginner,
  catalogId: 'c1',
);

const _bundledEntry = CatalogEntry(
  id: 'ode-to-joy',
  title: 'Ode to Joy',
  composer: 'Beethoven',
  assetPath: 'assets/scores/beginner/ode_to_joy.musicxml',
  level: PracticeLevel.beginner,
);

/// A player pinned to a fixed state, so the eligibility provider sees a
/// deterministic played fraction without driving a real playback loop.
class _FixedPlayer extends Player {
  _FixedPlayer(this._value);
  final PlayerData _value;
  @override
  PlayerData build() => _value;
}

/// `n` notes 500 ms apart; the playhead sits at [furthestMs].
PlayerData _playedTo(double furthestMs, {int n = 4}) => PlayerData(
  notes: [
    for (var i = 0; i < n; i++)
      TimedNote(pitch: 60 + i, startMs: i * 500, durationMs: 500),
  ],
  furthestElapsedMs: furthestMs,
);

void main() {
  late MockRatingService rating;
  late FakePreferencesService prefs;

  ProviderContainer build({
    CatalogEntry? selected = _catalogEntry,
    bool online = true,
    PlayerData? player,
    Map<String, String>? storedPrefs,
  }) {
    rating = MockRatingService();
    when(
      rating.myRating(catalogId: anyNamed('catalogId')),
    ).thenAnswer((_) async => MyRating.none);
    prefs = FakePreferencesService(storedPrefs);
    final container = ProviderContainer(
      overrides: [
        ratingServiceProvider.overrideWithValue(rating),
        preferencesServiceProvider.overrideWithValue(prefs),
        canUseOnlineServicesProvider.overrideWithValue(online),
        playerProvider.overrideWith(
          () => _FixedPlayer(player ?? _playedTo(2000)),
        ),
      ],
    );
    addTearDown(container.dispose);
    if (selected != null) {
      container.read(selectedScoreProvider.notifier).select(selected);
    }
    return container;
  }

  /// Resolves the notifier's async build, then reports eligibility.
  Future<bool> eligible(ProviderContainer c, {bool reachedEnd = false}) async {
    await c.read(postPlayRatingProvider.future);
    return c.read(postPlayRatingEligibleProvider(reachedEnd: reachedEnd));
  }

  group('eligibility', () {
    test('offers the prompt for a played, un-rated catalog score', () async {
      final c = build();
      expect(await eligible(c), isTrue);
      verify(rating.myRating(catalogId: 'c1')).called(1);
    });

    test('never prompts for an already-rated score', () async {
      final c = build();
      when(rating.myRating(catalogId: anyNamed('catalogId'))).thenAnswer(
        (_) async =>
            const MyRating(rated: true, verdict: RatingVerdict.love, stars: 5),
      );
      c.invalidate(postPlayRatingProvider);
      expect(await eligible(c), isFalse);
    });

    test(
      'never prompts for a bundled score, and asks the server nothing',
      () async {
        final c = build(selected: _bundledEntry);
        expect(await eligible(c), isFalse);
        verifyNever(rating.myRating(catalogId: anyNamed('catalogId')));
      },
    );

    test(
      'never prompts a signed-out user, and asks the server nothing',
      () async {
        final c = build(online: false);
        expect(await eligible(c), isFalse);
        verifyNever(rating.myRating(catalogId: anyNamed('catalogId')));
      },
    );

    test('a failed rated read suppresses the prompt (fail-closed)', () async {
      final c = build();
      when(
        rating.myRating(catalogId: anyNamed('catalogId')),
      ).thenThrow(Exception('offline'));
      c.invalidate(postPlayRatingProvider);
      expect(await eligible(c), isFalse);
    });

    test('an explicitly-refused score is never offered again', () async {
      final c = build(storedPrefs: {PostPlayRating.prefsKey: 'other,c1'});
      expect(await eligible(c), isFalse);
    });

    test('being shown the prompt does NOT consume the offer', () async {
      // The behaviour the user asked for: only a rating or an explicit refusal
      // retires a score. Simply having seen the stars and walked away must leave
      // the offer standing for the next run.
      final c = build();
      expect(await eligible(c), isTrue);
      expect(prefs.store[PostPlayRating.prefsKey], isNull);
      // Still eligible after any number of re-reads.
      expect(await eligible(c), isTrue);
    });

    test('too little played does not prompt on an early exit', () async {
      // 1 of 4 notes passed = 25%… so drop to a score where it is below.
      final c = build(player: _playedTo(0, n: 8)); // 1/8 = 12.5%
      expect(await eligible(c), isFalse);
      // …but finishing the run prompts regardless of the note count.
      expect(await eligible(c, reachedEnd: true), isTrue);
    });
  });

  group('decline', () {
    test('records the refusal and persists it', () async {
      final c = build();
      await c.read(postPlayRatingProvider.future);
      await c.read(postPlayRatingProvider.notifier).decline('c1');
      expect(prefs.store[PostPlayRating.prefsKey], 'c1');
      // …and the same score is no longer eligible.
      expect(c.read(postPlayRatingEligibleProvider(reachedEnd: true)), isFalse);
    });

    test('appends to what a previous launch stored', () async {
      final c = build(storedPrefs: {PostPlayRating.prefsKey: 'old1,old2'});
      await c.read(postPlayRatingProvider.future);
      await c.read(postPlayRatingProvider.notifier).decline('c1');
      expect(prefs.store[PostPlayRating.prefsKey], 'old1,old2,c1');
    });

    test('is a no-op for a score already refused', () async {
      final c = build(storedPrefs: {PostPlayRating.prefsKey: 'c1'});
      await c.read(postPlayRatingProvider.future);
      await c.read(postPlayRatingProvider.notifier).decline('c1');
      expect(prefs.store[PostPlayRating.prefsKey], 'c1'); // not duplicated
    });
  });

  group('submit', () {
    test('sends the star value and its derived verdict', () async {
      final c = build();
      await c.read(postPlayRatingProvider.future);
      when(
        rating.submit(
          catalogId: anyNamed('catalogId'),
          verdict: anyNamed('verdict'),
          stars: anyNamed('stars'),
        ),
      ).thenAnswer(
        (_) async => const RatingAggregate(
          average: 4,
          count: 1,
          dislikeCount: 0,
          likeCount: 1,
          loveCount: 0,
        ),
      );
      await c.read(postPlayRatingProvider.notifier).submit(4);
      verify(
        rating.submit(catalogId: 'c1', verdict: RatingVerdict.like, stars: 4),
      ).called(1);
      final data = c.read(postPlayRatingProvider).requireValue;
      expect(data.submittedStars, 4);
      expect(data.failed, isFalse);
      // The score counts as rated straight away, so nothing re-prompts…
      expect(data.rated, RatedState.rated);
      // …and a rating needs no entry in the refusal list to achieve that.
      expect(prefs.store[PostPlayRating.prefsKey], isNull);
      expect(c.read(postPlayRatingEligibleProvider(reachedEnd: true)), isFalse);
    });

    test('folds stars onto the deck\'s verdict scale', () async {
      for (final (stars, verdict) in const [
        (1, RatingVerdict.dislike),
        (2, RatingVerdict.dislike),
        (3, RatingVerdict.like),
        (4, RatingVerdict.like),
        (5, RatingVerdict.love),
      ]) {
        final c = build();
        await c.read(postPlayRatingProvider.future);
        when(
          rating.submit(
            catalogId: anyNamed('catalogId'),
            verdict: anyNamed('verdict'),
            stars: anyNamed('stars'),
          ),
        ).thenAnswer(
          (_) async => const RatingAggregate(
            average: 3,
            count: 1,
            dislikeCount: 0,
            likeCount: 1,
            loveCount: 0,
          ),
        );
        await c.read(postPlayRatingProvider.notifier).submit(stars);
        verify(
          rating.submit(catalogId: 'c1', verdict: verdict, stars: stars),
        ).called(1);
      }
    });

    test('a failed submission flags it without throwing', () async {
      final c = build();
      await c.read(postPlayRatingProvider.future);
      when(
        rating.submit(
          catalogId: anyNamed('catalogId'),
          verdict: anyNamed('verdict'),
          stars: anyNamed('stars'),
        ),
      ).thenThrow(Exception('network down'));
      // Must not throw: the caller may be on its way out of the player.
      await c.read(postPlayRatingProvider.notifier).submit(5);
      expect(c.read(postPlayRatingProvider).requireValue.failed, isTrue);
    });
  });
}
