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

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../services/rating_service.dart';
import '../state/rating_coach_notifier.dart';
import '../state/rating_deck_notifier.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/rating_card.dart';
import '../widgets/rating_deck_controls.dart';
import '../widgets/swipe_card.dart';

/// The Tinder-style swipe-rating deck (change: add-app-score-rating): a stack of
/// `accepted` catalog cards the user rates by swiping (left = dislike, right =
/// like, up = love) or with the mirrored buttons below, opens a 1–5 star rating
/// by tapping the card, and previews read-only via the card's Play control.
class RatingDeckScreen extends ConsumerWidget {
  const RatingDeckScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.ratingDeckTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      // The deck's side effects (a failed submit → snackbar) live in a dedicated
      // listener widget near the top of the subtree (architecture rule 4).
      body: const _RatingDeckListeners(
        child: SafeArea(child: _RatingDeckBody()),
      ),
    );
  }
}

/// Isolates the deck's `ref.listen` side effects (error snackbars) so they are
/// not scattered through the build methods below.
class _RatingDeckListeners extends ConsumerWidget {
  const _RatingDeckListeners({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    ref.listen(ratingDeckProvider.select((s) => s.error), (prev, next) {
      if (next != null && next != prev) {
        showAppSnackBar(ScaffoldMessenger.of(context), l10n.ratingSubmitError);
      }
    });
    return child;
  }
}

class _RatingDeckBody extends ConsumerWidget {
  const _RatingDeckBody();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final deck = ref.watch(ratingDeckProvider);

    // First load in flight (and nothing to show yet).
    if (deck.loading && deck.topCard == null) {
      return const Center(child: CircularProgressIndicator());
    }
    // Load failed before any card arrived — offer a retry.
    if (deck.error != null && deck.topCard == null) {
      return _DeckMessage(
        icon: Icons.cloud_off,
        title: l10n.ratingDeckLoadError,
        action: FilledButton(
          onPressed: () => ref.read(ratingDeckProvider.notifier).refresh(),
          child: Text(l10n.ratingDeckRetry),
        ),
      );
    }
    // Every sourced card has been judged.
    if (deck.isExhausted) {
      return _DeckMessage(
        icon: Icons.done_all,
        title: l10n.ratingDeckEmptyTitle,
        body: l10n.ratingDeckEmptyBody,
      );
    }

    // Rating is gated on "listened enough": the verdict/star controls are locked
    // until the card's auto-preview has played past the threshold. Skip is always
    // allowed.
    final locked = !deck.topUnlocked;
    return Column(
      children: [
        Expanded(child: _CardStack(deck: deck)),
        RatingDeckControls(
          locked: locked,
          onDislike: () =>
              ref.read(ratingDeckProvider.notifier).rate(RatingVerdict.dislike),
          onLike: () =>
              ref.read(ratingDeckProvider.notifier).rate(RatingVerdict.like),
          onLove: () =>
              ref.read(ratingDeckProvider.notifier).rate(RatingVerdict.love),
          onSkip: () => ref.read(ratingDeckProvider.notifier).skip(),
          onStars: (deck.topCard == null || locked)
              ? null
              : () => _openStars(context, ref, deck.topCard!),
        ),
      ],
    );
  }

  /// Opens the 1–5 star sheet for [card]; selecting a value submits it (the
  /// notifier reconciles it with the swipe verdict into one rating).
  static Future<void> _openStars(
    BuildContext context,
    WidgetRef ref,
    CatalogEntry card,
  ) async {
    final stars = await showRatingStarsSheet(context, card);
    if (stars != null) {
      unawaited(ref.read(ratingDeckProvider.notifier).rateStars(stars));
    }
  }
}

/// The card stack: the next card peeks behind the swipeable top card so the deck
/// reads as a stack. The coach mark overlays the top card on first use.
class _CardStack extends ConsumerWidget {
  const _CardStack({required this.deck});

