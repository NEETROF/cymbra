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
///   remembered against the offer itself (`streakRecoveryDeclineProvider`)
///   rather than for the life of this widget — two screens mount this listener,
///   so a widget-local flag re-asked on every navigation and on every cold
///   start;
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
  /// Whether the recovery cue has already been raised by THIS listener instance.
  /// Purely a re-entrancy guard: the offer provider can emit again (a refresh, a
  /// delivered play) within one mount, and that must not stack a second
  /// snackbar. Whether the cue should be raised at all is the provider's call —
  /// and across mounts and launches it is the recorded refusal that answers,
  /// not this flag.
  bool _cued = false;

  @override
  void initState() {
    super.initState();
    // `listenManual` + `fireImmediately`: the chip watches the same providers, so
    // by the time this subtree is (re-)entered the standing may already be
    // loaded. A plain `ref.listen` in build only sees CHANGES, so a recovery
    // offer would appear on the very first load and never again.
    ref.listenManual(streakRecoveryCueDueProvider, fireImmediately: true, (
      _,
      due,
    ) {
      if (due) _cueRecovery();
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

  /// Says, unprompted, that a recovery is on the table — and stops there.
  ///
  /// It used to open a confirmation (change: make-streak-recovery-reachable).
  /// The deadline is real and invisible: resuming restarts the run, so the offer
  /// dies on the next play, and something has to say so or it passes unseen.
  /// But a modal fires the instant the standing resolves — it interrupts someone
  /// who came to practise, and closing it to go and play recorded a refusal that
  /// then destroyed the offer. So it is a cue now, in the same register as the
  /// at-risk nudge below, pointing at the chip that can actually take the spend.
  ///
  /// Nothing here can debit anything. The confirmation lives with the money, in
  /// the sheet.
  void _cueRecovery() {
    final streak = ref.read(streakProvider).valueOrNull;
    if (_cued || streak == null || !streak.recoverable) return;
    // Recorded against THIS offer, not this session: the two screens that mount
    // this listener must not each raise it, and a relaunch inside the same
    // break must not either.
    _cued = true;
    final cue = ref.read(streakRecoveryCueProvider.notifier);
    unawaited(cue.silence(streak.recoverableStreak));
    _afterFrame(
      () => showAppSnackBar(
        ScaffoldMessenger.of(context),
        AppLocalizations.of(
          context,
        ).streakCueRecoverable(streak.recoverableStreak),
      ),
    );
  }

  void _maybeNudge(StreakView streak) {
    if (ref.read(streakNudgeShownProvider) || !streak.atRisk) return;
    ref.read(streakNudgeShownProvider.notifier).mark();
    _afterFrame(
      () => showAppSnackBar(
        ScaffoldMessenger.of(context),
        AppLocalizations.of(context).streakAtRiskNudge(streak.current),
      ),
    );
  }

  /// Surface the result of a recovery the user confirmed. Only reacts to a
  /// transition out of `loading` on a spend that was actually started, so an
  /// ordinary reload — which also passes through `loading` — never reports
  /// anything.
  ///
  /// The marker is armed by [Streak.recover] itself rather than by whoever
  /// called it: the spend starts in the streak sheet now, and this listener is
  /// what says how it went (change: make-streak-recovery-reachable).
  void _reportOutcome(
    AsyncValue<StreakView>? previous,
    AsyncValue<StreakView> next,
  ) {
    final pending = ref.read(streakRecoveryPendingProvider.notifier);
    final restored = ref.read(streakRecoveryPendingProvider);
    if (restored == 0 || previous is! AsyncLoading) return;
    switch (next) {
      case AsyncData(:final value) when value.playedToday:
        pending.clear();
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
        pending.clear();
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
