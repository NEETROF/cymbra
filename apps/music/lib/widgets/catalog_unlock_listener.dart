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
import '../screens/open_score.dart';
import '../state/catalog_daily_access_notifier.dart';
import 'app_snackbar.dart';

/// Dedicated listener widget for the confirmed day-slot unlock (change:
/// add-score-daily-access-rewards; architecture rule 4 — `ref.listen` side
/// effects live in one place). It renders [child] and only wires the listener.
///
/// The unlock sheet fires `unlock(entry)` and closes; the outcome lands in
/// [catalogUnlockProvider] as state and is surfaced here: success → a snackbar
/// and the piece is re-opened (now served); insufficient points / any other
/// failure → a localized snackbar (never the raw server error).
class CatalogUnlockListener extends ConsumerWidget {
  const CatalogUnlockListener({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(catalogUnlockProvider.select((s) => s.seq), (previous, next) {
      if (previous == null || next == previous) return;
      final l10n = AppLocalizations.of(context);
      final messenger = ScaffoldMessenger.of(context);
      final state = ref.read(catalogUnlockProvider);
      final unlocked = state.unlocked;
      if (unlocked != null) {
        showAppSnackBar(messenger, l10n.catalogUnlocked);
        unawaited(openScore(context, ref, unlocked));
      } else if (state.insufficient) {
        showAppSnackBar(messenger, l10n.catalogUnlockInsufficient);
      } else if (state.error) {
        showAppSnackBar(messenger, l10n.catalogUnlockFailed);
      }
    });
    return child;
  }
}
