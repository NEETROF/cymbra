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
import '../widgets/score_card.dart';
import 'player_screen.dart';

/// The Score Hub: a card grid over the public catalog with a search bar, a "mes
/// partitions" source toggle, and musical-facet filters in an end-drawer. Add or
/// remove catalog scores from the personal library. Signed-in only (the entry
/// point on the library is gated), so it always has an authenticated identity.
class ScoreHubScreen extends ConsumerWidget {
  const ScoreHubScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(catalogSearchProvider);
    final notifier = ref.read(catalogSearchProvider.notifier);
    final filtersActive =
        state.hasAdvancedFilters ||
        state.level != null ||
        state.author.isNotEmpty;

    return Scaffold(
      backgroundColor: CymbraColors.background,
      endDrawer: _FiltersDrawer(state: state, notifier: notifier, l10n: l10n),
      appBar: AppBar(
        title: Text(l10n.scoreHubTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
        actions: [
          Builder(
            builder: (ctx) => Padding(
              padding: const EdgeInsets.only(right: 4),
              child: IconButton.filledTonal(
                icon: Badge(
                  isLabelVisible: filtersActive,
                  smallSize: 8,
                  child: const Icon(Icons.tune),
                ),
                tooltip: l10n.scoreHubAdvancedFilters,
                onPressed: () => Scaffold.of(ctx).openEndDrawer(),
              ),
            ),
          ),
        ],
      ),
      body: SafeArea(
        top: false,
        child: Column(
          children: [
            _SearchBar(state: state, notifier: notifier, l10n: l10n),
            Expanded(
              child: _Results(state: state, notifier: notifier, l10n: l10n),
            ),
          ],
        ),
      ),
    );
  }
}

class _SearchBar extends StatelessWidget {
  const _SearchBar({
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
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(
            onChanged: notifier.setQuery,
            textInputAction: TextInputAction.search,
            style: const TextStyle(color: CymbraColors.onSurface),
            decoration: InputDecoration(
              hintText: l10n.scoreHubSearchHint,
              prefixIcon: const Icon(Icons.search, color: CymbraColors.outline),
              filled: true,
              fillColor: CymbraColors.surfaceContainer,
              isDense: true,
              contentPadding: const EdgeInsets.symmetric(vertical: 14),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(28),
                borderSide: BorderSide.none,
              ),
            ),
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              // "Mes partitions" scopes the hub to the user's own uploads.
              ChoiceChip(
                avatar: Icon(
                  state.isMyUploads ? Icons.person : Icons.public,
                  size: 18,
                  color: state.isMyUploads
                      ? CymbraColors.primaryContainer
                      : CymbraColors.onSurfaceVariant,
                ),
                label: Text(l10n.scoreHubMyScores),
                selected: state.isMyUploads,
                showCheckmark: false,
                onSelected: (on) => notifier.setSource(
                  on ? CatalogSource.myUploads : CatalogSource.catalog,
                ),
              ),
              const Spacer(),
              if (!state.loading)
                Text(
                  l10n.scoreHubResultsCount(state.entries.length),
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 13,
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }
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
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(
                Icons.library_music_outlined,
                size: 48,
                color: CymbraColors.outline,
              ),
              const SizedBox(height: 12),
              Text(
                state.isMyUploads
                    ? l10n.scoreHubMyScoresEmpty
                    : l10n.scoreHubNoResults,
                textAlign: TextAlign.center,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            ],
          ),
        ),
      );
    }
    return NotificationListener<ScrollNotification>(
      onNotification: (n) {
        if (n.metrics.pixels >= n.metrics.maxScrollExtent - 320) {
          notifier.loadMore();
        }
        return false;
      },
      child: Stack(
        children: [
          GridView.builder(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
            gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
              maxCrossAxisExtent: 320,
              mainAxisSpacing: 16,
              crossAxisSpacing: 16,
              childAspectRatio: 0.78,
            ),
            itemCount: state.entries.length,
            itemBuilder: (context, i) {
              final entry = state.entries[i];
              return _HubCard(
                entry: entry,
                saved: state.isSaved(entry),
                // Only catalog results are savable; uploads are already owned.
                onToggleSave: state.isMyUploads
                    ? null
                    : () => notifier.toggleSave(entry),
              );
            },
          ),
          if (state.loadingMore)
            const Positioned(
              left: 0,
              right: 0,
              bottom: 12,
              child: Center(child: CircularProgressIndicator()),
            ),
        ],
      ),
    );
  }
}

