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
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/post_play_rating_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/post_play_rating.dart';
import 'package:music/widgets/session_summary_modal.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/rating_fakes.dart';

/// The summary's standing block needs a leaderboard seam; an empty board keeps
/// these tests on the rating affordance.
class _EmptyLeaderboardService implements LeaderboardService {
  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async => Leaderboard.empty;

  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};
}

/// A player pinned to a fixed state: 4 notes 500 ms apart, playhead at
/// [furthestMs], so the played-note fraction is deterministic.
class _FixedPlayer extends Player {
  _FixedPlayer(this._value);
  final PlayerData _value;
  @override
  PlayerData build() => _value;
}

PlayerData _playedTo(double furthestMs) => PlayerData(
  notes: [
    for (var i = 0; i < 4; i++)
      TimedNote(pitch: 60 + i, startMs: i * 500, durationMs: 500),
  ],
  furthestElapsedMs: furthestMs,
);

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

late FakeRatingService rating;
late FakePreferencesService prefs;

/// A root container with every seam the prompt touches. Root (not a nested
/// ProviderScope) so the keepAlive overrides stay off `custom_lint`'s
/// scoped-providers rule — the repo convention.
ProviderContainer _container({
  CatalogEntry? selected = _catalogEntry,
  bool online = true,
  double furthestMs = 2000,
  bool failSubmit = false,
  Map<String, String>? storedPrefs,
}) {
  rating = FakeRatingService(fail: failSubmit);
  prefs = FakePreferencesService(storedPrefs);
  final c = ProviderContainer(
    overrides: [
      ratingServiceProvider.overrideWithValue(rating),
      preferencesServiceProvider.overrideWithValue(prefs),
      canUseOnlineServicesProvider.overrideWithValue(online),
      leaderboardServiceProvider.overrideWithValue(_EmptyLeaderboardService()),
      playerProvider.overrideWith(() => _FixedPlayer(_playedTo(furthestMs))),
    ],
  );
  if (selected != null) {
    c.read(selectedScoreProvider.notifier).select(selected);
  }
  return c;
}

SessionResult _result() => SessionResult.fromJudgments(
  pieceId: 'c1',
  title: 'Piece',
  hands: 'both',
  judgments: [
    const NoteJudgment(
      noteIndex: 0,
      pitch: 60,
      startMs: 0,
      waitMode: false,
      verdict: TimingVerdict.perfect,
      timingOffsetMs: 0,
      sustainRatio: 1,
    ),
  ],
  bestCombo: 1,
  playedAtMs: 0,
  speed: 1,
);

/// Opens the summary modal inside [c] and returns the chosen action holder.
Future<List<SummaryAction>> _openSummary(
  WidgetTester tester,
  ProviderContainer c,
) async {
  final actions = <SummaryAction>[];
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () async =>
                  actions.add(await showSessionSummary(context, _result())),
              child: const Text('go'),
            ),
          ),
        ),
      ),
    ),
  );
  // Let the notifier's async build (prefs + rated read) resolve before opening.
  await tester.pumpAndSettle();
  await tester.tap(find.text('go'));
  await tester.pumpAndSettle();
  return actions;
}

/// A screen whose back button funnels through the same exit path as the player:
/// offer the rating when eligible, then leave whatever the user does.
class _ExitHarness extends ConsumerStatefulWidget {
  const _ExitHarness();
  @override
  ConsumerState<_ExitHarness> createState() => _ExitHarnessState();
}

class _ExitHarnessState extends ConsumerState<_ExitHarness> {
  bool _leaving = false;

  Future<void> _requestExit() async {
    if (_leaving) return;
    _leaving = true;
    final navigator = Navigator.of(context);
    if (ref.read(postPlayRatingEligibleProvider(reachedEnd: false))) {
      await showPostPlayRatingSheet(context);
    }
    if (!mounted) return;
    navigator.pop();
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !ref.watch(postPlayRatingEligibleProvider(reachedEnd: false)),
    onPopInvokedWithResult: (didPop, _) {
      if (!didPop) _requestExit();
    },
    child: Scaffold(
      body: Center(
        child: ElevatedButton(
          onPressed: () => Navigator.of(context).maybePop(),
          child: const Text('back'),
        ),
      ),
    ),
  );
}

Future<void> _openPlayer(WidgetTester tester, ProviderContainer c) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(builder: (_) => const _ExitHarness()),
              ),
              child: const Text('open'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  await tester.tap(find.text('open'));
  await tester.pumpAndSettle();
}

