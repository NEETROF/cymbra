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
import '../screens/plan_screen.dart';
import '../services/plan_service.dart';
import '../state/imported_soundfonts.dart' show libraryQuotaCueProvider;
import '../state/plan_notifier.dart';
import '../state/plan_withdrawal.dart';
import 'app_snackbar.dart';

/// Dedicated listener widget for the plan system (change: add-premium-
/// subscription; architecture rule 4 — `ref.listen` side effects live in one
/// place). It renders [child] and only wires two listeners:
///
/// - purchase-flow outcomes → a localized snackbar (never a raw store / gRPC
///   string), claimed once across stacked screens;
/// - the plan dropping from premium to free (trial ended, subscription lapsed,
///   revocation — the server's answer, never the device clock alone) → the
///   **withdrawal** of plan-only downloads (premium SoundFonts not owned, the
///   offline cache of catalog scores; own uploads kept) plus a notice (design D13
///   — never silent).
class PlanListener extends ConsumerWidget {
  const PlanListener({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(purchaseFlowProvider.select((s) => s.seq), (previous, next) {
      if (previous == null || next == previous) return;
      if (!ref.read(purchaseFlowProvider.notifier).claim(next)) return;
      final l10n = AppLocalizations.of(context);
      final messenger = ScaffoldMessenger.of(context);
      final msg = switch (ref.read(purchaseFlowProvider).outcome) {
        PurchaseOutcome.purchased => l10n.planPurchaseDone,
        PurchaseOutcome.restored => l10n.planRestoreDone,
        PurchaseOutcome.cancelled => null,
        PurchaseOutcome.pending => l10n.planPurchasePending,
        PurchaseOutcome.checkoutOpened => l10n.planCheckoutOpened,
        PurchaseOutcome.nothingToRestore => l10n.planRestoreNothing,
        PurchaseOutcome.failed => l10n.planPurchaseFailed,
        PurchaseOutcome.none => null,
      };
      if (msg != null) showAppSnackBar(messenger, msg);
    });

    // A private `.sf2` import refused by the plan's library quota (HTTP 403):
    // say so with the upsell — the import stays local-only.
    ref.listen(libraryQuotaCueProvider, (previous, next) {
      if (previous == null || next == previous) return;
      final l10n = AppLocalizations.of(context);
      final messenger = ScaffoldMessenger.of(context);
      messenger.showSnackBar(
        SnackBar(
          content: Text(l10n.libraryQuotaUpsell),
          action: SnackBarAction(
            label: l10n.planTitle,
            onPressed: () => openPlanScreen(context),
          ),
        ),
      );
    });

    // Premium → free on a SERVER answer: withdraw plan-only downloads once.
    ref.listen<AsyncValue<PlanSnapshotView>>(planProvider, (previous, next) {
      final before = previous?.valueOrNull;
      final after = next.valueOrNull;
      if (before == null || after == null) return;
      if (!before.isPremium || after.isPremium) return;
      if (!ref.read(plansEnabledProvider)) return;
      final l10n = AppLocalizations.of(context);
      final messenger = ScaffoldMessenger.of(context);
      unawaited(
        ref.read(planWithdrawalProvider.notifier).withdraw().then((did) {
          if (did) showAppSnackBar(messenger, l10n.planWithdrawnNotice);
        }),
      );
    });
    return child;
  }
}
