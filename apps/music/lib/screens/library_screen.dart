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
import '../services/catalog_service.dart';
import '../services/score_upload_service.dart';
import '../state/contributed_scores.dart';
import '../state/saved_catalog_scores.dart';
import '../state/score_catalog.dart';
import '../state/session_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/language_selector.dart';
import '../widgets/score_card.dart';
import 'auth/account_menu.dart';
import 'player_screen.dart';
import 'score_hub_screen.dart';
import 'score_upload_screen.dart';

/// Localized name for a [PracticeLevel] section header.
String _levelLabel(AppLocalizations l10n, PracticeLevel level) =>
    switch (level) {
      PracticeLevel.beginner => l10n.levelBeginner,
      PracticeLevel.intermediate => l10n.levelIntermediate,
      PracticeLevel.advanced => l10n.levelAdvanced,
    };

/// Start screen: the bundled score catalog grouped by practice level, plus the
/// signed-in user's uploads and their saved catalog scores — each rendered as a
/// [ScoreCard]. Tapping a card records it as the selected score and opens the
/// player; the per-section action differs (bundled = open only, contributions =
/// delete, saved = remove-from-library).
class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final catalog = ref.watch(scoreCatalogProvider);

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.libraryTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
        actions: [
          // Score Hub + contribution entry points — only when signed in (spec:
          // hidden/disabled otherwise, and the flow is not reachable).
          if (ref.watch(canUseOnlineServicesProvider)) ...[
            IconButton(
              icon: const Icon(Icons.search),
              tooltip: l10n.scoreHubEntryTooltip,
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(builder: (_) => const ScoreHubScreen()),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.library_add_outlined),
              tooltip: 'Contribuer une partition',
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const ScoreUploadScreen(),
                ),
              ),
            ),
          ],
          const LanguageSelectorButton(),
          const AccountMenu(),
          const SizedBox(width: 8),
        ],
      ),
      body: SafeArea(
        top: false,
        child: ListView(
          padding: const EdgeInsets.only(bottom: 24),
          children: [
            // Bundled catalog, grouped by level — open only, no per-card action.
            for (final level in PracticeLevel.values)
              ..._section(
                context,
                _levelLabel(l10n, level),
                CymbraColors.primary,
                catalog.where((e) => e.level == level).toList(),
                (entry) => _open(context, ref, entry),
                actionFor: (_) => null,
              ),
            // The user's own uploads (hidden when signed out / empty). Each card
            // carries an owner-only delete.
            ...switch (ref.watch(myContributedScoresProvider)) {
              AsyncData(:final value) when value.isNotEmpty => _section(
                context,
                'MES CONTRIBUTIONS',
                CymbraColors.secondary,
                value,
                (entry) => _open(context, ref, entry),
                actionFor: (entry) => _CardAction(
                  icon: Icons.delete_outline,
                  tooltip: 'Supprimer',
                  onPressed: () => _confirmDelete(context, ref, entry),
                ),
              ),
              _ => const <Widget>[],
            },
            // Catalog scores saved from the Score Hub (hidden when signed out /
            // empty). Each card carries a remove-from-library action that never
            // deletes the public catalog entry.
            ...switch (ref.watch(savedCatalogScoresProvider)) {
              AsyncData(:final value) when value.isNotEmpty => _section(
                context,
                l10n.librarySavedSection,
                CymbraColors.primary,
                value,
                (entry) => _open(context, ref, entry),
                actionFor: (entry) => _CardAction(
                  icon: Icons.favorite,
                  color: CymbraColors.error,
                  tooltip: l10n.scoreHubRemoveFromLibrary,
                  onPressed: () => _removeSaved(ref, entry),
                ),
              ),
              _ => const <Widget>[],
            },
          ],
        ),
      ),
    );
  }

  /// A titled section: a header followed by a responsive grid of [ScoreCard]s.
  /// [actionFor] returns the per-card overlay action (or null for open-only).
  List<Widget> _section(
    BuildContext context,
    String title,
    Color titleColor,
    List<CatalogEntry> entries,
    void Function(CatalogEntry) onOpen, {
    required Widget? Function(CatalogEntry) actionFor,
  }) {
    if (entries.isEmpty) return const [];
    return [
      Padding(
        padding: const EdgeInsets.fromLTRB(16, 20, 16, 8),
        child: Text(
          title,
          style: TextStyle(
            color: titleColor,
            fontSize: 15,
            fontWeight: FontWeight.w800,
            letterSpacing: 0.5,
          ),
        ),
      ),
      GridView.builder(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
          maxCrossAxisExtent: 320,
          mainAxisSpacing: 16,
          crossAxisSpacing: 16,
          childAspectRatio: 0.78,
        ),
        itemCount: entries.length,
        itemBuilder: (context, i) => ScoreCard(
          entry: entries[i],
          onTap: () => onOpen(entries[i]),
          action: actionFor(entries[i]),
        ),
      ),
    ];
  }

  void _open(BuildContext context, WidgetRef ref, CatalogEntry entry) {
    ref.read(selectedScoreProvider.notifier).select(entry);
    Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => const PlayerScreen()));
  }

  /// Remove a saved catalog score from the library (backend remove + refresh).
  /// Reversible — the score can be found and saved again from the hub — so no
  /// destructive-delete confirmation.
  Future<void> _removeSaved(WidgetRef ref, CatalogEntry entry) async {
    await ref.read(catalogServiceProvider).remove(entry.catalogId!);
    ref.invalidate(savedCatalogScoresProvider);
  }

  Future<void> _confirmDelete(
    BuildContext context,
    WidgetRef ref,
    CatalogEntry entry,
  ) async {
    final ok = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Supprimer cette partition ?'),
        content: Text('« ${entry.title} » sera définitivement supprimée.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('Annuler'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('Supprimer'),
          ),
        ],
      ),
    );
    if (ok != true) return;
    await ref
        .read(scoreUploadServiceProvider)
        .deleteScore(entry.contributedId!);
    ref.invalidate(myContributedScoresProvider);
  }
}

/// A small overlay action on a [ScoreCard] cover (delete / remove), with a
/// subtle scrim so the icon stays legible over the artwork.
class _CardAction extends StatelessWidget {
  const _CardAction({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
    this.color = CymbraColors.onSurface,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(icon, color: color),
      tooltip: tooltip,
      style: IconButton.styleFrom(
        backgroundColor: Colors.black.withValues(alpha: 0.28),
      ),
      onPressed: onPressed,
    );
  }
}
