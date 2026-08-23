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
import '../services/streak_service.dart';
import '../state/streak_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'app_snackbar.dart';

/// Dedicated listener widget for the practice streak (change: add-practice-
/// streak, tasks 5.2/5.3), mounted near the top of the home subtree.
///
/// It renders [child] and owns every streak side effect in one place
/// (architecture rule 4), rather than scattering `ref.listen` through build
/// methods:
///
/// * the **recovery offer** — when the server reports a broken-but-recoverable
///   streak, one confirmation asking whether to spend points. Nothing is ever
///   debited without that explicit yes (design D2), and a "not this time" is
///   remembered for the day (`streakRecoveryDeclineProvider`) rather than for the
///   life of this widget — two screens mount this listener, so a widget-local
///   flag re-asked on every navigation and on every cold start;
/// * the **at-risk nudge** — a quiet in-app cue for a live streak with no play
///   today. This is the only reminder Windows/Linux get (they have no push
///   token), and it is harmless duplication elsewhere;
/// * the **outcome** of a confirmed recovery, success or refusal, as a snackbar
///   whose text is localized (a raw server error is logged, never shown).
///
/// The recovery is fired and forgotten (rule 3): this widget never awaits the
/// notifier's return to decide anything — it reacts to the resulting state.
class StreakListener extends ConsumerStatefulWidget {
  const StreakListener({required this.child, super.key});

  final Widget child;

  @override
  ConsumerState<StreakListener> createState() => _StreakListenerState();
}

class _StreakListenerState extends ConsumerState<StreakListener> {
  /// Whether the confirmation is on screen right now. Purely a re-entrancy guard:
  /// the offer provider can emit again (a refresh, a delivered play) before the
  /// user has answered, and that must not stack a second dialog. Whether the
  /// question should be asked at all is the provider's call, not this flag's.
  bool _asking = false;

  /// The streak we asked the user to confirm, so the success message can name it
  /// after the state has already moved on.
  int _pendingRestore = 0;

  @override
  void initState() {
    super.initState();
    // `listenManual` + `fireImmediately`: the chip watches the same providers, so
    // by the time this subtree is (re-)entered the standing may already be
    // loaded. A plain `ref.listen` in build only sees CHANGES, so a recovery
    // offer would appear on the very first load and never again.
    ref.listenManual(streakRecoveryOfferedProvider, fireImmediately: true, (
      _,
      offered,
    ) {
      if (offered) _maybeOfferRecovery();
    });
    ref.listenManual(streakProvider, fireImmediately: true, (previous, next) {
      final streak = next.valueOrNull;
      if (streak != null) _maybeNudge(streak);
      _reportOutcome(previous, next);
    });
  }

  @override
  Widget build(BuildContext context) => widget.child;

  /// Run [action] after the current frame. Every effect here (a dialog, a
  /// snackbar) mutates the widget tree, and `fireImmediately` can deliver during
  /// build — so nothing is done inline.
  void _afterFrame(VoidCallback action) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) action();
    });
  }

  void _maybeOfferRecovery() {
    final streak = ref.read(streakProvider).valueOrNull;
    if (_asking || streak == null || !streak.recoverable) return;
    _asking = true;
    _pendingRestore = streak.recoverableStreak;
    _afterFrame(() => unawaited(_confirmRecovery(streak)));
  }

  Future<void> _confirmRecovery(StreakView streak) async {
    final l10n = AppLocalizations.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        key: const Key('streak-recover-dialog'),
        backgroundColor: CymbraColors.surfaceContainerHigh,
        title: Text(l10n.streakRecoverTitle),
        content: Text(
          l10n.streakRecoverBody(streak.recoverableStreak, streak.recoverCost),
        ),
        actions: [
          TextButton(
            key: const Key('streak-recover-dismiss'),
            onPressed: () => Navigator.of(context).pop(false),
            child: Text(l10n.streakRecoverDismiss),
          ),
          FilledButton(
            key: const Key('streak-recover-confirm'),
            onPressed: () => Navigator.of(context).pop(true),
            child: Text(l10n.streakRecoverConfirm(streak.recoverCost)),
          ),
        ],
      ),
    );
    _asking = false;
    if (!mounted) return;
    // Declining spends nothing and asks nothing further — the streak simply
    // lapses at the end of the grace window. Recorded for the day so neither a
    // relaunch nor the other screen re-opens the same question.
    if (confirmed != true) {
      unawaited(
        ref.read(streakRecoveryDeclineProvider.notifier).declineToday(),
      );
      return;
    }
    // Fire the action; the outcome arrives as state, not as a return value.
    unawaited(ref.read(streakProvider.notifier).recover());
  }

  void _maybeNudge(StreakView streak) {
    if (ref.read(streakNudgeShownProvider) || _asking || !streak.atRisk) return;
    ref.read(streakNudgeShownProvider.notifier).mark();
    _afterFrame(
      () => showAppSnackBar(
        ScaffoldMessenger.of(context),
        AppLocalizations.of(context).streakAtRiskNudge(streak.current),
      ),
    );
  }

  /// Surface the result of a recovery the user confirmed. Only reacts to a
  /// transition out of `loading` on a spend we started, so an ordinary reload
  /// never reports anything.
  void _reportOutcome(
    AsyncValue<StreakView>? previous,
    AsyncValue<StreakView> next,
  ) {
    if (_pendingRestore == 0 || previous is! AsyncLoading) return;
    final restored = _pendingRestore;
    switch (next) {
      case AsyncData(:final value) when value.playedToday:
        _pendingRestore = 0;
        _afterFrame(
          () => showAppSnackBar(
            ScaffoldMessenger.of(context),
            AppLocalizations.of(context).streakRecovered(restored),
          ),
        );
      case AsyncError(:final error):
        // Refused (grace window elapsed, balance moved, offline). The cause is
        // logged; the user sees a localized message, never a gRPC string.
        debugPrint('streak recovery failed: $error');
        _pendingRestore = 0;
        _afterFrame(
          () => showAppSnackBar(
            ScaffoldMessenger.of(context),
            AppLocalizations.of(context).streakRecoverFailed,
          ),
        );
      case _:
        break;
    }
  }
}
