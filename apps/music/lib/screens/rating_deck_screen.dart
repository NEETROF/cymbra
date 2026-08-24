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
import '../state/coaching_notifier.dart';
import '../state/drums_access.dart';
import '../state/piano_catalog.dart';
import '../state/rating_deck_notifier.dart';
import '../state/score_catalog.dart';
import '../state/selected_kit.dart';
import '../state/selected_piano.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/coach_mark.dart';
import '../widgets/rating_card.dart';
import '../widgets/rating_deck_controls.dart';
import '../widgets/sound_selector_field.dart';
import '../widgets/swipe_card.dart';
import 'auth/account_menu.dart';

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
        // Swap the instrument sound the card auto-preview plays with — a compact
        // combobox so the moderator can audition scores with any catalog sound.
        //
        // It follows the family of the card ON SCREEN (change:
        // add-drum-audio-channel): a drum card is auditioned through the kit
        // memory, everything else through the piano memory. Bound to the piano
        // alone, the control offered a knob that changes nothing about what is
        // being heard — and would have silently retuned the piano while the
        // moderator was listening to a groove.
        actions: [
          // The account control is the curator standing pill (change: add-
          // curation-rewards) — it replaces the plain person icon; opens the
          // account menu (→ profile).
          const AccountMenu(),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
            child: SizedBox(
              // Wide enough that the "Add a SoundFont…" item fits the dropdown
              // menu (its width follows the field) without truncating.
              width: 260,
              child: Builder(
                builder: (context) {
                  final percussion =
                      ref.watch(
                        ratingDeckProvider.select((s) => s.topCard?.instrument),
                      ) ==
                      ScoreInstrument.percussion;
                  return SoundSelectorField(
                    dense: true,
                    family: percussion
                        ? SoundFamily.percussion
                        : SoundFamily.keyboard,
                    value: percussion
                        ? ref.watch(selectedKitProvider)
                        : ref.watch(selectedPianoProvider),
                    onChanged: (id) => percussion
                        ? ref.read(selectedKitProvider.notifier).select(id)
                        : ref.read(selectedPianoProvider.notifier).select(id),
                  );
                },
              ),
            ),
          ),
        ],
      ),
      // The deck's side effects (a failed submit → snackbar) live in a dedicated
      // listener widget near the top of the subtree (architecture rule 4).
      body: const _RatingDeckListeners(
        child: SafeArea(child: _RatingDeckBody()),
      ),
    );
  }
}

/// The rater's own choice of what to be dealt: everything, piano, or drums.
///
/// Offered only to the drum audience — with a single family in the corpus the
/// control would be three ways of saying the same thing (change:
/// add-drums-access, which is also what makes the drum option meaningful: the
/// backend serves percussion rows to that audience only).
class _DeckInstrumentFilter extends ConsumerWidget {
  const _DeckInstrumentFilter();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (!ref.watch(drumsEnabledProvider)) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    final selected = ref.watch(ratingDeckProvider.select((s) => s.instrument));
    final notifier = ref.read(ratingDeckProvider.notifier);

