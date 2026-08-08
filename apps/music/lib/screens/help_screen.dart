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
import '../state/coaching_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/coach_copy.dart';
import 'notation_glossary_screen.dart';

/// Opens the help/tips surface from any stable entry point (the library app bar,
/// the account menu).
void openHelp(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const HelpScreen()));

/// Help & tips (D5): where a user re-reads how the app works after the one-time
/// hints are gone — the core loop, ratings, points, the shop and badges, the
/// profile, leaderboards, and going public — plus a replay of the guided player
/// sequence.
///
/// Deliberately covers only what a *player* does: moderation is backstage and
/// never surfaced here.
class HelpScreen extends ConsumerWidget {
  const HelpScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final replayArmed = ref.watch(
      coachingProvider.select((s) => s.replayArmed),
    );
    final topics = <({String title, String body})>[
      (title: l10n.helpCoreLoopTitle, body: l10n.helpCoreLoopBody),
      // The one-time hints are re-readable here, from the very same strings.
      coachHintCopy(l10n, CoachHint.ratingDeck),
      (title: l10n.helpPointsTitle, body: l10n.helpPointsBody),
      coachHintCopy(l10n, CoachHint.rewards),
      (title: l10n.helpProfileTitle, body: l10n.helpProfileBody),
      (title: l10n.helpLeaderboardsTitle, body: l10n.helpLeaderboardsBody),
      coachHintCopy(l10n, CoachHint.goingPublic),
    ];

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.helpTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            for (final topic in topics)
              _Topic(title: topic.title, body: topic.body),
            const SizedBox(height: 8),
            _ReplayCard(armed: replayArmed),
            const SizedBox(height: 8),
            const _NotationGuideCard(),
          ],
        ),
      ),
    );
  }
}

class _Topic extends StatelessWidget {
  const _Topic({required this.title, required this.body});

  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 18),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 16,
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            body,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 14,
              height: 1.35,
            ),
          ),
        ],
      ),
    );
  }
}

/// Opens the browsable notation glossary — the "reading the staff" guide whose
/// explanations are the same ones shown as long-press bubbles on a score.
class _NotationGuideCard extends StatelessWidget {
  const _NotationGuideCard();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.notationHelpGlossaryTitle,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 16,
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            l10n.notationHelpHintBody,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 14,
              height: 1.35,
            ),
          ),
          const SizedBox(height: 10),
          Align(
            alignment: Alignment.centerLeft,
            child: FilledButton.icon(
              key: const Key('help-open-notation-glossary'),
              icon: const Icon(Icons.menu_book_outlined),
              label: Text(l10n.notationHelpGlossaryOpen),
              onPressed: () => openNotationGlossary(context),
            ),
          ),
        ],
      ),
    );
  }
}

/// Replays the guided player sequence. The controls it points at live in the
/// player's setup surface, so the replay is *armed* here and runs the next time
/// a score is opened — stated plainly rather than pretending it starts now.
class _ReplayCard extends ConsumerWidget {
  const _ReplayCard({required this.armed});

  final bool armed;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final copy = coachHintCopy(l10n, CoachHint.playerTour);
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            copy.title,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 16,
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            copy.body,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 14,
              height: 1.35,
            ),
          ),
          const SizedBox(height: 10),
          Align(
            alignment: Alignment.centerLeft,
            child: FilledButton.icon(
              key: const Key('help-replay-player-tour'),
              icon: const Icon(Icons.replay),
              label: Text(
                armed ? l10n.helpReplayArmed : l10n.helpReplayPlayerTour,
              ),
              onPressed: armed ? null : () => _arm(context, ref, l10n),
            ),
          ),
        ],
      ),
    );
  }

  void _arm(BuildContext context, WidgetRef ref, AppLocalizations l10n) {
    ref.read(coachingProvider.notifier).armPlayerTourReplay();
    showAppSnackBar(ScaffoldMessenger.of(context), l10n.helpReplayArmed);
  }
}
