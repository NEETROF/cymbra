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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../state/contributed_scores.dart';
import '../state/drums_access.dart';
import '../state/instrument_context.dart';
import '../state/favorite_scores.dart';
import '../state/player_preferences.dart';
import '../state/saved_catalog_scores.dart';
import '../state/score_catalog.dart';
import '../state/session_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/catalog_access_widgets.dart';
import '../widgets/courses_section.dart';
import '../widgets/instrument_context_widgets.dart';
import '../widgets/library_listeners.dart';
import '../widgets/rating_invite_banner.dart';
import '../services/connectivity_service.dart';
import '../widgets/score_card.dart';
import 'auth/account_menu.dart';
import 'community_screen.dart';
import 'help_screen.dart';
import 'onboarding/sign_in_invitation.dart';
import 'open_score.dart';
import 'rating_deck_screen.dart';
import 'score_hub_screen.dart';
import 'soundfonts_screen.dart';

/// Localized name for a [PracticeLevel] section header.
String _levelLabel(AppLocalizations l10n, PracticeLevel level) =>
    switch (level) {
      PracticeLevel.beginner => l10n.levelBeginner,
      PracticeLevel.intermediate => l10n.levelIntermediate,
      PracticeLevel.advanced => l10n.levelAdvanced,
    };

/// Start screen. Signed in, it is the user's **favorites** — the catalog scores
/// they saved from the hub plus their favorited uploads, unified and grouped by
/// level (each removable from favorites; deletion of an upload happens in the
/// hub's "mes partitions"). Signed out, it shows the bundled demo catalog.
class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Warm the persisted play preferences at startup (the library is the first
    // screen) so they're restored before the first score's player seeds from
    // them — otherwise a cold-start open would fall back to defaults. Listen (not
    // watch): activate the keepAlive provider without rebuilding this screen when
    // the settings change.
    ref.listen(playerPreferencesProvider, (_, _) {});
    final l10n = AppLocalizations.of(context);
    final signedIn = ref.watch(canUseOnlineServicesProvider);
    // The context switcher rides the home header, only while drums are
    // visible (change: add-instrument-context); everyone else gets today's
    // header untouched.
    final drumsVisible = ref.watch(drumsEnabledProvider);

    return LibraryListeners(
      child: Scaffold(
        backgroundColor: CymbraColors.background,
        appBar: AppBar(
          title: Text(l10n.libraryTitle),
          backgroundColor: CymbraColors.surfaceContainerLowest,
          bottom: drumsVisible
              ? const PreferredSize(
                  preferredSize: Size.fromHeight(54),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: InstrumentSwitcher(),
                  ),
                )
              : null,
          actions: [
            // Shown to everyone: reaching them signed out is what triggers the
            // contextual sign-in invitation (change: add-welcome-onboarding, D3)
            // — the benefit is named, declining keeps the user here, and
            // accepting resumes the action they were after.
            IconButton(
              key: const Key('library-hub'),
              icon: const Icon(Icons.search),
              tooltip: l10n.scoreHubEntryTooltip,
              onPressed: () =>
                  _openGated(context, ref, SignInBenefit.saveLibrary, _openHub),
            ),
            IconButton(
              key: const Key('library-rating-deck'),
              icon: const Icon(Icons.swipe),
              tooltip: l10n.ratingDeckEntryTooltip,
              onPressed: () => _openGated(
                context,
                ref,
                SignInBenefit.earnPoints,
                _openRatingDeck,
              ),
            ),
            // The Community destination — the global, seasonal boards (change:
            // add-global-leaderboard). Gated like its neighbours: reading a board
            // needs an identity (the RPC is authenticated), so a signed-out tap
            // names the benefit and resumes here after signing in.
            IconButton(
              key: const Key('community-entry'),
              icon: const Icon(Icons.leaderboard_outlined),
              tooltip: l10n.communityEntryTooltip,
              onPressed: () => _openGated(
                context,
                ref,
                SignInBenefit.leaderboards,
                _openCommunity,
              ),
            ),
            if (signedIn)
              IconButton(
                icon: const Icon(Icons.library_music_outlined),
                tooltip: l10n.soundfontsEntryTooltip,
                onPressed: () => _openSoundFonts(context),
              ),
            // Help & tips: signed in, it's redundant with the account menu's
            // own "Aide et astuces" entry, so it only shows for a signed-out
            // session (change: add-welcome-onboarding, D5) — the account menu
            // there is just a "sign in" button with no help entry of its own.
            if (!signedIn)
              IconButton(
                key: const Key('library-help'),
                icon: const Icon(Icons.help_outline),
                tooltip: l10n.helpTitle,
                onPressed: () => openHelp(context),
              ),
            // The account control is the curator standing pill (change: add-
            // curation-rewards) — it replaces the plain person icon and opens the
            // account menu (→ profile, where the rewards live).
            const AccountMenu(),
            const SizedBox(width: 8),
          ],
        ),
        body: InstrumentChoiceListener(
          child: SafeArea(
            top: false,
            child: signedIn
                ? _SignedInBody(l10n: l10n)
                : _bundledBody(context, ref),
          ),
        ),
      ),
    );
  }

  /// Signed-out: the bundled demo catalog, grouped by level (open-only) and
  /// seeded from the instrument context (change: add-instrument-context) —
  /// drums shows the bundled grooves, keyboard the piano repertoire.
  Widget _bundledBody(BuildContext context, WidgetRef ref) {
    final catalog = ref.watch(scoreCatalogProvider);
    final wantsDrums =
        ref.watch(effectiveInstrumentContextProvider) == AppInstrument.drums;
    final shown = [
      for (final e in catalog)
        if ((e.instrument == ScoreInstrument.percussion) == wantsDrums) e,
    ];
    if (shown.isEmpty && wantsDrums) return const DrumsEmptyInvitation();
    return ListView(
      padding: const EdgeInsets.only(bottom: 24),
      children: _levelSections(
        context,
        shown,
        onOpen: (entry) => _open(context, ref, entry),
        actionFor: (_) => null,
      ),
    );
  }

  static void _open(BuildContext context, WidgetRef ref, CatalogEntry entry) {
    unawaited(openScore(context, ref, entry));
  }

  /// Runs an account-gated entry point: signed in, it opens straight away;
  /// signed out, it invites sign-in with [benefit] and — only if the user
  /// accepts and completes it — **resumes** by opening the screen. Declining
  /// returns the user to the library, never a dead end.
  static Future<void> _openGated(
    BuildContext context,
    WidgetRef ref,
    SignInBenefit benefit,
    void Function(BuildContext) open,
  ) async {
    if (!await inviteSignIn(context, ref, benefit)) return;
    if (!context.mounted) return;
    open(context);
  }

  static void _openHub(BuildContext context) => Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => const ScoreHubScreen()));

  static void _openRatingDeck(BuildContext context) => Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => const RatingDeckScreen()));

  static void _openSoundFonts(BuildContext context) => Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => const SoundFontsScreen()));

  static void _openCommunity(BuildContext context) => Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => const CommunityScreen()));
}