class _HubCard extends ConsumerWidget {
  const _HubCard({required this.entry, required this.saved, this.onToggleSave});

  final CatalogEntry entry;
  final bool saved;
  final VoidCallback? onToggleSave;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return ScoreCard(
      entry: entry,
      onTap: () {
        ref.read(selectedScoreProvider.notifier).select(entry);
        Navigator.of(
          context,
        ).push(MaterialPageRoute<void>(builder: (_) => const PlayerScreen()));
      },
      action: onToggleSave == null
          ? null
          : IconButton(
              icon: Icon(
                saved ? Icons.favorite : Icons.favorite_border,
                color: saved ? CymbraColors.error : CymbraColors.onSurface,
              ),
              tooltip: saved
                  ? l10n.scoreHubRemoveFromLibrary
                  : l10n.scoreHubAddToLibrary,
              onPressed: onToggleSave,
            ),
    );
  }
}

// --- advanced filters drawer -----------------------------------------------

class _FiltersDrawer extends StatefulWidget {
  const _FiltersDrawer({
    required this.state,
    required this.notifier,
    required this.l10n,
  });

  final CatalogSearchState state;
  final CatalogSearch notifier;
  final AppLocalizations l10n;

  @override
  State<_FiltersDrawer> createState() => _FiltersDrawerState();
}

class _FiltersDrawerState extends State<_FiltersDrawer> {
  late final TextEditingController _author = TextEditingController(
    text: widget.state.author,
  );

