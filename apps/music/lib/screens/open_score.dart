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
import 'player_screen.dart';

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
  CatalogEntry entry,
) async {
  final l10n = AppLocalizations.of(context);
  final messenger = ScaffoldMessenger.of(context);
  final navigator = Navigator.of(context);
  final rootNavigator = Navigator.of(context, rootNavigator: true);

  ref.read(selectedScoreProvider.notifier).select(entry);

  final completer = Completer<bool>();
  final sub = ref.listenManual<NotationData>(notationProvider, (_, next) {
    if (completer.isCompleted) return;
    if (next.error != null) {
      completer.complete(false);
    } else if (next.hasDocument) {
      completer.complete(true);
    }
  }, fireImmediately: true);

  // Blocking progress while the score pre-loads.
  unawaited(
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => const Center(child: CircularProgressIndicator()),
    ),
  );

  final loaded = await completer.future;
  // Dismiss the progress dialog (pushed on the root navigator).
  if (rootNavigator.canPop()) rootNavigator.pop();

  if (loaded) {
    navigator
        .push(
          MaterialPageRoute<void>(builder: (_) => const PlayerScreen()),
        )
        .whenComplete(sub.close);
  } else {
    sub.close();
    messenger.showSnackBar(SnackBar(content: Text(l10n.playerScoreLoadError)));
  }
}
