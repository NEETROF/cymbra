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
/// Rendering it marks the score as offered, retiring it from any future prompt
/// whether or not the user rates. Callers only mount it when
/// `postPlayRatingEligibleProvider` says so.
class PostPlayRatingRow extends ConsumerStatefulWidget {
  const PostPlayRatingRow({super.key, this.compact = false});

  /// Tighter spacing and smaller stars, for the summary modal on a phone.
  final bool compact;

  @override
  ConsumerState<PostPlayRatingRow> createState() => _PostPlayRatingRowState();
}

class _PostPlayRatingRowState extends ConsumerState<PostPlayRatingRow> {
  @override
  void initState() {
    super.initState();
    // Being shown IS the offer: record it now, so a dismissal retires the score
    // exactly like a rating does. Fire-and-forget — the UI never awaits it.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final id = ref.read(postPlayRatingProvider).valueOrNull?.catalogId;
      if (id == null) return;
      unawaited(ref.read(postPlayRatingProvider.notifier).markOffered(id));
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final data = ref.watch(postPlayRatingProvider).valueOrNull;
    final submitted = data?.submittedStars;
    final failed = data?.failed ?? false;
    final gap = widget.compact ? 4.0 : 8.0;

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

    return Padding(
      padding: EdgeInsets.symmetric(vertical: gap),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            l10n.postPlayRatingPrompt,
            textAlign: TextAlign.center,
            style: const TextStyle(
              fontSize: 13,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              for (var star = 1; star <= 5; star++)
                IconButton(
                  key: Key('post-play-star-$star'),
                  visualDensity: widget.compact
                      ? VisualDensity.compact
                      : VisualDensity.standard,
                  iconSize: widget.compact ? 26 : 34,
                  tooltip: l10n.postPlayRatingStarTooltip(star),
                  icon: const Icon(
                    Icons.star_border,
                    color: CymbraColors.primary,
                  ),
                  onPressed: () => unawaited(
                    ref.read(postPlayRatingProvider.notifier).submit(star),
                  ),
                ),
            ],
          ),
          // A failed submission says so in the user's language and leaves the
          // stars tappable for a retry — never a transport error string.
          if (failed)
            Text(
              l10n.postPlayRatingFailed,
              textAlign: TextAlign.center,
              style: const TextStyle(fontSize: 12, color: CymbraColors.error),
            ),
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
Future<void> showPostPlayRatingSheet(BuildContext context) =>
    showModalBottomSheet<void>(
      context: context,
      isDismissible: true,
      showDragHandle: true,
      backgroundColor: CymbraColors.surfaceContainerLow,
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(24, 4, 24, 20),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const PostPlayRatingRow(),
              TextButton(
                key: const Key('post-play-rating-skip'),
                onPressed: () => Navigator.of(context).pop(),
                child: Text(AppLocalizations.of(context).postPlayRatingSkip),
              ),
            ],
          ),
        ),
      ),
    );
