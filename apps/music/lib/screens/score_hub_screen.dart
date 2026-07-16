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
import '../state/catalog_search_notifier.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import 'player_screen.dart';

/// The Score Hub: search the public catalog by title/composer, narrow by author
/// and difficulty, toggle to the user's own uploads ("mes partitions"), and add
/// or remove catalog scores from the personal library. Signed-in only (the entry
/// point on the library is gated), so it always has an authenticated identity.
class ScoreHubScreen extends ConsumerWidget {
  const ScoreHubScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(catalogSearchProvider);
    final notifier = ref.read(catalogSearchProvider.notifier);

    return Scaffold(
      backgroundColor: CymbraColors.surfaceContainerLow,
      appBar: AppBar(
        title: Text(l10n.scoreHubTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: SafeArea(
        top: false,
        child: Column(
          children: [
            _Filters(state: state, notifier: notifier, l10n: l10n),
            const Divider(height: 1),
            Expanded(
              child: _Results(state: state, notifier: notifier, l10n: l10n),
            ),
          ],
        ),
      ),
    );
  }
}

class _Filters extends StatelessWidget {
  const _Filters({
    required this.state,
    required this.notifier,
    required this.l10n,
  });

  final CatalogSearchState state;
  final CatalogSearch notifier;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 12, 12, 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(
            decoration: InputDecoration(
              prefixIcon: const Icon(Icons.search),
              hintText: l10n.scoreHubSearchHint,
              isDense: true,
              border: const OutlineInputBorder(),
            ),
            textInputAction: TextInputAction.search,
            onChanged: notifier.setQuery,
          ),
          const SizedBox(height: 8),
          TextField(
            decoration: InputDecoration(
              prefixIcon: const Icon(Icons.person_outline),
              hintText: l10n.scoreHubComposerFilter,
              isDense: true,
              border: const OutlineInputBorder(),
            ),
            onChanged: notifier.setAuthor,
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 4,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              // "Mes partitions" quick-filter: scopes the hub to the user's own
              // uploads instead of the public catalog.
              FilterChip(
                label: Text(l10n.scoreHubMyScores),
                selected: state.isMyUploads,
                onSelected: (on) => notifier.setSource(
                  on ? CatalogSource.myUploads : CatalogSource.catalog,
                ),
              ),
              const SizedBox(width: 4),
              ChoiceChip(
                label: Text(l10n.scoreHubAllLevels),
                selected: state.level == null,
                onSelected: (_) => notifier.setLevel(null),
              ),
              for (final level in PracticeLevel.values)
                ChoiceChip(
                  label: Text(level.localizedLabel(l10n)),
                  selected: state.level == level,
                  onSelected: (_) => notifier.setLevel(level),
                ),
            ],
          ),
          // Advanced musical-facet filters, collapsed by default so the basic
          // controls stay uncluttered.
          _AdvancedFilters(state: state, notifier: notifier, l10n: l10n),
        ],
      ),
    );
  }
}

/// Collapsible panel of musical-facet filters (rhythmic granularity, chords/
/// tuplets/dotted, hand span, tempo band).
class _AdvancedFilters extends StatelessWidget {
  const _AdvancedFilters({
    required this.state,
    required this.notifier,
    required this.l10n,
  });

