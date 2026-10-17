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
import '../state/daily_session_notifier.dart';
import '../state/home_door.dart';
import '../state/today_ledger.dart';
import 'reward_celebration.dart';

/// Dedicated listener for the daily session's one UI side effect (change:
/// add-music-daily-session, D5): when today's done count reaches the goal,
/// the completion moment is raised once for the day — on the shared
/// celebration surface (change: add-music-motion-and-celebrations). The
/// moment waits for this route to be current again, so it never stacks over
/// the lesson player's own celebration.
class DailySessionListener extends ConsumerStatefulWidget {
  const DailySessionListener({super.key, required this.child});

  final Widget child;

  @override
  ConsumerState<DailySessionListener> createState() =>
      _DailySessionListenerState();
}

class _DailySessionListenerState extends ConsumerState<DailySessionListener> {
  bool _pending = false;
  bool _raising = false;

  void _maybeRaise() {
    if (!_pending || _raising) return;
    if (!(ModalRoute.of(context)?.isCurrent ?? true)) return;
    _raising = true;
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      if (!mounted) return;
      _pending = false;
      final ledger = ref.read(dailyLedgerProvider);
      final session = ref.read(dailySessionProvider).valueOrNull;
      if (ledger.cueRaised || session == null || !session.completedToday) {
        _raising = false;
        return;
      }
      await ref.read(dailyLedgerProvider.notifier).markCueRaised();
      if (!mounted) return;
      final l10n = AppLocalizations.of(context);
      await showRewardCelebration(
        context,
        title: l10n.dailySessionCompleteTitle,
        message: l10n.dailySessionCompleteMessage(session.doneCount),
        icon: Icons.today,
      );
      _raising = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    // Only while the Learn door is open: the session is composed there, and
    // listening from the Play door would fetch the catalogue for nothing (a
    // guest who never learned stays offline).
    if (ref.watch(effectiveHomeDoorProvider) == HomeDoor.learn) {
      ref.listen(dailySessionProvider, (previous, next) {
        final session = next.valueOrNull;
        if (session == null || !session.completedToday) return;
        if (ref.read(dailyLedgerProvider).cueRaised) return;
        _pending = true;
        _maybeRaise();
      });
    }
    // `ModalRoute.of` is a dependency: this rebuilds when the route becomes
    // current again, which is when a deferred moment gets its turn.
    _maybeRaise();
    return widget.child;
  }
}
