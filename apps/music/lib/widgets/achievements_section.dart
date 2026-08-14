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
import '../services/achievements_service.dart';
import '../state/achievements_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The signed-in user's **Achievements**, rendered inline in their profile
/// (change: add-achievement-badges). Badges used to be a grid inside the curator
/// section, which promised an achievement system to anyone who never rated a
/// score; they now have their own section spanning every family — playing,
/// consistency, ranking, contribution, curation, learning.
///
/// Nothing here enumerates badges. Families, labels, descriptions, thresholds and
/// tiers all come from the server registry, which is what lets a new badge ship
/// without an app release. The only app-side copy is the section chrome (the
/// title, the family names, "12/25") and the iconography.
///
/// Returns a `Column`, so it drops into the profile's scroll view (no Scaffold or
/// ListView of its own).
class AchievementsSection extends ConsumerWidget {
  const AchievementsSection({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(achievementsProvider);
    return async.when(
      loading: () => const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(child: CircularProgressIndicator()),
      ),
      // An achievements read failure degrades quietly with a retry — it must
      // never take down the rest of the profile.
      error: (_, _) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 16),
        child: Row(
          children: [
            const Icon(Icons.cloud_off, color: CymbraColors.onSurfaceVariant),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                l10n.achievementsLoadError,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            ),
            TextButton(
              onPressed: () =>
                  ref.read(achievementsProvider.notifier).refresh(),
              child: Text(l10n.curatorRetry),
            ),
          ],
        ),
      ),
      data: (view) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Clear the "new" markers for the next visit (side effect isolated in
          // its own widget — architecture rule 4).
          _AchievementsSeenListener(view: view),
          _SectionTitle(l10n.achievementsTitle),
          for (final family in view.families) ...[
            const SizedBox(height: 16),
            _FamilySection(family: family, languageCode: view.languageCode),
          ],
        ],
      ),
    );
  }
}

/// Isolates the "mark achievements seen" side effect (architecture rule 4): once
/// the grid has loaded, record the newest earned moment so the markers clear on
/// the next visit — not while the user is still looking at them.
class _AchievementsSeenListener extends ConsumerStatefulWidget {
  const _AchievementsSeenListener({required this.view});

  final AchievementsView view;

  @override
  ConsumerState<_AchievementsSeenListener> createState() =>
      _AchievementsSeenListenerState();
}

class _AchievementsSeenListenerState
    extends ConsumerState<_AchievementsSeenListener> {
  @override
  void initState() {
    super.initState();
    final at = widget.view.newestEarnedAt;
    if (at != null) {
      // Defer past the first frame — never mutate providers during build.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        ref.read(achievementsSeenProvider.notifier).markSeen(at);
      });
    }
  }

  @override
  Widget build(BuildContext context) => const SizedBox.shrink();
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Text(
    text,
    style: const TextStyle(
      color: CymbraColors.primary,
      fontSize: 15,
      fontWeight: FontWeight.w800,
      letterSpacing: 0.5,
    ),
  );
}

/// One family: its name, how many of its badges are earned, and its tiles.
class _FamilySection extends StatelessWidget {
  const _FamilySection({required this.family, required this.languageCode});

  final AchievementFamilyView family;
  final String languageCode;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Column(
      key: Key('achievement-family-${family.family}'),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(
              _familyIcon(family.family),
              size: 16,
              color: CymbraColors.onSurfaceVariant,
            ),
            const SizedBox(width: 6),
            Text(
              familyLabel(l10n, family.family),
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 13,
                fontWeight: FontWeight.w700,
              ),
            ),
            const Spacer(),
            Text(
              l10n.achievementFamilyCount(
                family.earnedCount,
                family.totalCount,
              ),
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 12,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
            maxCrossAxisExtent: 180,
            mainAxisSpacing: 10,
            crossAxisSpacing: 10,
            childAspectRatio: 1.05,
          ),
          itemCount: family.tiles.length,
          itemBuilder: (context, i) =>
              _BadgeTile(tile: family.tiles[i], languageCode: languageCode),
        ),
      ],
    );
  }
}

