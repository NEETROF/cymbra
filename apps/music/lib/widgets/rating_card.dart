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

import '../l10n/gen/app_localizations.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import 'score_card.dart';

/// The rating deck's card visual (change: add-app-score-rating). Reuses the
/// shared [ScoreCard] for the cover / title / composer / attribution, and adds
/// the deck affordances: a Play control that opens the read-only preview, and a
/// tap target that opens the 1–5 star rating. When [interactive] is false (the
/// peeked next card behind the top one) the affordances are omitted.
class RatingCard extends StatelessWidget {
  const RatingCard({
    super.key,
    required this.entry,
    this.onTapStars,
    this.onPreview,
    this.interactive = true,
  });

  final CatalogEntry entry;

  /// Opens the star rating (tapping the card). Null when non-interactive.
  final VoidCallback? onTapStars;

  /// Opens the read-only preview (the card's Play control). Null when
  /// non-interactive.
  final VoidCallback? onPreview;

  final bool interactive;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final card = ScoreCard(
      entry: entry,
      onTap: onTapStars ?? () {},
      action: interactive && onPreview != null
          ? _PreviewButton(onPressed: onPreview!, tooltip: l10n.ratingPreview)
          : null,
    );
    // A non-interactive card (the peeked next card behind the top one) is purely
    // decorative: it must not absorb any pointer, so the swipe/tap always reaches
    // the top card.
    return interactive ? card : IgnorePointer(child: card);
  }
}

/// A circular Play button overlaid on the card cover, opening the preview.
class _PreviewButton extends StatelessWidget {
  const _PreviewButton({required this.onPressed, required this.tooltip});

  final VoidCallback onPressed;
  final String tooltip;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.black.withValues(alpha: 0.45),
      shape: const CircleBorder(),
      clipBehavior: Clip.antiAlias,
      child: IconButton(
        icon: const Icon(Icons.play_arrow, color: Colors.white),
        tooltip: tooltip,
        onPressed: onPressed,
      ),
    );
  }
}

/// Opens the 1–5 star rating sheet for [entry] and resolves to the chosen star
/// value (1..5), or `null` if the user dismissed it without choosing. The caller
/// (the deck notifier) reconciles the stars with the swipe verdict into one
/// rating (design D2).
Future<int?> showRatingStarsSheet(BuildContext context, CatalogEntry entry) {
  return showModalBottomSheet<int>(
    context: context,
    backgroundColor: CymbraColors.surfaceContainerHigh,
    shape: const RoundedRectangleBorder(
      borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
    ),
    builder: (sheetContext) => _StarsSheet(entry: entry),
  );
}

class _StarsSheet extends StatelessWidget {
  const _StarsSheet({required this.entry});

  final CatalogEntry entry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(24, 20, 24, 28),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              l10n.ratingStarsSheetTitle,
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 18,
                fontWeight: FontWeight.w800,
              ),
            ),
            const SizedBox(height: 6),
            Text(
              entry.title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 14,
              ),
            ),
            const SizedBox(height: 20),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                for (var star = 1; star <= 5; star++)
                  IconButton(
                    key: Key('rating-star-$star'),
                    iconSize: 40,
                    icon: const Icon(
                      Icons.star_border,
                      color: CymbraColors.primary,
                    ),
                    onPressed: () => Navigator.of(context).pop(star),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
