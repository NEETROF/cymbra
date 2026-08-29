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
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/state/plan_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/plan_withdrawal.dart';
import 'package:music/widgets/plan_listener.dart';

import '../support/localized.dart';

/// What the listener asked of the withdrawal — kept outside the notifier, whose
/// public API must stay `state` (riverpod_lint).
class _Calls {
  int withdrawals = 0;
  int armings = 0;
}

/// Records those calls, so the test pins the TRIGGER (the widget seam) rather
/// than the deletions themselves.
class _RecordingWithdrawal extends PlanWithdrawal {
  _RecordingWithdrawal(this._calls);

  final _Calls _calls;

  @override
  int build() => 0;

  @override
  void armForNextLapse() => _calls.armings++;

  @override
  Future<bool> withdraw() async {
    _calls.withdrawals++;
    return true;
  }
}

/// Serves plan answers on demand, the way the server would.
class _FakePlans extends Fake implements PlanService {
  _FakePlans(this.snapshot);
  PlanSnapshotView snapshot;
  @override
  Future<PlanSnapshotView> getMyPlan(AppPlatform platform) async => snapshot;
}

const _free = PlanSnapshotView(plan: 'free');
const _premium = PlanSnapshotView(plan: 'premium');

Future<_Calls> _pump(
  WidgetTester tester, {
  required PlanSnapshotView plan,
}) async {
  final calls = _Calls();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        planServiceProvider.overrideWithValue(_FakePlans(plan)),
        canUseOnlineServicesProvider.overrideWithValue(true),
        currentUserIdProvider.overrideWithValue('u1'),
        plansEnabledProvider.overrideWithValue(true),
        planWithdrawalProvider.overrideWith(() => _RecordingWithdrawal(calls)),
      ],
      child: localizedApp(
        const Scaffold(body: PlanListener(child: SizedBox())),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return calls;
}

void main() {
  testWidgets(
    'a cold start on a free plan withdraws — the lapse happened app-closed',
    (tester) async {
      // The regression: the trigger used to require an observed premium→free
      // transition, so the first answer of a launch (loading → free) withdrew
      // nothing and premium SoundFonts stayed on disk for good.
      final w = await _pump(tester, plan: _free);
      expect(w.withdrawals, 1);
      expect(w.armings, 0);
      // ... and the user is told (design D13 — a withdrawal is never silent).
      expect(find.byType(SnackBar), findsOneWidget);
    },
  );

  testWidgets('a premium answer arms the next lapse and withdraws nothing', (
    tester,
  ) async {
    final w = await _pump(tester, plan: _premium);
    expect(w.withdrawals, 0);
    expect(w.armings, 1);
  });
}