/// One tile: a standalone badge, or a whole track collapsed to its current tier.
class _BadgeTile extends StatelessWidget {
  const _BadgeTile({required this.tile, required this.languageCode});

  final AchievementTileView tile;
  final String languageCode;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final earned = tile.earned;
    final color = earned ? CymbraColors.primary : CymbraColors.outline;
    return InkWell(
      key: Key('achievement-tile-${tile.id}'),
      borderRadius: BorderRadius.circular(12),
      onTap: () => showAchievementDetail(context, tile, languageCode),
      child: Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          color: CymbraColors.surfaceContainerHigh,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: earned
                ? CymbraColors.primary.withValues(alpha: 0.5)
                : Colors.transparent,
          ),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Stack(
              clipBehavior: Clip.none,
              children: [
                Icon(_familyIcon(tile.badge.family), color: color, size: 28),
                if (!earned)
                  const Positioned(
                    right: -6,
                    bottom: -2,
                    child: Icon(
                      Icons.lock_outline,
                      size: 14,
                      color: CymbraColors.outline,
                    ),
                  ),
                if (tile.isNew)
                  Positioned(
                    right: -10,
                    top: -8,
                    child: Container(
                      key: const Key('achievement-new-marker'),
                      padding: const EdgeInsets.symmetric(
                        horizontal: 5,
                        vertical: 1,
                      ),
                      decoration: BoxDecoration(
                        color: CymbraColors.primary,
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: Text(
                        l10n.achievementNew,
                        style: const TextStyle(
                          color: CymbraColors.background,
                          fontSize: 9,
                          fontWeight: FontWeight.w800,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              tile.badge.labelIn(languageCode),
              textAlign: TextAlign.center,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                color: earned
                    ? CymbraColors.onSurface
                    : CymbraColors.onSurfaceVariant,
                fontSize: 12.5,
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 6),
            // A locked tile shows how far along the user is — "12/25" with a bar
            // — rather than only the target it has yet to reach.
            if (tile.next != null)
              _Progress(badge: tile.next!)
            else
              Text(
                l10n.achievementEarned,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 10.5,
                ),
              ),
          ],
        ),
      ),
    );
  }
}

/// The "12/25" indicator for the next rung still to earn.
class _Progress extends StatelessWidget {
  const _Progress({required this.badge});

  final AchievementBadgeView badge;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Column(
      children: [
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            key: Key('achievement-progress-${badge.key}'),
            value: badge.progress,
            minHeight: 5,
            backgroundColor: CymbraColors.background,
            valueColor: const AlwaysStoppedAnimation(CymbraColors.primary),
          ),
        ),
        const SizedBox(height: 3),
        Text(
          l10n.achievementProgress(badge.value, badge.threshold),
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 10.5,
          ),
        ),
      ],
    );
  }
}

/// Opens the badge detail sheet: what it takes, where the user stands, when they
/// earned it, and — for a track — every tier with the earned ones marked.
Future<void> showAchievementDetail(
  BuildContext context,
  AchievementTileView tile,
  String languageCode,
) => showModalBottomSheet<void>(
  context: context,
  backgroundColor: CymbraColors.surfaceContainerHigh,
  showDragHandle: true,
  // The app is landscape-only, so the sheet's height budget is the SHORT side of
  // the screen — 402pt on an iPhone. The default bottom-sheet cap (9/16 of that)
  // cannot hold a multi-tier ladder, which overflowed. Let it use the full
  // height; the sheet still sizes to its content when that content is short.
  isScrollControlled: true,
  builder: (_) =>
      AchievementDetailSheet(tile: tile, languageCode: languageCode),
);

/// The badge detail view. Public so a test can pump it directly.
class AchievementDetailSheet extends StatelessWidget {
  const AchievementDetailSheet({
    required this.tile,
    required this.languageCode,
    super.key,
  });