/// Groups [entries] into per-level card grids (only non-empty levels), in
/// beginner → advanced order.
List<Widget> _levelSections(
  BuildContext context,
  List<CatalogEntry> entries, {
  required void Function(CatalogEntry) onOpen,
  required Widget? Function(CatalogEntry) actionFor,
  bool Function(CatalogEntry)? offlineUnavailableFor,
}) {
  final l10n = AppLocalizations.of(context);
  final widgets = <Widget>[];
  for (final level in PracticeLevel.values) {
    final inLevel = entries.where((e) => e.level == level).toList();
    if (inLevel.isEmpty) continue;
    widgets.add(
      Padding(
        padding: const EdgeInsets.fromLTRB(16, 20, 16, 8),
        child: Text(
          _levelLabel(l10n, level),
          style: const TextStyle(
            color: CymbraColors.primary,
            fontSize: 15,
            fontWeight: FontWeight.w800,
            letterSpacing: 0.5,
          ),
        ),
      ),
    );
    widgets.add(
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
        itemCount: inLevel.length,
        itemBuilder: (context, i) => ScoreCard(
          entry: inLevel[i],
          onTap: () => onOpen(inLevel[i]),
          action: actionFor(inLevel[i]),
          offlineUnavailable: offlineUnavailableFor?.call(inLevel[i]) ?? false,
        ),
      ),
    );
  }
  return widgets;
}

