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
import '../state/favorite_scores.dart';
import '../state/player_preferences.dart';
import '../state/saved_catalog_scores.dart';
import '../state/score_catalog.dart';
import '../state/session_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/library_listeners.dart';
import '../widgets/rating_invite_banner.dart';
import '../widgets/score_card.dart';
import 'auth/account_menu.dart';
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

    return LibraryListeners(
      child: Scaffold(
        backgroundColor: CymbraColors.background,
        appBar: AppBar(
          title: Text(l10n.libraryTitle),
          backgroundColor: CymbraColors.surfaceContainerLowest,
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
            if (signedIn)
              IconButton(
                icon: const Icon(Icons.library_music_outlined),
                tooltip: l10n.soundfontsEntryTooltip,
                onPressed: () => _openSoundFonts(context),
              ),
            // Help & tips: the stable entry that makes the one-time hints
            // re-findable (change: add-welcome-onboarding, D5). Shown to every
            // session — including a signed-out one browsing the bundled scores.
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
        body: SafeArea(
          top: false,
          child: signedIn
              ? Column(
                  children: [
                    // Nudge to rate scores after a lull (renders nothing when not
                    // due), pinned above the favorites list.
                    const RatingInviteBanner(),
                    Expanded(child: _FavoritesBody(l10n: l10n)),
                  ],
                )
              : _bundledBody(context, ref),
        ),
      ),
    );
  }

  /// Signed-out: the bundled demo catalog, grouped by level (open-only).
  Widget _bundledBody(BuildContext context, WidgetRef ref) {
    final catalog = ref.watch(scoreCatalogProvider);
    return ListView(
      padding: const EdgeInsets.only(bottom: 24),
      children: _levelSections(
        context,
        catalog,
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
}

/// Groups [entries] into per-level card grids (only non-empty levels), in
/// beginner → advanced order.
List<Widget> _levelSections(
  BuildContext context,
  List<CatalogEntry> entries, {
  required void Function(CatalogEntry) onOpen,
  required Widget? Function(CatalogEntry) actionFor,
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
        ),
      ),
    );
  }
  return widgets;
}

/// Signed-in body: the favorites, grouped by level, with a remove-from-favorites
/// heart on each; an empty state points to the hub.
class _FavoritesBody extends ConsumerWidget {
  const _FavoritesBody({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final favorites = ref.watch(favoriteScoresProvider);
    return switch (favorites) {
      AsyncData(:final value) when value.isEmpty => _Empty(l10n: l10n),
      AsyncData(:final value) => ListView(
        padding: const EdgeInsets.only(bottom: 24),
        children: _levelSections(
          context,
          value,
          onOpen: (entry) => LibraryScreen._open(context, ref, entry),
          actionFor: (entry) =>
              _FavoriteHeart(onPressed: () => _removeFromFavorites(ref, entry)),
        ),
      ),
      AsyncError() => _Empty(l10n: l10n),
      _ => const Center(child: CircularProgressIndicator()),
    };
  }

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