  final CatalogSearchState state;
  final CatalogSearch notifier;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    return Theme(
      // Drop the ExpansionTile dividers for a cleaner inline look.
      data: Theme.of(context).copyWith(dividerColor: Colors.transparent),
      child: ExpansionTile(
        tilePadding: EdgeInsets.zero,
        childrenPadding: const EdgeInsets.only(bottom: 8),
        title: Text(
          l10n.scoreHubAdvancedFilters,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 13,
            fontWeight: FontWeight.w600,
          ),
        ),
        children: [
          _chipRow(l10n.scoreHubFastestNote, [
            _opt(
              l10n.scoreHubAny,
              state.maxNoteValue == null,
              () => notifier.setMaxNoteValue(null),
            ),
            _opt(
              l10n.scoreHubNoteQuarter,
              state.maxNoteValue == 4,
              () => notifier.setMaxNoteValue(4),
            ),
            _opt(
              l10n.scoreHubNoteEighth,
              state.maxNoteValue == 8,
              () => notifier.setMaxNoteValue(8),
            ),
            _opt(
              l10n.scoreHubNoteSixteenth,
              state.maxNoteValue == 16,
              () => notifier.setMaxNoteValue(16),
            ),
          ]),
          _chipRow(l10n.scoreHubHandSpan, [
            _opt(
              l10n.scoreHubAny,
              state.maxAmbitusSemitones == null,
              () => notifier.setMaxAmbitusSemitones(null),
            ),
            _opt(
              l10n.scoreHubSpanOneOctave,
              state.maxAmbitusSemitones == 12,
              () => notifier.setMaxAmbitusSemitones(12),
            ),
            _opt(
              l10n.scoreHubSpanTwoOctaves,
              state.maxAmbitusSemitones == 24,
              () => notifier.setMaxAmbitusSemitones(24),
            ),
          ]),
          _chipRow(l10n.scoreHubTempo, [
            _opt(
              l10n.scoreHubAny,
              state.minBpm == null && state.maxBpm == null,
              () => notifier.setTempoBand(null, null),
            ),
            _opt(
              l10n.scoreHubTempoSlow,
              state.maxBpm == 75,
              () => notifier.setTempoBand(null, 75),
            ),
            _opt(
              l10n.scoreHubTempoModerate,
              state.minBpm == 76 && state.maxBpm == 120,
              () => notifier.setTempoBand(76, 120),
            ),
            _opt(
              l10n.scoreHubTempoFast,
              state.minBpm == 121,
              () => notifier.setTempoBand(121, null),
            ),
          ]),
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Wrap(
              spacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                FilterChip(
                  label: Text(l10n.scoreHubChords),
                  selected: state.hasChords == true,
                  onSelected: notifier.toggleChords,
                ),
                FilterChip(
                  label: Text(l10n.scoreHubTuplets),
                  selected: state.hasTuplets == true,
                  onSelected: notifier.toggleTuplets,
                ),
                FilterChip(
                  label: Text(l10n.scoreHubDotted),
                  selected: state.hasDotted == true,
                  onSelected: notifier.toggleDotted,
                ),
                if (state.hasAdvancedFilters)
                  TextButton(
                    onPressed: notifier.clearAdvancedFilters,
                    child: Text(l10n.scoreHubClearFilters),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// A labelled row of mutually-exclusive choice chips.
  Widget _chipRow(String label, List<Widget> chips) => Padding(
    padding: const EdgeInsets.only(top: 4),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 12,
          ),
        ),
        const SizedBox(height: 2),
        Wrap(spacing: 8, runSpacing: 4, children: chips),
      ],
    ),
  );

  Widget _opt(String label, bool selected, VoidCallback onTap) => ChoiceChip(
    label: Text(label),
    selected: selected,
    onSelected: (_) => onTap(),
  );
}

class _Results extends StatelessWidget {
  const _Results({
    required this.state,
    required this.notifier,
    required this.l10n,
  });

  final CatalogSearchState state;
  final CatalogSearch notifier;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    if (state.loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (state.isEmptyResult) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            state.isMyUploads
                ? l10n.scoreHubMyScoresEmpty
                : l10n.scoreHubNoResults,
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.onSurfaceVariant),
          ),
        ),
      );
    }
    // Auto-load the next page when the list nears its end (catalog source only).
    return NotificationListener<ScrollNotification>(
      onNotification: (n) {
        if (n.metrics.pixels >= n.metrics.maxScrollExtent - 240) {
          notifier.loadMore();
        }
        return false;
      },
      child: ListView.builder(
        itemCount: state.entries.length + (state.loadingMore ? 1 : 0),
        itemBuilder: (context, i) {
          if (i >= state.entries.length) {
            return const Padding(
              padding: EdgeInsets.all(16),
              child: Center(child: CircularProgressIndicator()),
            );
          }
          final entry = state.entries[i];
          return _ResultTile(
            entry: entry,
            saved: state.isSaved(entry),
            // Only catalog results are savable; uploads are already owned.
            onToggleSave: state.isMyUploads
                ? null
                : () => notifier.toggleSave(entry),
          );
        },
      ),
    );
  }
}

class _ResultTile extends ConsumerWidget {
  const _ResultTile({
    required this.entry,
    required this.saved,
    this.onToggleSave,
  });

  final CatalogEntry entry;
  final bool saved;

  /// Save/remove toggle (catalog results); `null` for the user's own uploads.
  final VoidCallback? onToggleSave;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return ListTile(
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
      trailing: onToggleSave == null
          ? const Icon(
              Icons.chevron_right,
              color: CymbraColors.onSurfaceVariant,
            )
          : IconButton(
              icon: Icon(
                saved ? Icons.bookmark : Icons.bookmark_add_outlined,
                color: saved
                    ? CymbraColors.primary
                    : CymbraColors.onSurfaceVariant,
              ),
              tooltip: saved
                  ? l10n.scoreHubRemoveFromLibrary
                  : l10n.scoreHubAddToLibrary,
              onPressed: onToggleSave,
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
