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
import '../courses/course_manifest.dart';
import '../services/catalog_service.dart';
import '../state/lesson_context.dart';
import '../state/notation_data.dart';
import '../state/notation_notifier.dart';
import '../state/score_catalog.dart';
import '../state/saved_catalog_scores.dart';
import '../state/score_preview_playback.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/catalog_unlock_sheet.dart';
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
/// simply await it; callers that just open a score keep ignoring it. It
/// resolves to the load failure when the player never opened, null otherwise
/// (opened, or the wait was cancelled) — a lesson's piece step reads it
/// (change: add-music-piece-lessons).
Future<ScoreLoadFailure?> openScore(
  BuildContext context,
  WidgetRef ref,
  CatalogEntry entry,
) async {
  final l10n = AppLocalizations.of(context);
  final messenger = ScaffoldMessenger.of(context);
  final navigator = Navigator.of(context);
  final rootNavigator = Navigator.of(context, rootNavigator: true);

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
    return null;
  }
  if (loaded) {
    await navigator
        .push(MaterialPageRoute<void>(builder: (_) => const PlayerScreen()))
        .whenComplete(sub.close);
    return null;
  } else {
    // Surface the specific (but localized) cause — missing, not-ready, offline…
    final failure =
        ref.read(notationProvider).failure ?? ScoreLoadFailure.generic;
    sub.close();
    if (failure == ScoreLoadFailure.locked) {
      // Refused by the daily quota (change: add-score-daily-access-rewards):
      // not an error — offer the teaser / the points unlock / the upsell.
      if (context.mounted) await showCatalogUnlockSheet(context, ref, entry);
      return failure;
    }
    showAppSnackBar(messenger, scoreLoadFailureMessage(l10n, failure));
    return failure;
  }
}

/// How a lesson's piece step's attempt to open its piece ended (change:
/// add-music-piece-lessons).
enum PieceOpenOutcome {
  /// The player opened (and has been left since).
  opened,

  /// The piece is unknown, gone, or could not be loaded.
  missing,

  /// Offline, and the piece has no cached copy.
  offline,
}

/// Resolves the score a piece step names into a playable library entry: a
/// bundled score from the shipped catalog, or a catalogue piece looked up by
/// id. Null when unknown or unreachable.
Future<CatalogEntry?> resolvePieceEntry(
  WidgetRef ref,
  PieceScoreRef score,
) async {
  if (score.bundled case final id?) {
    return ref.read(scoreCatalogProvider).where((e) => e.id == id).firstOrNull;
  }
  if (score.catalog case final id?) {
    try {
      return catalogEntryFromHit(
        await ref.read(catalogScoreLookupProvider).getCatalogScore(id),
      );
    } catch (_) {
      return null;
    }
  }
  return null;
}

/// Opens the piece a lesson step names in the full player, with the lesson's
/// imposed settings armed for that run (change: add-music-piece-lessons). The
/// context is armed right before the player builds and cleared when it is
/// left, so free play is never touched by it. Completes when the player is
/// left (or could not open).
Future<PieceOpenOutcome> openScoreForLesson(
  BuildContext context,
  WidgetRef ref, {
  required PieceScoreRef score,
  required LessonContext lesson,
}) async {
  final entry = await resolvePieceEntry(ref, score);
  if (entry == null) return PieceOpenOutcome.missing;
  if (!context.mounted) return PieceOpenOutcome.missing;
  final pending = ref.read(pendingLessonContextProvider.notifier);
  pending.arm(lesson);
  try {
    final failure = await openScore(context, ref, entry);
    return switch (failure) {
      null => PieceOpenOutcome.opened,
      ScoreLoadFailure.offlineUnavailable => PieceOpenOutcome.offline,
      _ => PieceOpenOutcome.missing,
    };
  } finally {
    pending.clear();
  }
}
