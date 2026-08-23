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
import '../state/contributed_scores.dart';
import '../state/drums_access.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/catalog_access_widgets.dart';
import '../widgets/library_listeners.dart';
import '../widgets/score_card.dart';
import '../widgets/score_propose_sheet.dart';
import 'open_score.dart';
import 'score_upload_screen.dart';

/// The Score Hub: a card grid over the public catalog with a search bar, a "mes
/// partitions" source toggle, and musical-facet filters in an end-drawer. Add or
/// remove catalog scores from the personal library. Signed-in only (the entry
/// point on the library is gated), so it always has an authenticated identity.
class ScoreHubScreen extends ConsumerStatefulWidget {
  const ScoreHubScreen({super.key});

  @override
  ConsumerState<ScoreHubScreen> createState() => _ScoreHubScreenState();
}

class _ScoreHubScreenState extends ConsumerState<ScoreHubScreen> {
  @override
  void initState() {
    super.initState();
    // A moderator may have changed a contribution's proposal status in the back office;
    // the app can't observe that locally, and `MyUploads` is kept alive by the home
    // screen (so it never refetches on its own). Refresh "mes partitions" each time the
    // hub opens so a stale `pending`/`accepted`/`rejected` tag can't linger (change:
    // add-score-catalog-proposal). `refresh()` keeps the current cards visible while it
    // re-fetches, so there is no loading flicker.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) ref.read(myUploadsProvider.notifier).refresh();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(catalogSearchProvider);
    final notifier = ref.read(catalogSearchProvider.notifier);
    final filtersActive =
        state.hasAdvancedFilters ||
        state.level != null ||
        state.author.isNotEmpty;

    return LibraryListeners(
      child: Scaffold(
        backgroundColor: CymbraColors.background,
        endDrawer: _FiltersDrawer(
          state: state,
          notifier: notifier,
          l10n: l10n,
          // The instrument filter is offered only when the drum feature is
          // visible to this caller (change: add-drums-access); otherwise the
          // drawer is exactly as before. Defence in depth — the backend
          // enforces the audience regardless.
          showInstrumentFilter: ref.watch(drumsEnabledProvider),
        ),
        appBar: AppBar(
          title: Text(l10n.scoreHubTitle),
          backgroundColor: CymbraColors.surfaceContainerLowest,
          actions: [
            IconButton(
              icon: const Icon(Icons.library_add_outlined),
              tooltip: l10n.scoreHubContributeTooltip,
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const ScoreUploadScreen(),
                ),
              ),
            ),
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
          child: _Results(state: state, notifier: notifier, l10n: l10n),
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
      // Compact header: it lives in a floating app bar that scrolls away, so it
      // must stay slim (mobile screens are short). Tighter paddings + a denser
      // search field than a full-page header.
      padding: const EdgeInsets.fromLTRB(16, 6, 16, 8),
      child: Column(
        mainAxisSize: MainAxisSize.min,
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
              contentPadding: const EdgeInsets.symmetric(vertical: 9),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(24),
                borderSide: BorderSide.none,
              ),
            ),
          ),
          const SizedBox(height: 6),
          Row(
            children: [
              // "Mes partitions": a quick-filter — checked shows only the user's
              // uploads; unchecked mixes them into the catalog results.
              FilterChip(
                avatar: Icon(
                  Icons.person,
                  size: 18,
                  color: state.myScoresOnly
                      ? CymbraColors.primaryContainer
                      : CymbraColors.onSurfaceVariant,
                ),
                label: Text(l10n.scoreHubMyScores),
                selected: state.myScoresOnly,
                showCheckmark: false,
                onSelected: notifier.setMyScoresOnly,
              ),
              const Spacer(),
              if (!state.loading)
                Text(
                  l10n.scoreHubResultsCount(state.displayCount),
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

class _Results extends ConsumerWidget {
  const _Results({
    required this.state,
    required this.notifier,
    required this.l10n,
  });

  final CatalogSearchState state;
  final CatalogSearch notifier;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return RefreshIndicator(
      // Pull-to-refresh re-fetches the caller's uploads so a proposal status a
      // moderator changed in the back office is picked up immediately (change:
      // add-score-catalog-proposal).
      onRefresh: () => ref.read(myUploadsProvider.notifier).refresh(),
      child: NotificationListener<ScrollNotification>(
        onNotification: (n) {
          if (n.metrics.pixels >= n.metrics.maxScrollExtent - 320) {
            notifier.loadMore();
          }
          return false;
        },
        child: CustomScrollView(
          // Always scrollable so pull-to-refresh works even when the grid is short.
          physics: const AlwaysScrollableScrollPhysics(),
          slivers: [
            // The search + "mes partitions" chip live in a FLOATING app bar: it
            // shows on entry, scrolls away as the grid scrolls, and snaps back on
            // any upward flick — so short mobile screens aren't half-eaten by it.
            SliverAppBar(
              floating: true,
              snap: true,
              automaticallyImplyLeading: false,
              // The Scaffold owns an endDrawer, and an AppBar with no actions
              // auto-inserts a second end-drawer (≡) button. Filters are already
              // reachable via the tune button in the main AppBar, so pass an
              // explicit (non-empty) actions list to suppress the duplicate.
              actions: const [SizedBox.shrink()],
              backgroundColor: CymbraColors.background,
              surfaceTintColor: Colors.transparent,
              elevation: 0,
              scrolledUnderElevation: 0,
              titleSpacing: 0,
              toolbarHeight: 112,
              title: _SearchBar(state: state, notifier: notifier, l10n: l10n),
            ),
            // Daily free-open quota (change: add-score-daily-access-rewards):
            // renders nothing when the gate is off for this caller.
            const SliverToBoxAdapter(child: CatalogAccessChip()),
            ..._resultSlivers(),
          ],
        ),
      ),
    );
  }

  /// The results area below the floating header: a spinner, the empty state, or
  /// the card grid (+ a load-more spinner).
  List<Widget> _resultSlivers() {
    if (state.loading) {
      return const [
        SliverFillRemaining(
          hasScrollBody: false,
          child: Center(child: CircularProgressIndicator()),
        ),
      ];
    }
    if (state.isEmptyResult) {
      return [
        SliverFillRemaining(
          hasScrollBody: false,
          child: Center(
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
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ];
    }
    return [
      SliverPadding(
        padding: const EdgeInsets.fromLTRB(16, 4, 16, 16),
        sliver: SliverGrid.builder(
          gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
            maxCrossAxisExtent: 320,
            mainAxisSpacing: 16,
            crossAxisSpacing: 16,
            childAspectRatio: 0.78,
          ),
          itemCount: state.entries.length,
          itemBuilder: (context, i) {
            final entry = state.entries[i];
            // Per entry: a catalog result is savable (add/remove heart); the
            // user's own upload instead offers a favorite toggle + delete.
            final isUpload = entry.contributedId != null;
            return _HubCard(
              entry: entry,
              saved: state.isSaved(entry),
              onToggleSave: isUpload ? null : () => notifier.toggleSave(entry),
              deletable: isUpload,
            );
          },
        ),
      ),
      if (state.loadingMore)
        const SliverToBoxAdapter(
          child: Padding(
            padding: EdgeInsets.only(bottom: 12),
            child: Center(child: CircularProgressIndicator()),
          ),
        ),
    ];
  }
}

class _HubCard extends ConsumerWidget {
  const _HubCard({
    required this.entry,
    required this.saved,
    this.onToggleSave,
    this.deletable = false,
  });

  final CatalogEntry entry;
  final bool saved;
  final VoidCallback? onToggleSave;

  /// True for the user's own uploads ("mes partitions"): the card offers a
  /// delete action instead of the save toggle.
  final bool deletable;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final status = entry.proposalStatus;
    return ScoreCard(
      entry: entry,
      onTap: () => openScore(context, ref, entry),
      action: _action(context, ref, l10n),
      // The proposal status pill sits at the bottom-left of the cover (its own slot),
      // so a long label like "En attente de vérification" never overlaps the difficulty
      // badge or the action buttons (change: add-score-catalog-proposal).
      statusTag: deletable && status != null
          ? ScoreProposalTag(status: status)
          : null,
    );
  }

  /// The card's corner action: for the user's own uploads a favorite toggle +
  /// delete; for catalog results a save toggle (or nothing when not saveable).
  Widget? _action(BuildContext context, WidgetRef ref, AppLocalizations l10n) {
    if (deletable) {
      final status = entry.proposalStatus;
      return Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Public-catalog proposal (change: add-score-catalog-proposal): the status pill
          // is rendered by ScoreCard (bottom-left); here only the opt-in propose action
          // (shown when not yet proposed, or to re-propose a rejected score).
          if (status == null || status == 'rejected')
            _overlayButton(
              icon: Icons.public,
              color: CymbraColors.onSurface,
              tooltip: l10n.scoreProposeAction,
              onPressed: () => _propose(context, ref),
            ),
          _overlayButton(
            icon: entry.favorite ? Icons.favorite : Icons.favorite_border,
            color: entry.favorite ? CymbraColors.error : CymbraColors.onSurface,
            tooltip: l10n.scoreHubRemoveFromLibrary,
            onPressed: () => _toggleFavorite(ref),
          ),
          _overlayButton(
            icon: Icons.delete_outline,
            color: CymbraColors.onSurface,
            tooltip: 'Supprimer',
            onPressed: () => _confirmDelete(context, ref),
          ),
        ],
      );
    }
    if (onToggleSave == null) return null;
    return IconButton(
      icon: Icon(
        saved ? Icons.favorite : Icons.favorite_border,
        color: saved ? CymbraColors.error : CymbraColors.onSurface,
      ),
      tooltip: saved
          ? l10n.scoreHubRemoveFromLibrary
          : l10n.scoreHubAddToLibrary,
      onPressed: onToggleSave,
    );
  }

  Widget _overlayButton({
    required IconData icon,
    required Color color,
    required String tooltip,
    required VoidCallback onPressed,
  }) => IconButton(
    visualDensity: VisualDensity.compact,
    icon: Icon(icon, color: color),
    tooltip: tooltip,
    style: IconButton.styleFrom(
      backgroundColor: Colors.black.withValues(alpha: 0.28),
    ),
    onPressed: onPressed,
  );

  /// Favorite / un-favorite one of the caller's uploads from the hub (updates
  /// the home favorites too). Never deletes the upload. Delegates to the notifier
  /// (which reloads itself); the catalog list already reacts to that change, so no
  /// manual invalidate/refresh here.
  void _toggleFavorite(WidgetRef ref) {
    final id = entry.contributedId;
    if (id == null) return;
    ref
        .read(myUploadsProvider.notifier)
        .toggleFavorite(id, favorite: !entry.favorite);
  }

  /// Delete one of the caller's own uploads (destructive) with a confirm dialog,
  /// then refresh the hub list. Removes it everywhere — home + hub.
  Future<void> _confirmDelete(BuildContext context, WidgetRef ref) async {
    final id = entry.contributedId;
    if (id == null) return;
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
    // Fire the notifier action; failures surface via the listener widget, and the
    // catalog list reacts to the uploads change on its own.
    ref.read(myUploadsProvider.notifier).delete(id);
  }

  /// Propose (or re-propose, when rejected) one of the caller's uploads to the public
  /// catalog (change: add-score-catalog-proposal). Opens the licence + attestation
  /// sheet; for a rejected score it shows the reason and requires a justification.
  /// Fires the notifier action and reacts to state (no awaited-return branching).
  Future<void> _propose(BuildContext context, WidgetRef ref) async {
    final id = entry.contributedId;
    if (id == null) return;
    final rejected = entry.proposalStatus == 'rejected';
    final r = await showScoreProposeDialog(
      context,
      rejected: rejected,
      rejectionReason: entry.proposalRejectionReason,
    );
    if (r == null) return;
    // Fire and react: the outcome (submitted / already in the catalog / refused)
    // is surfaced by the library listener, so no message is announced here before
    // the server has answered.
    ref
        .read(myUploadsProvider.notifier)
        .proposeToPublicCatalog(
          id,
          license: r.license,
          attestation: true,
          attribution: r.attribution,
          resubmissionNote: r.justification,
        );
  }
}

// --- advanced filters drawer -----------------------------------------------

class _FiltersDrawer extends StatefulWidget {
  const _FiltersDrawer({
    required this.state,
    required this.notifier,
    required this.l10n,
    this.showInstrumentFilter = false,
  });

  final CatalogSearchState state;
  final CatalogSearch notifier;
  final AppLocalizations l10n;

  /// Whether to offer the instrument filter — true only for the drum audience
  /// (change: add-drums-access); false keeps the drawer exactly as before.
  final bool showInstrumentFilter;

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
                  // Instrument filter (change: add-drums-access), offered only
                  // to the drum audience; the drum option filters to percussion
                  // scores, which the backend serves only to that audience.
                  if (widget.showInstrumentFilter)
                    _Section(
                      icon: Icons.music_note,
                      label: l10n.scoreHubInstrument,
                      children: [
                        _choice(
                          l10n.scoreHubAny,
                          state.instrument == null,
                          () => notifier.setInstrument(null),
                        ),
                        _choice(
                          l10n.instrumentKeyboard,
                          state.instrument == ScoreInstrument.keyboard,
                          () =>
                              notifier.setInstrument(ScoreInstrument.keyboard),
                        ),
                        _choice(
                          l10n.instrumentDrums,
                          state.instrument == ScoreInstrument.percussion,
                          () => notifier.setInstrument(
                            ScoreInstrument.percussion,
                          ),
                        ),
                      ],
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
