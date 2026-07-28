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
import 'in_card_preview.dart';
import 'score_card.dart';

/// The rating deck's card visual (change: add-app-score-rating). Reuses the
/// shared [ScoreCard] for the cover / title / composer / attribution, and adds
/// the deck affordances: a Play control that plays the read-only preview **in the
/// card** (the cover region becomes the scrolling game-score render), and a tap
/// target that opens the 1–5 star rating. When [interactive] is false (the peeked
/// next card behind the top one) the affordances are omitted and it ignores
/// pointers so the swipe/tap always reaches the top card.
class RatingCard extends StatefulWidget {
  const RatingCard({
    super.key,
    required this.entry,
    this.onTapStars,
    this.interactive = true,
  });

  final CatalogEntry entry;

  /// Opens the star rating (tapping the card). Null when non-interactive.
  final VoidCallback? onTapStars;

  final bool interactive;

  @override
  State<RatingCard> createState() => _RatingCardState();
}

class _RatingCardState extends State<RatingCard> {
  /// Whether the in-card read-only preview is playing over the cover region.
  bool _previewing = false;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final entry = widget.entry;
    if (!widget.interactive) {
      // Decorative peek card: no affordances, and ignores pointers.
      return IgnorePointer(
        child: ScoreCard(entry: entry, onTap: () {}),
      );
    }
    final canPreview = entry.catalogId != null;
    return LayoutBuilder(
      builder: (context, constraints) {
        // The preview sits over the card's cover region (ScoreCard uses a 16/11
        // cover), leaving the title/composer visible below it.
        final coverHeight = constraints.maxWidth * 11 / 16;
        return Stack(
          children: [
            ScoreCard(
              entry: entry,
              onTap: widget.onTapStars ?? () {},
              action: (canPreview && !_previewing)
                  ? _PreviewButton(
                      onPressed: () => setState(() => _previewing = true),
                      tooltip: l10n.ratingPreview,
                    )
                  : null,
            ),
            if (_previewing && canPreview)
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
                    onClose: () => setState(() => _previewing = false),
                  ),
                ),
              ),
          ],
        );
      },
    );
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
