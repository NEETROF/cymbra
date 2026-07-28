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
import 'difficulty_badge.dart';
import 'in_card_preview.dart';
import 'score_card.dart';

/// The rating deck's card visual (change: add-app-score-rating). Reuses the
/// shared [ScoreCard] for the title / composer / attribution, but the cover region
/// **auto-plays** the read-only game-score preview (the notation scrolls and
/// sounds as soon as the card is shown — no Play button). Tapping the card opens
/// the 1–5 star rating. When [interactive] is false (the peeked next card behind
/// the top one) the preview and affordances are omitted and it ignores pointers so
/// the swipe/tap always reaches the top card.
class RatingCard extends StatelessWidget {
  const RatingCard({
    super.key,
    required this.entry,
    this.onTapStars,
    this.onPreviewProgress,
    this.interactive = true,
  });

  final CatalogEntry entry;

  /// Opens the star rating (tapping the card). Null when non-interactive.
  final VoidCallback? onTapStars;

  /// Forwards the auto-preview's playback fraction (0..1) so the deck can unlock
  /// rating once enough of the score has been heard.
  final ValueChanged<double>? onPreviewProgress;

  final bool interactive;

  @override
  Widget build(BuildContext context) {
    if (!interactive) {
      // Decorative peek card: static cover, no preview, and ignores pointers.
      return IgnorePointer(
        child: ScoreCard(entry: entry, onTap: () {}),
      );
    }
    final l10n = AppLocalizations.of(context);
    final canPreview = entry.catalogId != null;
    return LayoutBuilder(
      builder: (context, constraints) {
        // The preview sits over the card's cover region (ScoreCard uses a 16/11
        // cover), leaving the title/composer visible below it.
        final coverHeight = constraints.maxWidth * 11 / 16;
        return Stack(
          children: [
            ScoreCard(entry: entry, onTap: onTapStars ?? () {}),
            if (canPreview)
              Positioned(
                top: 0,
                left: 0,
                right: 0,
                height: coverHeight,
                child: ClipRRect(
                  borderRadius: const BorderRadius.vertical(
                    top: Radius.circular(18),
                  ),
                  child: InCardPreview(
                    catalogId: entry.catalogId!,
                    onProgress: onPreviewProgress,
                  ),
                ),
              ),
            // Keep the difficulty badge visible over the auto-playing preview.
            Positioned(
              top: 10,
              left: 10,
              child: DifficultyBadge(level: entry.level, l10n: l10n),
            ),
          ],
        );
      },
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
