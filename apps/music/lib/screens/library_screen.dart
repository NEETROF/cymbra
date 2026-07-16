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
import '../layout/device_class.dart';
import '../services/score_upload_service.dart';
import '../state/contributed_scores.dart';
import '../state/score_catalog.dart';
import '../state/session_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/language_selector.dart';
import 'auth/account_menu.dart';
import 'player_screen.dart';
import 'score_upload_screen.dart';

/// Localized name for a [PracticeLevel] section header.
String _levelLabel(AppLocalizations l10n, PracticeLevel level) =>
    switch (level) {
      PracticeLevel.beginner => l10n.levelBeginner,
      PracticeLevel.intermediate => l10n.levelIntermediate,
      PracticeLevel.advanced => l10n.levelAdvanced,
    };

/// Start screen: the bundled score catalog, grouped by practice level. Tapping
/// an entry records it as the selected score and opens the partition screen.
class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final catalog = ref.watch(scoreCatalogProvider);

    return Scaffold(
      backgroundColor: CymbraColors.surfaceContainerLow,
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).libraryTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
        actions: [
          // Contribution entry point — only when signed in (spec: hidden/disabled
          // otherwise, and the flow is not reachable).
          if (ref.watch(canUseOnlineServicesProvider))
            IconButton(
              icon: const Icon(Icons.library_add_outlined),
              tooltip: 'Contribuer une partition',
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const ScoreUploadScreen(),
                ),
              ),
            ),
          const LanguageSelectorButton(),
          const AccountMenu(),
          const SizedBox(width: 8),
        ],
      ),
      // In landscape the display cutout (camera/notch) sits on a side, so the
      // list must inset for it — otherwise the level headers and tiles run under
      // the notch. The AppBar already handles the top inset. `top: false` avoids
      // double-insetting below it.
      body: SafeArea(
        top: false,
        child: LayoutBuilder(
          builder: (context, constraints) {
            // The wide landscape viewport fits several tiles side by side, so
            // lay each level's scores out in responsive columns (≈340 px each,
            // 1–3). This shows far more scores at once — especially on a phone,
            // where vertical space is scarce.
            final columns = (constraints.maxWidth / 340).floor().clamp(1, 3);
            return ListView(
              padding: const EdgeInsets.symmetric(vertical: 8),
              children: [
                for (final level in PracticeLevel.values)
                  ..._levelSection(
                    context,
                    ref,
                    level,
                    catalog.where((e) => e.level == level).toList(),
                    columns,
                  ),
                // The signed-in user's own uploads (no section when signed out or
                // empty). Byte-sourced, so they open in the player like bundled
                // scores; each carries an owner-only delete.
                ...switch (ref.watch(myContributedScoresProvider)) {
                  AsyncData(:final value) when value.isNotEmpty =>
                    _contributedSection(context, ref, value, columns),
                  _ => const <Widget>[],
                },
              ],
            );
          },
        ),
      ),
    );
  }

  List<Widget> _levelSection(
    BuildContext context,
    WidgetRef ref,
    PracticeLevel level,
    List<CatalogEntry> entries,
    int columns,
  ) {
    if (entries.isEmpty) return const [];
    return [
      Padding(
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
        child: Text(
          _levelLabel(AppLocalizations.of(context), level),
          style: const TextStyle(
            color: CymbraColors.primary,
            fontSize: 15,
            fontWeight: FontWeight.w800,
            letterSpacing: 0.5,
          ),
        ),
      ),
      // Chunk the entries into rows of [columns] equal-width tiles; the last row
      // pads with empty cells so the tiles stay column-aligned.
      for (var i = 0; i < entries.length; i += columns)
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (var j = 0; j < columns; j++)
              Expanded(
                child: i + j < entries.length
                    ? _EntryTile(entry: entries[i + j])
                    : const SizedBox.shrink(),
              ),
          ],
        ),
    ];
  }

  /// The signed-in user's contributed scores, with an owner-only delete on each.
  List<Widget> _contributedSection(
    BuildContext context,
    WidgetRef ref,
    List<CatalogEntry> entries,
    int columns,
  ) => [
    const Padding(
      padding: EdgeInsets.fromLTRB(16, 20, 16, 4),
      child: Text(
        'MES CONTRIBUTIONS',
        style: TextStyle(
          color: CymbraColors.secondary,
          fontSize: 15,
          fontWeight: FontWeight.w800,
          letterSpacing: 0.5,
        ),
      ),
    ),
    for (var i = 0; i < entries.length; i += columns)
      Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (var j = 0; j < columns; j++)
            Expanded(
              child: i + j < entries.length
                  ? _EntryTile(
                      entry: entries[i + j],
                      onDelete: () =>
                          _confirmDelete(context, ref, entries[i + j]),
                    )
                  : const SizedBox.shrink(),
            ),
        ],
      ),
  ];

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

class _EntryTile extends ConsumerWidget {
  final CatalogEntry entry;

  /// When set (a user-owned contributed score), the tile shows a delete action
  /// instead of the open chevron. Bundled entries never get one (spec).
  final VoidCallback? onDelete;
  const _EntryTile({required this.entry, this.onDelete});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Denser rows on phones (shorter, tighter padding) so more scores fit in the
    // short landscape viewport; tablet/desktop keep the roomier tile.
    final isPhone = context.isPhoneLayout;
    return ListTile(
      dense: isPhone,
      visualDensity: isPhone ? VisualDensity.compact : null,
      contentPadding: isPhone
          ? const EdgeInsets.symmetric(horizontal: 12)
          : null,
      leading: const Icon(Icons.music_note, color: CymbraColors.secondary),
      title: Text(
        entry.title,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: const TextStyle(color: CymbraColors.onSurface),
      ),
      subtitle: Text(
        entry.composer,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: const TextStyle(color: CymbraColors.onSurfaceVariant),
      ),
      trailing: onDelete != null
          ? IconButton(
              icon: const Icon(
                Icons.delete_outline,
                color: CymbraColors.onSurfaceVariant,
              ),
              tooltip: 'Supprimer',
              onPressed: onDelete,
            )
          : const Icon(
              Icons.chevron_right,
              color: CymbraColors.onSurfaceVariant,
            ),
      onTap: () {
        ref.read(selectedScoreProvider.notifier).select(entry);
        Navigator.of(
          context,
        ).push(MaterialPageRoute<void>(builder: (_) => const PlayerScreen()));
      },
    );
  }
}