    Widget choice(String label, ScoreInstrument? value) => Padding(
      padding: const EdgeInsets.symmetric(horizontal: 4),
      child: ChoiceChip(
        label: Text(label),
        selected: selected == value,
        onSelected: (_) => notifier.setInstrument(value),
        showCheckmark: false,
        labelStyle: TextStyle(
          color: selected == value
              ? CymbraColors.background
              : CymbraColors.onSurfaceVariant,
          fontWeight: FontWeight.w600,
        ),
        backgroundColor: CymbraColors.surfaceContainerLowest,
        selectedColor: CymbraColors.secondary,
        side: BorderSide(
          color: selected == value
              ? CymbraColors.secondary
              : CymbraColors.outlineVariant,
        ),
      ),
    );

    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 0),
      child: Wrap(
        alignment: WrapAlignment.center,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          choice(l10n.scoreHubAny, null),
          choice(l10n.instrumentKeyboard, ScoreInstrument.keyboard),
          choice(l10n.instrumentDrums, ScoreInstrument.percussion),
        ],
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
    // Immediate "+N" curator-points cue on a rating that earned coverage points
    // (change: add-curation-rewards). The seq bump fires it on repeat awards.
    ref.listen(ratingDeckProvider.select((s) => s.pointsCueSeq), (prev, next) {
      if (prev == null || next == prev) return;
      final points = ref.read(ratingDeckProvider).lastPointsAwarded;
      if (points != null && points > 0) {
        showAppSnackBar(
          ScaffoldMessenger.of(context),
          l10n.ratingPointsCue(points),
        );
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

    // The instrument chooser sits ABOVE whatever the body shows — including
    // the empty and error states. A filter that disappears the moment it
    // empties the deck is a trap: the way out of "nothing to rate" is the
    // control that got you there.
    Widget framed(Widget content) => Column(
      children: [
        const _DeckInstrumentFilter(),
        Expanded(child: content),
      ],
    );

    // First load in flight (and nothing to show yet).
    if (deck.loading && deck.topCard == null) {
      return framed(const Center(child: CircularProgressIndicator()));
    }
    // Load failed before any card arrived — offer a retry.
    if (deck.error != null && deck.topCard == null) {
      return framed(
        _DeckMessage(
          icon: Icons.cloud_off,
          title: l10n.ratingDeckLoadError,
          action: FilledButton(
            onPressed: () => ref.read(ratingDeckProvider.notifier).refresh(),
            child: Text(l10n.ratingDeckRetry),
          ),
        ),
      );
    }
    // Every sourced card has been judged.
    if (deck.isExhausted) {
      return framed(
        _DeckMessage(
          icon: Icons.done_all,
          title: l10n.ratingDeckEmptyTitle,
          body: l10n.ratingDeckEmptyBody,
        ),
      );
    }

    // Rating is gated on "listened enough": the verdict/star controls are locked
    // until the card's auto-preview has played past the threshold. Skip is always
    // allowed.
    final locked = !deck.topUnlocked;
    RatingDeckControls controls(Axis axis) => RatingDeckControls(
      locked: locked,
      axis: axis,
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
    );
    // The "listen before rating" caption stays visible (in both layouts) while
    // locked, so the gate is always explained — not just a tooltip.
    final Widget cardArea = Column(
      children: [
        Expanded(child: _CardStack(deck: deck)),
        if (locked) const _ListenHint(),
      ],
    );
    // Decide the layout from the real available space: a short landscape viewport
    // (phone held sideways) moves the controls to a side rail so the card keeps
    // the full height; with room (portrait/tablet/desktop) they sit in the usual
    // bottom bar.
    return LayoutBuilder(
      builder: (context, c) {
        final sideRail = c.maxWidth > c.maxHeight && c.maxHeight < 460;
        return framed(
          sideRail
              ? Row(
                  children: [
                    Expanded(child: cardArea),
                    controls(Axis.vertical),
                  ],
                )
              : Column(
                  children: [
                    Expanded(child: cardArea),
                    controls(Axis.horizontal),
                  ],
                ),
        );
      },
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

/// The one-time swipe-gesture hint (change: add-app-score-rating, task 5.1),
/// now delivered through the **shared** coaching mechanism (change:
/// add-welcome-onboarding, D4) rather than its own overlay: same look as every
/// other first-use hint, one "seen" store, dismissible, and never blocking the
/// deck underneath for longer than a tap.
class _CoachMarkOverlay extends ConsumerWidget {
  const _CoachMarkOverlay();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final coaching = ref.watch(coachingProvider);
    // Nothing is shown until the flags are known, so a returning user never
    // sees it flash.
    if (!coaching.shouldShow(CoachHint.ratingDeck)) {
      return const SizedBox.shrink();
    }
    final l10n = AppLocalizations.of(context);
    void dismiss() =>
        ref.read(coachingProvider.notifier).markSeen(CoachHint.ratingDeck);
    return Positioned.fill(
      child: CoachMarkOverlay(
        title: l10n.ratingCoachTitle,
        body: l10n.ratingCoachBody,
        nextLabel: l10n.ratingCoachDismiss,
        onNext: dismiss,
      ),
    );
  }
}

/// The persistent "listen before rating" caption, shown under the card while the
/// rating is locked — visible in every layout (not hidden in a tooltip) so the
/// gate is always explained.
class _ListenHint extends StatelessWidget {
  const _ListenHint();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      key: const Key('rating-locked-hint'),
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(
            Icons.headphones,
            size: 18,
            color: CymbraColors.onSurfaceVariant,
          ),
          const SizedBox(width: 8),
          Flexible(
            child: Text(
              l10n.ratingLockedHint,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 13.5,
              ),
            ),
          ),
        ],
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
