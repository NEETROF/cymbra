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

import '../l10n/gen/app_localizations.dart';
import '../screens/lesson_player_screen.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../state/placement_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The placement offer on the Learn door (change: add-music-placement-test,
/// D5): shown while a placement course is published, none was taken and no
/// lesson was completed on this device; "Later" hides it for the session.
/// Opening it runs the placement course in the lesson player.
class PlacementOfferCard extends ConsumerWidget {
  const PlacementOfferCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final course = ref.watch(placementCourseProvider).valueOrNull;
    final placement = ref.watch(placementProvider);
    final completed = ref.watch(
      courseCompletionProvider.select((s) => s.completed),
    );
    final dismissed = ref.watch(placementOfferDismissedProvider);
    if (course == null ||
        !placement.loaded ||
        placement.taken ||
        completed.isNotEmpty ||
        dismissed) {
      return const SizedBox.shrink();
    }
    final l10n = AppLocalizations.of(context);
    return Container(
      key: const Key('placement-offer'),
      margin: const EdgeInsets.fromLTRB(16, 8, 16, 4),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CymbraColors.primary.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: CymbraColors.primary.withValues(alpha: 0.35)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.explore, color: CymbraColors.primary),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  l10n.placementOfferTitle,
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 16,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            l10n.placementOfferBody,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 13,
            ),
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 4,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              FilledButton(
                key: const Key('placement-offer-start'),
                onPressed: () => openLessonPlayer(context, course.id),
                child: Text(l10n.placementOfferStart),
              ),
              TextButton(
                key: const Key('placement-offer-later'),
                onPressed: () => ref
                    .read(placementOfferDismissedProvider.notifier)
                    .dismiss(),
                child: Text(
                  l10n.placementOfferLater,
                  style: const TextStyle(color: CymbraColors.onSurfaceVariant),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// The path header's way into the placement, always reachable (D5): a text
/// button on the Learn door, an icon on the path screen's app bar. Nothing
/// when no placement course is published.
class PlacementEntryButton extends ConsumerWidget {
  const PlacementEntryButton({super.key, this.compact = false});

  final bool compact;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final course = ref.watch(placementCourseProvider).valueOrNull;
    if (course == null) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    void open() => openLessonPlayer(context, course.id);
    if (compact) {
      return IconButton(
        key: const Key('placement-entry'),
        tooltip: l10n.placementOfferStart,
        icon: const Icon(Icons.explore_outlined),
        onPressed: open,
      );
    }
    return TextButton.icon(
      key: const Key('placement-entry'),
      onPressed: open,
      icon: const Icon(Icons.explore_outlined, size: 18),
      label: Text(
        l10n.placementOfferStart,
        style: const TextStyle(color: CymbraColors.secondary, fontSize: 12.5),
      ),
    );
  }
}
