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
import '../state/contributed_scores.dart';
import '../state/saved_catalog_scores.dart';
import 'app_snackbar.dart';
import 'catalog_unlock_listener.dart';
import 'plan_listener.dart';
import 'streak_listener.dart';

/// Dedicated listener widget for the library/hub subtree (architecture rule 4:
/// isolate `ref.listen` side effects in one place instead of scattering them
/// through build methods). It renders [child] and only wires listeners.
///
/// Since the favorite/delete/remove mutations are fire-and-forget (rule 3 — the UI
/// does not await their return), a failure lands in the owning provider's
/// [AsyncValue] and is surfaced here as a snackbar. Per the "no raw technical
/// errors in UI" rule, the message is generic and the cause is only logged.
class LibraryListeners extends ConsumerWidget {
  const LibraryListeners({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(myUploadsProvider, (_, next) => _surfaceError(context, next));
    ref.listen(
      savedCatalogScoresProvider,
      (_, next) => _surfaceError(context, next),
    );
    // The outcome of a public-catalog proposal (change: add-score-catalog-proposal):
    // its own channel, so a refusal is phrased for what it is instead of falling into
    // the generic mutation-failed message — and without emptying the uploads list.
    ref.listen(scoreProposalFeedbackProvider, (_, next) {
      if (next == null) return;
      _surfaceProposal(context, next);
      ref.read(scoreProposalFeedbackProvider.notifier).clear();
    });
    // The streak's own effects (recovery offer, at-risk nudge, outcome) and the
    // day-slot unlock outcome (change: add-score-daily-access-rewards) live in
    // their own listeners rather than here — one concern per listener widget.
    return PlanListener(
      child: CatalogUnlockListener(child: StreakListener(child: child)),
    );
  }

  void _surfaceError(BuildContext context, AsyncValue<Object?> value) {
    if (value case AsyncError(:final error)) {
      debugPrint('library action failed: $error'); // logged, never shown raw
      showAppSnackBar(
        ScaffoldMessenger.of(context),
        AppLocalizations.of(context).libraryActionFailed,
      );
    }
  }

  void _surfaceProposal(BuildContext context, ScoreProposalOutcome outcome) {
    final l10n = AppLocalizations.of(context);
    showAppSnackBar(ScaffoldMessenger.of(context), switch (outcome) {
      ScoreProposalOutcome.submitted => l10n.scoreProposeDone,
      ScoreProposalOutcome.alreadyInCatalog => l10n.scoreProposeAlreadyDone,
      ScoreProposalOutcome.failed => l10n.scoreProposeError,
    });
  }
}
