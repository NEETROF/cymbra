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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../layout/device_class.dart';
import '../state/post_play_rating_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The rating affordance shown after playing a piece (change:
/// add-post-play-rating-prompt): one compact row of 1–5 stars, which becomes a
/// thanks line once submitted.
///
/// Deliberately **not** gated on any listening threshold, unlike the deck's card:
/// the user has just performed the piece, which is far stronger evidence than a
/// preview. Both surfaces — the summary modal and the early-exit sheet — use this
/// same widget, so they can never diverge.
///
/// Being shown costs the score nothing: only a rating or an explicit refusal (the
/// "not this one" button) retires it from future prompts, so closing a summary to
/// leave the player does not silently burn the chance to rate the piece. Callers
/// only mount it when `postPlayRatingEligibleProvider` says so.
class PostPlayRatingRow extends ConsumerWidget {
  const PostPlayRatingRow({super.key, this.compact = false, this.onDecline});

  /// Tighter spacing and smaller stars, for the summary modal on a phone.
  final bool compact;

  /// Run after the user explicitly refuses to rate this score (the refusal itself
  /// is recorded here). The sheet passes a pop; the summary passes nothing, since
  /// the refusal simply makes the row disappear.
  final VoidCallback? onDecline;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final data = ref.watch(postPlayRatingProvider).valueOrNull;
    final submitted = data?.submittedStars;
    final failed = data?.failed ?? false;
    final gap = compact ? 4.0 : 8.0;

    if (submitted != null && !failed) {
      return Padding(
        padding: EdgeInsets.symmetric(vertical: gap),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.check_circle_outline,
              size: 18,
              color: CymbraColors.tertiary,
            ),
            const SizedBox(width: 8),
            Flexible(
              child: Text(
                l10n.postPlayRatingThanks,
                style: const TextStyle(
                  fontSize: 13,
                  color: CymbraColors.onSurfaceVariant,
                ),
              ),
            ),
          ],
        ),
      );
    }

    final stars = [
      for (var star = 1; star <= 5; star++)
        IconButton(
          key: Key('post-play-star-$star'),
          tooltip: l10n.postPlayRatingStarTooltip(star),
          iconSize: compact ? 20 : 34,
          // IconButton reserves a 48×48 Material tap target. In the summary modal
          // that alone is taller than the whole row can afford on a phone-landscape
          // viewport, and `constraints` does NOT override it — only shrinking the
          // tap target does. 28×28 stays comfortably tappable for a star.
          style: compact
              ? IconButton.styleFrom(
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  minimumSize: const Size(28, 28),
                  padding: EdgeInsets.zero,
                )
              : null,
          visualDensity: compact
              ? VisualDensity.compact
              : VisualDensity.standard,
          icon: const Icon(Icons.star_border, color: CymbraColors.primary),
          onPressed: () =>
              unawaited(ref.read(postPlayRatingProvider.notifier).submit(star)),
        ),
    ];

    final label = Text(
      l10n.postPlayRatingPrompt,
      textAlign: TextAlign.center,
      style: TextStyle(
        fontSize: compact ? 12 : 13,
        color: CymbraColors.onSurfaceVariant,
      ),
    );

    final decline = TextButton(
      key: const Key('post-play-rating-skip'),
      style: compact
          ? TextButton.styleFrom(
              visualDensity: VisualDensity.compact,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              minimumSize: const Size(0, 28),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            )
          : null,
      onPressed: () {
        final id = data?.catalogId;
        if (id != null) {
          unawaited(ref.read(postPlayRatingProvider.notifier).decline(id));
        }
        onDecline?.call();
      },
      child: Text(l10n.postPlayRatingSkip),
    );

    final failure = failed
        ? Text(
            l10n.postPlayRatingFailed,
            textAlign: TextAlign.center,
            style: const TextStyle(fontSize: 12, color: CymbraColors.error),
          )
        : null;

    return Padding(
      padding: EdgeInsets.symmetric(vertical: gap),
      child: compact
          // Summary modal: two tight lines, not four. The modal caps at 420 px
          // wide, so the question and the five stars cannot share a line — but the
          // stars and the refusal can, and every default tap target is shrunk. What
          // matters is the total height: the statistics above (the headline
          // percentage first of all) need the room on a phone-landscape viewport.
          ? Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                label,
                // Wrap, not Row: "Don't ask again" is a long label in several
                // languages, and on a narrow portrait window the stars plus the
                // button exceed the modal's inner width. Wrapping to a second line
                // is a graceful degradation; a Row would throw an overflow.
                Wrap(
                  alignment: WrapAlignment.center,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  spacing: 4,
                  children: [...stars, decline],
                ),
                ?failure,
              ],
            )
          : Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                label,
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: stars,
                ),
                ?failure,
                // The escape hatch. Because being shown no longer retires the
                // score, this is the ONLY way (short of rating) to stop being
                // asked about this piece — so it is on every surface.
                decline,
              ],
            ),
    );
  }
}

/// Offer the rating on the way out of the player (change:
/// add-post-play-rating-prompt).
///
/// Non-blocking by construction: the sheet has no "stay" action, every dismissal
/// path resolves it, and the caller leaves the player whatever happens. It never
/// asks the user to confirm the exit.
Future<void> showPostPlayRatingSheet(
  BuildContext context,
) => showModalBottomSheet<void>(
  context: context,
  isDismissible: true,
  showDragHandle: true,
  backgroundColor: CymbraColors.surfaceContainerLow,
  builder: (context) => SafeArea(
    child: Padding(
      padding: const EdgeInsets.fromLTRB(24, 4, 24, 20),
      // A bottom sheet is capped at 9/16 of the screen height. On a phone lying
      // down — the player's only orientation — that is barely 200 px, and the
      // home-indicator inset the SafeArea gives up takes it under what the
      // full-size row needs. So a phone gets the same compact row as the summary,
      // and the content scrolls rather than painting an overflow stripe if a
      // future string or a large text scale still does not fit.
      child: SingleChildScrollView(
        child: PostPlayRatingRow(
          compact: context.isPhoneLayout,
          // Refusing from the sheet also leaves the player, like every other
          // dismissal path. Dismissing the sheet WITHOUT pressing it (barrier,
          // back, drag) records nothing: the user just wanted out.
          onDecline: () => Navigator.of(context).pop(),
        ),
      ),
    ),
  ),
);
