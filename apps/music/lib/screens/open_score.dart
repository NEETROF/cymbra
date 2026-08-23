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
import '../state/notation_data.dart';
import '../state/notation_notifier.dart';
import '../state/score_catalog.dart';
import '../state/score_preview_playback.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/catalog_unlock_sheet.dart';
import 'drums_not_playable_screen.dart';
import 'player_screen.dart';
import 'score_load_message.dart';

/// Minimum time the loading animation stays on screen, so a fast (local) load
/// still shows it instead of flickering.
const _minSpinnerVisible = Duration(milliseconds: 550);

/// Opens [entry] in the player, guarded by a pre-flight load: the score is
/// selected and fetched/parsed behind a blocking spinner, and the player is only
/// pushed once the notation has loaded. On failure the user stays put and gets a
/// localized snackbar — never a broken game screen showing a raw error.
///
/// The pre-flight subscription is kept alive for the player's lifetime, so the
/// score loads exactly once (the pushed player re-uses this already-loaded
/// state) and is disposed when the player is popped.
///
/// The returned future completes when the player is **left**, so a caller that
/// has something to do afterwards (the no-account try offering sign-in) can
/// simply await it; callers that just open a score keep ignoring it.
Future<void> openScore(
  BuildContext context,
  WidgetRef ref,
  CatalogEntry entry,
) async {
  final l10n = AppLocalizations.of(context);
  final messenger = ScaffoldMessenger.of(context);
  final navigator = Navigator.of(context);
  final rootNavigator = Navigator.of(context, rootNavigator: true);

  // INTERIM guard (change: add-drums-access, removed by `add-drum-kit-view`):
  // a percussion score has no presentation yet — show the localized "drums not
  // playable yet" state instead of entering the player, which would render the
  // GM key numbers as falling piano notes and sound them on the piano synth.
  if (entry.instrument == ScoreInstrument.percussion) {
    await navigator.push(
      MaterialPageRoute<void>(
        builder: (_) => DrumsNotPlayableScreen(title: entry.title),
      ),
    );
    return;
  }

  // Opening a piece silences any teaser audition (change:
  // add-score-daily-access-rewards): the clip and the synth share the engine.
  unawaited(ref.read(scorePreviewPlaybackProvider.notifier).stop());
  ref.read(selectedScoreProvider.notifier).select(entry);

  final completer = Completer<bool>();
  final sub = ref.listenManual<NotationData>(notationProvider, (_, next) {
    if (completer.isCompleted) return;
    if (next.failure != null) {
      completer.complete(false);
    } else if (next.hasDocument) {
      completer.complete(true);
    }
  }, fireImmediately: true);

  // Blocking progress while the score pre-loads — a spinner + label in a card so
  // it reads as a real loading step. The wait is escapable (change:
  // add-client-transport-deadlines): the cancel control clears the selection so
  // the load's stale-entry guards discard the late result, and completes the
  // wait with `cancelled` set. It does NOT pop the dialog itself — the main
  // flow below owns the single pop; a second one would dismiss the underlying
  // screen. The barrier stays non-dismissible: an accidental tap outside must
  // not cancel a load.
  var cancelled = false;
  unawaited(
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => Center(
        child: Card(
          color: CymbraColors.surfaceContainerHigh,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const CircularProgressIndicator(),
                const SizedBox(height: 16),
                Text(
                  l10n.playerScoreLoading,
                  style: const TextStyle(color: CymbraColors.onSurfaceVariant),
                ),
                const SizedBox(height: 8),
                TextButton(
                  onPressed: () {
                    cancelled = true;
                    ref.read(selectedScoreProvider.notifier).clear();
                    if (!completer.isCompleted) completer.complete(false);
                  },
                  child: Text(l10n.cancel),
                ),
              ],
            ),
          ),
        ),
      ),
    ),
  );

  // Keep the animation visible for a minimum time so a fast load doesn't flash.
  await Future.wait([
    completer.future,
    Future<void>.delayed(_minSpinnerVisible),
  ]);
  final loaded = await completer.future;
  // Dismiss the progress dialog (pushed on the root navigator).
  if (rootNavigator.canPop()) rootNavigator.pop();

  if (cancelled) {
    // A user decision, not a failure: no banner, stay on the current screen.
    // The selection is already cleared, so the in-flight load's guards discard
    // its late result whether it succeeds or fails.
    sub.close();
    return;
  }
  if (loaded) {
    await navigator
        .push(MaterialPageRoute<void>(builder: (_) => const PlayerScreen()))
        .whenComplete(sub.close);
  } else {
    // Surface the specific (but localized) cause — missing, not-ready, offline…
    final failure =
        ref.read(notationProvider).failure ?? ScoreLoadFailure.generic;
    sub.close();
    if (failure == ScoreLoadFailure.locked) {
      // Refused by the daily quota (change: add-score-daily-access-rewards):
      // not an error — offer the teaser / the points unlock / the upsell.
      if (context.mounted) await showCatalogUnlockSheet(context, ref, entry);
      return;
    }
    showAppSnackBar(messenger, scoreLoadFailureMessage(l10n, failure));
  }
}
