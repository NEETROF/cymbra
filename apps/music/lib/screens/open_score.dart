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
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
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
Future<void> openScore(
  BuildContext context,
  WidgetRef ref,
  CatalogEntry entry, {
  bool preview = false,
}) async {
  final l10n = AppLocalizations.of(context);
  final messenger = ScaffoldMessenger.of(context);
  final navigator = Navigator.of(context);
  final rootNavigator = Navigator.of(context, rootNavigator: true);

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
  // it reads as a real loading step.
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

  if (loaded) {
    navigator
        .push(
          MaterialPageRoute<void>(
            builder: (_) => PlayerScreen(preview: preview),
          ),
        )
        .whenComplete(sub.close);
  } else {
    // Surface the specific (but localized) cause — missing, not-ready, offline…
    final failure =
        ref.read(notationProvider).failure ?? ScoreLoadFailure.generic;
    sub.close();
    showAppSnackBar(messenger, scoreLoadFailureMessage(l10n, failure));
  }
}

/// Opens [entry] in the player's **read-only preview** (change:
/// add-app-score-rating): the same pre-flight load + render as [openScore], but
/// the player mounts in preview mode (notation renders and notes sound, with no
/// interaction or scoring) and stopping/returning pops back to the caller (the
/// rating deck). Thin wrapper over [openScore] so both share the load guard.
Future<void> openScorePreview(
  BuildContext context,
  WidgetRef ref,
  CatalogEntry entry,
) => openScore(context, ref, entry, preview: true);