  final AchievementTileView tile;
  final String languageCode;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    // The rung the sheet describes: what you are working toward if anything is
    // left, else the top of the ladder you completed.
    final focus = tile.target;
    final earnedAt = tile.badge.earnedAt;
    return SafeArea(
      // A long ladder (six tiers) is taller than the landscape viewport can show,
      // so the content scrolls rather than overflowing. `mainAxisSize.min` keeps
      // a short badge's sheet compact — this only bites when it has to.
      child: SingleChildScrollView(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 0, 20, 24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    _familyIcon(focus.family),
                    color: tile.earned
                        ? CymbraColors.primary
                        : CymbraColors.outline,
                    size: 30,
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          focus.labelIn(languageCode),
                          style: const TextStyle(
                            color: CymbraColors.onSurface,
                            fontSize: 18,
                            fontWeight: FontWeight.w800,
                          ),
                        ),
                        Text(
                          familyLabel(l10n, focus.family),
                          style: const TextStyle(
                            color: CymbraColors.onSurfaceVariant,
                            fontSize: 12,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              // What it takes — server copy, so a new badge explains itself.
              Text(
                focus.descriptionIn(languageCode),
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 13,
                ),
              ),
              const SizedBox(height: 16),
              // Where the user stands right now.
              if (tile.next != null)
                _Progress(badge: tile.next!)
              else
                Text(
                  l10n.achievementEarned,
                  style: const TextStyle(
                    color: CymbraColors.primary,
                    fontSize: 13,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              const SizedBox(height: 8),
              Text(
                earnedAt != null
                    ? l10n.achievementEarnedOn(
                        MaterialLocalizations.of(
                          context,
                        ).formatShortDate(earnedAt.toLocal()),
                      )
                    : l10n.achievementNotEarnedYet,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                ),
              ),
              // The full ladder, for a graduated series only — a standalone badge
              // is a one-rung ladder and would just repeat itself.
              if (tile.ladder.length > 1) ...[
                const SizedBox(height: 20),
                Text(
                  l10n.achievementTiersTitle,
                  style: const TextStyle(
                    color: CymbraColors.primary,
                    fontSize: 13,
                    fontWeight: FontWeight.w800,
                    letterSpacing: 0.5,
                  ),
                ),
                const SizedBox(height: 6),
                for (final rung in tile.ladder)
                  Padding(
                    key: Key('achievement-ladder-${rung.key}'),
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Row(
                      children: [
                        Icon(
                          rung.earned
                              ? Icons.check_circle
                              : Icons.radio_button_unchecked,
                          size: 16,
                          color: rung.earned
                              ? CymbraColors.primary
                              : CymbraColors.outline,
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            rung.labelIn(languageCode),
                            style: TextStyle(
                              color: rung.earned
                                  ? CymbraColors.onSurface
                                  : CymbraColors.onSurfaceVariant,
                              fontSize: 13,
                            ),
                          ),
                        ),
                        Text(
                          l10n.achievementProgress(rung.value, rung.threshold),
                          style: const TextStyle(
                            color: CymbraColors.onSurfaceVariant,
                            fontSize: 12,
                          ),
                        ),
                      ],
                    ),
                  ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// The localized name of a badge family. Chrome, not badge identity: the badges
/// themselves are named by the server. A family this build does not know yet
/// falls back to its raw key rather than disappearing.
String familyLabel(AppLocalizations l10n, String family) => switch (family) {
  'play' => l10n.achievementFamilyPlay,
  'consistency' => l10n.achievementFamilyConsistency,
  'ranking' => l10n.achievementFamilyRanking,
  'contribution' => l10n.achievementFamilyContribution,
  'curation' => l10n.achievementFamilyCuration,
  'learning' => l10n.achievementFamilyLearning,
  _ => family,
};

/// Distinct iconography per family, so the grid is not seven identical medals. A
/// family added server-side later still renders, with a generic medal.
IconData _familyIcon(String family) => switch (family) {
  'play' => Icons.piano,
  'consistency' => Icons.local_fire_department,
  'ranking' => Icons.emoji_events,
  'contribution' => Icons.volunteer_activism,
  'curation' => Icons.rate_review,
  'learning' => Icons.school,
  _ => Icons.military_tech,
};