  @override
  void dispose() {
    _author.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = widget.l10n;
    final state = widget.state;
    final notifier = widget.notifier;
    return Drawer(
      backgroundColor: CymbraColors.surfaceContainerLow,
      child: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 16, 8, 8),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      l10n.scoreHubAdvancedFilters,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: CymbraColors.primary,
                        fontSize: 20,
                        fontWeight: FontWeight.w800,
                      ),
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.of(context).maybePop(),
                  ),
                ],
              ),
            ),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.symmetric(horizontal: 20),
                children: [
                  TextField(
                    controller: _author,
                    onChanged: notifier.setAuthor,
                    style: const TextStyle(color: CymbraColors.onSurface),
                    decoration: InputDecoration(
                      labelText: l10n.scoreHubComposerFilter,
                      prefixIcon: const Icon(Icons.person_outline),
                      isDense: true,
                    ),
                  ),
                  _Section(
                    icon: Icons.bar_chart,
                    label: l10n.scoreHubDifficulty,
                    children: [
                      _choice(
                        l10n.scoreHubAny,
                        state.level == null,
                        () => notifier.setLevel(null),
                      ),
                      for (final lvl in PracticeLevel.values)
                        _choice(
                          lvl.localizedLabel(l10n),
                          state.level == lvl,
                          () => notifier.setLevel(lvl),
                        ),
                    ],
                  ),
                  _Section(
                    icon: Icons.speed,
                    label: l10n.scoreHubFastestNote,
                    children: [
                      _choice(
                        l10n.scoreHubAny,
                        state.maxNoteValue == null,
                        () => notifier.setMaxNoteValue(null),
                      ),
                      _choice(
                        l10n.scoreHubNoteQuarter,
                        state.maxNoteValue == 4,
                        () => notifier.setMaxNoteValue(4),
                      ),
                      _choice(
                        l10n.scoreHubNoteEighth,
                        state.maxNoteValue == 8,
                        () => notifier.setMaxNoteValue(8),
                      ),
                      _choice(
                        l10n.scoreHubNoteSixteenth,
                        state.maxNoteValue == 16,
                        () => notifier.setMaxNoteValue(16),
                      ),
                    ],
                  ),
                  _Section(
                    icon: Icons.straighten,
                    label: l10n.scoreHubHandSpan,
                    children: [
                      _choice(
                        l10n.scoreHubAny,
                        state.maxAmbitusSemitones == null,
                        () => notifier.setMaxAmbitusSemitones(null),
                      ),
                      _choice(
                        l10n.scoreHubSpanOneOctave,
                        state.maxAmbitusSemitones == 12,
                        () => notifier.setMaxAmbitusSemitones(12),
                      ),
                      _choice(
                        l10n.scoreHubSpanTwoOctaves,
                        state.maxAmbitusSemitones == 24,
                        () => notifier.setMaxAmbitusSemitones(24),
                      ),
                    ],
                  ),
                  _Section(
                    icon: Icons.timer_outlined,
                    label: l10n.scoreHubTempo,
                    children: [
                      _choice(
                        l10n.scoreHubAny,
                        state.minBpm == null && state.maxBpm == null,
                        () => notifier.setTempoBand(null, null),
                      ),
                      _choice(
                        l10n.scoreHubTempoSlow,
                        state.maxBpm == 75,
                        () => notifier.setTempoBand(null, 75),
                      ),
                      _choice(
                        l10n.scoreHubTempoModerate,
                        state.minBpm == 76 && state.maxBpm == 120,
                        () => notifier.setTempoBand(76, 120),
                      ),
                      _choice(
                        l10n.scoreHubTempoFast,
                        state.minBpm == 121,
                        () => notifier.setTempoBand(121, null),
                      ),
                    ],
                  ),
                  _Section(
                    icon: Icons.piano,
                    label: l10n.scoreHubRhythmTexture,
                    children: [
                      _filter(
                        l10n.scoreHubChords,
                        state.hasChords == true,
                        notifier.toggleChords,
                      ),
                      _filter(
                        l10n.scoreHubTuplets,
                        state.hasTuplets == true,
                        notifier.toggleTuplets,
                      ),
                      _filter(
                        l10n.scoreHubDotted,
                        state.hasDotted == true,
                        notifier.toggleDotted,
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                ],
              ),
            ),
            const Divider(height: 1),
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  SizedBox(
                    width: double.infinity,
                    child: FilledButton(
                      onPressed: () => Navigator.of(context).maybePop(),
                      child: Text(l10n.scoreHubApplyFilters),
                    ),
                  ),
                  TextButton(
                    onPressed: () {
                      _author.clear();
                      notifier.setAuthor('');
                      notifier.setLevel(null);
                      notifier.clearAdvancedFilters();
                    },
                    child: Text(l10n.scoreHubResetAll),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  // `showCheckmark: false` so selecting a chip doesn't add a tick that changes
  // its width and reflows the row; the fill colour already marks the selection.
  Widget _choice(String label, bool selected, VoidCallback onTap) => ChoiceChip(
    label: Text(label),
    selected: selected,
    showCheckmark: false,
    onSelected: (_) => onTap(),
  );

  Widget _filter(String label, bool selected, ValueChanged<bool> onSelected) =>
      FilterChip(
        label: Text(label),
        selected: selected,
        showCheckmark: false,
        onSelected: onSelected,
      );
}

class _Section extends StatelessWidget {
  const _Section({
    required this.icon,
    required this.label,
    required this.children,
  });

  final IconData icon;
  final String label;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 18, color: CymbraColors.secondary),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  label.toUpperCase(),
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 12,
                    fontWeight: FontWeight.w800,
                    letterSpacing: 0.6,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          Wrap(spacing: 8, runSpacing: 8, children: children),
        ],
      ),
    );
  }
}
