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
import '../services/profile_service.dart';
import '../state/play_activity_notifier.dart';
import '../state/profile_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'play_heatmap.dart';

/// Slide-in **right-hand panel** showing another player's profile, opened by
/// tapping their pseudo on a leaderboard (change: add-global-leaderboard).
///
/// It deliberately does NOT navigate: the board underneath stays mounted and
/// visible behind the scrim, so dismissing the panel returns the user exactly
/// where they were — mid-scroll, on the same mode and season.
///
/// A [Scaffold] `endDrawer` (the repo's usual drawer) is not usable here: the
/// per-piece board also renders inside a `Dialog`, which has no Scaffold in its
/// subtree. A right-anchored route gives the same affordance from every host —
/// the Community screen, the leaderboard dialog, and the pre-play modal — with a
/// single code path.
///
/// The content is **read-only**: it shows the same allow-listed public fields as
/// another player's profile page (handle/display name, activity, songs played)
/// and never the owner-only controls (visibility, analytics consent, rewards).
Future<void> showPlayerProfilePanel(
  BuildContext context, {
  required String userId,
}) {
  final l10n = AppLocalizations.of(context);
  return showGeneralDialog<void>(
    context: context,
    barrierDismissible: true,
    barrierLabel: l10n.profileTitle,
    barrierColor: Colors.black54,
    transitionDuration: const Duration(milliseconds: 220),
    pageBuilder: (_, _, _) => Align(
      alignment: Alignment.centerRight,
      child: _Panel(userId: userId),
    ),
    transitionBuilder: (_, animation, _, child) => SlideTransition(
      position: Tween<Offset>(
        begin: const Offset(1, 0),
        end: Offset.zero,
      ).animate(CurvedAnimation(parent: animation, curve: Curves.easeOutCubic)),
      child: child,
    ),
  );
}

class _Panel extends StatelessWidget {
  const _Panel({required this.userId});

  final String userId;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final width = MediaQuery.of(context).size.width;
    return Material(
      color: CymbraColors.surfaceContainerLow,
      child: SafeArea(
        child: SizedBox(
          // Roomy on a tablet, but never edge-to-edge on a phone in landscape:
          // the board must stay partly visible so the panel reads as an overlay.
          width: width * 0.42 < 320 ? width * 0.7 : 420,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 8, 8, 0),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.profileTitle,
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.w600,
                          color: CymbraColors.onSurface,
                        ),
                      ),
                    ),
                    IconButton(
                      key: const Key('player-profile-panel-close'),
                      visualDensity: VisualDensity.compact,
                      icon: const Icon(
                        Icons.close,
                        color: CymbraColors.onSurface,
                      ),
                      onPressed: () => Navigator.of(context).pop(),
                    ),
                  ],
                ),
              ),
              Expanded(child: PlayerProfileView(userId: userId)),
            ],
          ),
        ),
      ),
    );
  }
}

/// Another player's profile, **read-only** — the allow-listed public fields only.
/// Exposed separately from the panel so it can be rendered (and tested) on its own.
class PlayerProfileView extends ConsumerWidget {
  const PlayerProfileView({super.key, required this.userId});

  final String userId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final profile = ref.watch(playerProfileProvider(userId));
    return profile.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      // Fail-closed on the server: a private/ineligible target reads as an error,
      // and the panel says so rather than surfacing the raw gRPC failure.
      error: (_, _) => Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            l10n.profileUnavailable,
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.onSurfaceVariant),
          ),
        ),
      ),
      data: (p) => ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _Header(profile: p),
          const SizedBox(height: 20),
          _Activity(userId: userId),
        ],
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.profile});

  final PlayerProfile profile;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final name = profile.displayName ?? profile.handle;
    return Row(
      children: [
        const CircleAvatar(radius: 24, child: Icon(Icons.person)),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (name != null)
                Text(
                  name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: theme.textTheme.titleMedium,
                ),
              if (profile.handle != null)
                Text('@${profile.handle}', style: theme.textTheme.bodyMedium),
            ],
          ),
        ),
      ],
    );
  }
}

class _Activity extends ConsumerWidget {
  const _Activity({required this.userId});

  final String userId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final activity = ref.watch(playActivityProvider(userId));
    return activity.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => const SizedBox.shrink(),
      data: (a) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          PlayHeatmap(activity: a),
          const SizedBox(height: 12),
          Text(
            l10n.profileSongsPlayed(a.totalSessions),
            style: Theme.of(context).textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }
}