void main() {
  group('summary modal', () {
    testWidgets('offers the rating for an eligible score', (tester) async {
      final c = _container();
      addTearDown(c.dispose);
      await _openSummary(tester, c);
      expect(find.byKey(const Key('post-play-star-5')), findsOneWidget);
    });

    testWidgets('shows nothing for a bundled score', (tester) async {
      final c = _container(selected: _bundledEntry);
      addTearDown(c.dispose);
      await _openSummary(tester, c);
      expect(find.byKey(const Key('post-play-star-5')), findsNothing);
      // The modal is otherwise exactly its old self.
      expect(find.text('Retry'), findsOneWidget);
    });

    testWidgets('shows nothing for a signed-out player', (tester) async {
      final c = _container(online: false);
      addTearDown(c.dispose);
      await _openSummary(tester, c);
      expect(find.byKey(const Key('post-play-star-5')), findsNothing);
    });

    testWidgets('shows nothing for an already-rated score', (tester) async {
      final c = _container();
      addTearDown(c.dispose);
      // Rate it first, so the server-truth read reports it as rated.
      await rating.submit(
        catalogId: 'c1',
        verdict: RatingVerdict.like,
        stars: 4,
      );
      await _openSummary(tester, c);
      expect(find.byKey(const Key('post-play-star-5')), findsNothing);
    });

    testWidgets('rating submits and does NOT dismiss the modal', (
      tester,
    ) async {
      final c = _container();
      addTearDown(c.dispose);
      final actions = await _openSummary(tester, c);
      await tester.tap(find.byKey(const Key('post-play-star-4')));
      await tester.pumpAndSettle();
      // Submitted with the deck's derived verdict…
      expect(rating.submissions, [
        (catalogId: 'c1', verdict: RatingVerdict.like, stars: 4),
      ]);
      // …the modal is still up, still awaiting an explicit choice…
      expect(actions, isEmpty);
      expect(find.text('Retry'), findsOneWidget);
      // …and the row switched to its thanks state.
      expect(find.byKey(const Key('post-play-star-4')), findsNothing);
      // Quitting still works afterwards.
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
      expect(actions, [SummaryAction.close]);
    });

    testWidgets('quitting without answering leaves the offer standing', (
      tester,
    ) async {
      // The point of the design change: closing the summary to leave is not a
      // statement about the piece, so nothing is recorded and the next run asks
      // again.
      final c = _container();
      addTearDown(c.dispose);
      final actions = await _openSummary(tester, c);
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
      expect(actions, [SummaryAction.close]);
      expect(rating.submissions, isEmpty);
      expect(prefs.store[PostPlayRating.prefsKey], isNull);
      // Re-opening the summary offers it again.
      await tester.tap(find.text('go'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('post-play-star-5')), findsOneWidget);
    });

    testWidgets('refusing explicitly hides the row and retires the score', (
      tester,
    ) async {
      final c = _container();
      addTearDown(c.dispose);
      final actions = await _openSummary(tester, c);
      await tester.tap(find.byKey(const Key('post-play-rating-skip')));
      await tester.pumpAndSettle();
      // The row is gone but the modal stays, still awaiting an explicit choice.
      expect(find.byKey(const Key('post-play-star-5')), findsNothing);
      expect(actions, isEmpty);
      expect(find.text('Retry'), findsOneWidget);
      expect(prefs.store[PostPlayRating.prefsKey], 'c1');
      // And a later summary for the same score never offers it again.
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
      await tester.tap(find.text('go'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('post-play-star-5')), findsNothing);
    });

    testWidgets(
      'a failed submission stays localized and keeps the modal usable',
      (tester) async {
        final c = _container(failSubmit: true);
        addTearDown(c.dispose);
        final actions = await _openSummary(tester, c);
        await tester.tap(find.byKey(const Key('post-play-star-2')));
        await tester.pumpAndSettle();
        expect(find.textContaining('could not be saved'), findsOneWidget);
        expect(find.textContaining('Exception'), findsNothing);
        // The actions still work.
        await tester.tap(find.byIcon(Icons.close));
        await tester.pumpAndSettle();
        expect(actions, [SummaryAction.close]);
      },
    );

    testWidgets('fits the smallest real phone-landscape window', (
      tester,
    ) async {
      // The app is landscape-locked (main.dart), so the tightest window that can
      // actually happen is a small phone on its side — 568×320, barely taller than
      // the modal's own chrome. The refusal label is long in several languages,
      // hence the Wrap rather than a Row.
      tester.view.physicalSize = const Size(568, 320);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final c = _container();
      addTearDown(c.dispose);
      await _openSummary(tester, c);
      expect(find.byKey(const Key('post-play-star-5')), findsOneWidget);
      expect(find.byKey(const Key('post-play-rating-skip')), findsOneWidget);
      expect(
        tester.getSize(find.byType(PostPlayRatingRow)).height,
        lessThanOrEqualTo(64),
      );
    });

    testWidgets('does not crowd out the statistics on a short viewport', (
      tester,
    ) async {
      // REGRESSION: the first version stacked label / stars / refuse vertically,
      // which on phone landscape ate ~120px and clipped the overall percentage —
      // the modal's headline number — off the top of the scroll area. Asserting
      // the buttons exist was not enough; the row's own height is the thing that
      // has to stay small.
      tester.view.physicalSize = const Size(1200, 480); // phone landscape
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      final c = _container();
      addTearDown(c.dispose);
      await _openSummary(tester, c);

      final rowHeight = tester.getSize(find.byType(PostPlayRatingRow)).height;
      expect(
        rowHeight,
        lessThanOrEqualTo(64),
        reason: 'the rating row must stay a single compact line in the summary',
      );
      // The headline statistic — the modal's whole point — is still rendered.
      expect(find.text('100%'), findsWidgets);
      // …and everything stays reachable.
      expect(find.byKey(const Key('post-play-star-5')), findsOneWidget);
      expect(find.byKey(const Key('post-play-rating-skip')), findsOneWidget);
      expect(find.text('Replay mistakes'), findsOneWidget);
      expect(find.text('Retry'), findsOneWidget);
      expect(find.byIcon(Icons.close), findsOneWidget);
    });
  });

  group('early exit', () {
    testWidgets('the sheet fits a phone-landscape viewport', (tester) async {
      // REGRESSION: a bottom sheet is capped at 9/16 of the screen height, and on
      // a phone lying down that is ~210px — less than the full-size star row plus
      // the label and the refusal button needed.
      tester.view.physicalSize = const Size(
        852,
        393,
      ); // iPhone 15 Pro, landscape
      tester.view.devicePixelRatio = 1.0;
      // The home-indicator inset is what tips it over: the sheet's SafeArea gives
      // up that height, and a widget test reports zero insets unless told.
      tester.view.padding = const FakeViewPadding(bottom: 21);
      tester.view.viewPadding = const FakeViewPadding(bottom: 21);
      addTearDown(tester.view.reset);
      final c = _container();
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(find.byKey(const Key('post-play-star-5')), findsOneWidget);
      expect(find.byKey(const Key('post-play-rating-skip')), findsOneWidget);
    });

    testWidgets('offers the rating, then leaves once it is dismissed', (
      tester,
    ) async {
      final c = _container();
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      // The sheet is up and the player is still there…
      expect(find.byKey(const Key('post-play-star-3')), findsOneWidget);
      expect(find.text('back'), findsOneWidget);
      await tester.tap(find.byKey(const Key('post-play-rating-skip')));
      await tester.pumpAndSettle();
      // …dismissing it leaves, with nothing recorded.
      expect(find.text('back'), findsNothing);
      expect(find.text('open'), findsOneWidget);
      expect(rating.submissions, isEmpty);
    });

    testWidgets('rating on the way out submits and still leaves', (
      tester,
    ) async {
      final c = _container();
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('post-play-star-5')));
      await tester.pumpAndSettle();
      expect(rating.submissions, [
        (catalogId: 'c1', verdict: RatingVerdict.love, stars: 5),
      ]);
      // The sheet stays until dismissed (the thanks state), then leaving works.
      await tester.tapAt(const Offset(10, 10)); // the barrier
      await tester.pumpAndSettle();
      expect(find.text('open'), findsOneWidget);
    });

    testWidgets('tapping outside the sheet also leaves', (tester) async {
      final c = _container();
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      await tester.tapAt(const Offset(10, 10)); // the barrier
      await tester.pumpAndSettle();
      expect(find.text('open'), findsOneWidget);
    });

    testWidgets('a failed submission does not trap the player', (tester) async {
      final c = _container(failSubmit: true);
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('post-play-star-1')));
      await tester.pumpAndSettle();
      expect(find.textContaining('could not be saved'), findsOneWidget);
      await tester.tapAt(const Offset(10, 10)); // the barrier
      await tester.pumpAndSettle();
      expect(find.text('open'), findsOneWidget);
    });

    testWidgets('an ineligible score leaves immediately, with no sheet', (
      tester,
    ) async {
      // Only 1 of 4 notes reached (25% is the bar, and startMs 0 counts as
      // passed) → below the threshold at furthestMs -1.
      final c = _container(furthestMs: -1);
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('post-play-star-3')), findsNothing);
      expect(find.text('open'), findsOneWidget);
    });

    testWidgets('a second back press does not stack another prompt', (
      tester,
    ) async {
      final c = _container();
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('post-play-star-3')), findsOneWidget);
      // A back gesture while the sheet is up dismisses the sheet — and then the
      // exit completes, rather than the guard re-opening a second prompt. It is
      // not a refusal either: nothing is recorded.
      await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('post-play-star-3')), findsNothing);
      expect(find.text('open'), findsOneWidget);
      expect(rating.submissions, isEmpty);
      expect(prefs.store[PostPlayRating.prefsKey], isNull);
    });

    testWidgets('re-entering the player does not prompt again', (tester) async {
      // The score was offered once; the memory is per score and permanent.
      final c = _container();
      addTearDown(c.dispose);
      await _openPlayer(tester, c);
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('post-play-rating-skip')));
      await tester.pumpAndSettle();
      // Back in, play again, leave again → straight out, no sheet.
      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('back'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('post-play-star-3')), findsNothing);
      expect(find.text('open'), findsOneWidget);
    });
  });
}