  final RatingDeckState deck;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final top = deck.topCard;
    final next = deck.nextCard;
    return Stack(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
          child: LayoutBuilder(
            builder: (context, constraints) {
              // A Tinder-style card is bounded and centred — never stretched
              // across a wide tablet/desktop viewport. Its height is exactly the
              // preview cover (16/11) plus a compact info strip, so there is no
              // dead space below the card; it shrinks to fit a short viewport.
              const maxWidth = 420.0;
              const infoHeight =
                  118.0; // ScoreCard title + composer + attribution
              var cardW = math.min(constraints.maxWidth, maxWidth);
              var cardH = cardW * 11 / 16 + infoHeight;
              if (cardH > constraints.maxHeight) {
                cardH = constraints.maxHeight;
                cardW = math.min(
                  cardW,
                  math.max(0, cardH - infoHeight) * 16 / 11,
                );
              }
              return Center(
                child: SizedBox(
                  width: cardW,
                  height: cardH,
                  child: Stack(
                    alignment: Alignment.center,
                    children: [
                      // Behind: a static peek of the next card, inset below.
                      if (next != null)
                        Positioned.fill(
                          child: Padding(
                            padding: const EdgeInsets.symmetric(horizontal: 12),
                            child: Transform.translate(
                              offset: const Offset(0, 14),
                              child: Opacity(
                                opacity: 0.55,
                                child: RatingCard(
                                  entry: next,
                                  interactive: false,
                                ),
                              ),
                            ),
                          ),
                        ),
                      // Front: the swipeable, interactive top card (keyed by id
                      // so a new top card resets the swipe animation). Swiping is
                      // enabled only once the card is unlocked (listened enough).
                      if (top != null)
                        Positioned.fill(
                          child: SwipeCard(
                            key: ValueKey(top.catalogId ?? top.id),
                            enabled: deck.topUnlocked,
                            onDislike: () => ref
                                .read(ratingDeckProvider.notifier)
                                .rate(RatingVerdict.dislike),
                            onLike: () => ref
                                .read(ratingDeckProvider.notifier)
                                .rate(RatingVerdict.like),
                            onLove: () => ref
                                .read(ratingDeckProvider.notifier)
                                .rate(RatingVerdict.love),
                            child: RatingCard(
                              entry: top,
                              onTapStars: deck.topUnlocked
                                  ? () =>
                                        _CardStackStars.open(context, ref, top)
                                  : null,
                              onPreviewProgress: top.catalogId == null
                                  ? null
                                  : (f) => ref
                                        .read(ratingDeckProvider.notifier)
                                        .markPreviewed(top.catalogId!, f),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              );
            },
          ),
        ),
        // The one-time coach mark covers the whole deck area (over the card).
        const _CoachMarkOverlay(),
      ],
    );
  }
}

/// Small helper to open the star sheet from within the card stack (keeps the
/// callback wiring above terse).
class _CardStackStars {
  static Future<void> open(
    BuildContext context,
    WidgetRef ref,
    CatalogEntry card,
  ) async {
    final stars = await showRatingStarsSheet(context, card);
    if (stars != null) {
      unawaited(ref.read(ratingDeckProvider.notifier).rateStars(stars));
    }
  }
}

/// The one-time coach mark (change: add-app-score-rating, task 5.1): a dismissible
/// scrim explaining the gestures, shown only until the user has seen it once
/// (persisted via [ratingCoachMarkProvider]).
class _CoachMarkOverlay extends ConsumerWidget {
  const _CoachMarkOverlay();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final seen = ref.watch(ratingCoachMarkProvider);
    // `null` = still loading the flag (show nothing yet); `true` = already seen.
    if (seen != false) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    return Positioned.fill(
      child: GestureDetector(
        onTap: () => ref.read(ratingCoachMarkProvider.notifier).markSeen(),
        child: ColoredBox(
          color: Colors.black.withValues(alpha: 0.72),
          // Centre the hint when it fits, and let it scroll on a short (landscape)
          // viewport instead of overflowing.
          child: LayoutBuilder(
            builder: (context, constraints) => SingleChildScrollView(
              child: ConstrainedBox(
                constraints: BoxConstraints(minHeight: constraints.maxHeight),
                child: Center(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24,
                      vertical: 20,
                    ),
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 420),
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const Icon(
                            Icons.swipe,
                            size: 46,
                            color: CymbraColors.primary,
                          ),
                          const SizedBox(height: 14),
                          Text(
                            l10n.ratingCoachTitle,
                            textAlign: TextAlign.center,
                            style: const TextStyle(
                              color: CymbraColors.onSurface,
                              fontSize: 19,
                              fontWeight: FontWeight.w800,
                            ),
                          ),
                          const SizedBox(height: 10),
                          Text(
                            l10n.ratingCoachBody,
                            textAlign: TextAlign.center,
                            style: const TextStyle(
                              color: CymbraColors.onSurfaceVariant,
                              fontSize: 14.5,
                              height: 1.3,
                            ),
                          ),
                          const SizedBox(height: 18),
                          FilledButton(
                            onPressed: () => ref
                                .read(ratingCoachMarkProvider.notifier)
                                .markSeen(),
                            child: Text(l10n.ratingCoachDismiss),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// A centered icon + title (+ optional body / action) for the deck's empty and
/// error states.
class _DeckMessage extends StatelessWidget {
  const _DeckMessage({
    required this.icon,
    required this.title,
    this.body,
    this.action,
  });

  final IconData icon;
  final String title;
  final String? body;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 56, color: CymbraColors.onSurfaceVariant),
            const SizedBox(height: 20),
            Text(
              title,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 20,
                fontWeight: FontWeight.w800,
              ),
            ),
            if (body != null) ...[
              const SizedBox(height: 12),
              Text(
                body!,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 15,
                  height: 1.35,
                ),
              ),
            ],
            if (action != null) ...[const SizedBox(height: 24), action!],
          ],
        ),
      ),
    );
  }
}