/// Signed-in body: rating nudge, courses and the favorites (grouped by level,
/// each with a remove-from-favorites heart) in **one vertical scroll** — on a
/// phone the courses card alone ate most of the viewport, leaving the favorites
/// to scroll inside a sliver of screen. An empty state points to the hub.
class _SignedInBody extends ConsumerWidget {
  const _SignedInBody({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final favorites = ref.watch(favoriteScoresProvider);
    // Under the drums context (change: add-instrument-context) the home seeds
    // drums: favorites filter to percussion and the courses card — keyboard-only
    // for now — makes way; the explicit invitation covers the nothing-to-show
    // case.
    //
    // The bundled grooves are deliberately NOT listed here. They exist to make
    // the core loop playable with no account at all, which is the signed-out
    // body's job; once there is an account, the home is the catalog's — a
    // demo score sitting among a signed-in user's own favorites reads as
    // something they saved, and it cannot be favorited, rated or removed like
    // everything else on this screen.
    final wantsDrums =
        ref.watch(effectiveInstrumentContextProvider) == AppInstrument.drums;
    // Offline marking: a favorite with no cached bytes, while the device is
    // offline, is shown but flagged "not available offline". Both are read
    // non-blocking (default: online, everything playable) so they never gate the
    // list render (change: add-offline-score-cache).
    final online = ref.watch(isOnlineNowProvider).valueOrNull ?? true;
    final playable =
        ref.watch(offlinePlayableIdsProvider).valueOrNull ?? const <String>{};
    bool offlineUnavailable(CatalogEntry entry) =>
        !online && !playable.contains(entry.id);
    List<CatalogEntry> byContext(List<CatalogEntry> entries) => [
      for (final e in entries)
        if ((e.instrument == ScoreInstrument.percussion) == wantsDrums) e,
    ];
    return CustomScrollView(
      slivers: [
        // Nudge to rate scores after a lull (renders nothing when not due).
        const SliverToBoxAdapter(child: RatingInviteBanner()),
        // Daily free-open quota (change: add-score-daily-access-rewards):
        // renders nothing when the gate is off for this caller.
        const SliverToBoxAdapter(child: CatalogAccessChip()),
        // Interactive courses (change: add-notation-courses), above the
        // favorites; omits itself when there are none. No drum course exists
        // yet, so the card steps aside under the drums context.
        if (!wantsDrums) const SliverToBoxAdapter(child: CoursesSection()),
        switch (favorites) {
          AsyncData(:final value) when byContext(value).isEmpty => _fill(
            wantsDrums ? const DrumsEmptyInvitation() : _Empty(l10n: l10n),
          ),
          AsyncData(:final value) => SliverPadding(
            padding: const EdgeInsets.only(bottom: 24),
            sliver: SliverList.list(
              children: _levelSections(
                context,
                byContext(value),
                onOpen: (entry) => LibraryScreen._open(context, ref, entry),
                actionFor: (entry) => _FavoriteHeart(
                  onPressed: () => _removeFromFavorites(ref, entry),
                ),
                offlineUnavailableFor: offlineUnavailable,
              ),
            ),
          ),
          AsyncError() => _fill(
            wantsDrums ? const DrumsEmptyInvitation() : _Empty(l10n: l10n),
          ),
          _ => _fill(const Center(child: CircularProgressIndicator())),
        },
      ],
    );
  }

  /// A non-scrolling sliver that takes the space left below the courses card,
  /// so the empty/loading state stays centered instead of hugging the header.
  static Widget _fill(Widget child) =>
      SliverFillRemaining(hasScrollBody: false, child: child);

  /// Remove from favorites: for a saved catalog score, remove it from the
  /// library; for an upload, un-favorite it (the upload is kept, still in the
  /// hub's "mes partitions"). Never deletes anything here.
  void _removeFromFavorites(WidgetRef ref, CatalogEntry entry) {
    // Delegate to the owning notifier (which reloads itself + surfaces failures via
    // its state) — the UI never calls the service or invalidates a provider itself.
    if (entry.catalogId case final id?) {
      ref.read(savedCatalogScoresProvider.notifier).remove(id);
    } else if (entry.contributedId case final id?) {
      ref.read(myUploadsProvider.notifier).toggleFavorite(id, favorite: false);
    }
  }
}

/// A filled-heart button to remove an entry from favorites.
class _FavoriteHeart extends StatelessWidget {
  const _FavoriteHeart({required this.onPressed});

  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: const Icon(Icons.favorite, color: CymbraColors.error),
      tooltip: AppLocalizations.of(context).scoreHubRemoveFromLibrary,
      style: IconButton.styleFrom(
        backgroundColor: Colors.black.withValues(alpha: 0.28),
      ),
      onPressed: onPressed,
    );
  }
}

/// Empty-favorites state: a centered call-to-action linking to the Score Hub.
class _Empty extends StatelessWidget {
  const _Empty({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(
              Icons.favorite_border,
              size: 56,
              color: CymbraColors.outline,
            ),
            const SizedBox(height: 16),
            Text(
              l10n.libraryEmptyTitle,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 18,
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              l10n.libraryEmptyBody,
              textAlign: TextAlign.center,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
            const SizedBox(height: 20),
            FilledButton.icon(
              icon: const Icon(Icons.search),
              label: Text(l10n.libraryEmptyAction),
              onPressed: () => LibraryScreen._openHub(context),
            ),
          ],
        ),
      ),
    );
  }
}
