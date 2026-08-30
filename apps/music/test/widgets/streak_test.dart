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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/streak_service.dart';
import 'package:music/state/rating_activity_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/streak_notifier.dart';
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/widgets/curator_chip.dart';
import 'package:music/widgets/streak_listener.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import 'streak_test.mocks.dart';

@GenerateNiceMocks([
  MockSpec<StreakService>(),
  MockSpec<CuratorRewardsService>(),
])
/// A streak standing, defaulting to a healthy run secured today.
StreakView _streak({
  int current = 5,
  int longest = 12,
  bool playedToday = true,
  bool recoverable = false,
  int recoverCost = 0,
  int recoverableStreak = 0,
}) => StreakView(
  current: current,
  longest: longest,
  playedToday: playedToday,
  recoverable: recoverable,
  recoverCost: recoverCost,
  recoverableStreak: recoverableStreak,
);

/// A broken-but-recoverable 7-day streak costing 30 points.
StreakView _broken() => _streak(
  current: 7,
  playedToday: false,
  recoverable: true,
  recoverCost: 30,
  recoverableStreak: 7,
);

/// The curator standing the pill renders alongside the flame — irrelevant to
/// these tests, but it has to resolve or the pill never builds.
CuratorRewardsService _rewardsService() {
  final rewards = MockCuratorRewardsService();
  when(rewards.getRewards()).thenAnswer(
    (_) async => const CuratorRewardsView(
      lifetimePoints: 200,
      spendableBalance: 200,
      level: 2,
      levelFloor: 150,
      nextLevelAt: 350,
      totalRatings: 12,
      coverageContribution: 7,
      alignmentRate: 0.75,
      badges: [],
      recent: [],
    ),
  );
  return rewards;
}

List<Override> _overrides(
  StreakService streak, {
  bool online = true,
  PreferencesService? prefs,
  DateTime Function()? now,
}) => [
  streakServiceProvider.overrideWithValue(streak),
  curatorRewardsServiceProvider.overrideWithValue(_rewardsService()),
  preferencesServiceProvider.overrideWithValue(
    prefs ?? FakePreferencesService(),
  ),
  if (now != null) nowFnProvider.overrideWithValue(now),
  // The streak is server-owned account data: it is only read when signed in.
  canUseOnlineServicesProvider.overrideWithValue(online),
];

Widget _hostPill(StreakService service, {PreferencesService? prefs}) =>
    ProviderScope(
      overrides: _overrides(service, prefs: prefs),
      child: localizedApp(
        const Scaffold(body: Center(child: CuratorStandingPill())),
      ),
    );

Widget _hostListener(
  StreakService service, {
  PreferencesService? prefs,
  DateTime Function()? now,
}) => ProviderScope(
  overrides: _overrides(service, prefs: prefs, now: now),
  child: localizedApp(
    const StreakListener(child: Scaffold(body: SizedBox.shrink())),
  ),
);

void main() {
  group('standing pill', () {
    testWidgets('renders the flame with the current streak', (tester) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _streak(current: 5));
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('curator-chip-streak')), findsOneWidget);
      expect(find.byIcon(Icons.local_fire_department), findsOneWidget);
      expect(find.text('5'), findsOneWidget);
      // A live streak is lit, not muted.
      final flame = tester.widget<Icon>(
        find.byIcon(Icons.local_fire_department),
      );
      expect(flame.color, isNot(CymbraColors.onSurfaceVariant));
    });

    testWidgets('a zero streak stays visible, muted', (tester) async {
      // Spec: "a muted flame/hint (start a streak), not a hidden control".
      final zero = MockStreakService();
      when(zero.getStreak()).thenAnswer((_) async => StreakView.none);
      await tester.pumpWidget(_hostPill(zero));
      await tester.pumpAndSettle();

      // Present and readable, but visibly dimmed — the "start a streak" hint.
      // (The lit counterpart is asserted by the live-streak test above.)
      expect(find.byKey(const Key('curator-chip-streak')), findsOneWidget);
      expect(find.text('0'), findsOneWidget);
      final flame = tester.widget<Icon>(
        find.byIcon(Icons.local_fire_department),
      );
      expect(flame.color, CymbraColors.onSurfaceVariant);
    });

    testWidgets('a failed read falls back to the muted zero state', (
      tester,
    ) async {
      // A streak the server could not report must never break the app bar.
      final service = MockStreakService();
      when(service.getStreak()).thenThrow(Exception('offline'));
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('curator-chip-streak')), findsOneWidget);
      expect(find.text('0'), findsOneWidget);
    });
  });

  group('the streak sheet', () {
    testWidgets('the chip opens it whether or not a recovery is available', (
      tester,
    ) async {
      // Design D1: a control that only sometimes responds teaches players not
      // to try it. An intact streak opens the sheet too — it just has nothing
      // to offer.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _streak(current: 5));
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('curator-chip-streak-tap')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-sheet')), findsOneWidget);
      expect(find.byKey(const Key('streak-sheet-recovery')), findsNothing);
    });

    testWidgets('a recoverable break offers the buy-back with its cost', (
      tester,
    ) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      when(service.recover()).thenAnswer(
        (_) async => StreakRecoveryView(
          streak: _streak(current: 7, playedToday: true),
          newBalance: 70,
        ),
      );
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('curator-chip-streak-tap')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-sheet-recovery')), findsOneWidget);
      expect(find.textContaining('30'), findsWidgets);
      // Opening it spends nothing.
      verifyNever(service.recover());

      await tester.tap(find.byKey(const Key('streak-sheet-recover')));
      await tester.pumpAndSettle();
      verify(service.recover()).called(1);
    });

    testWidgets('an unaffordable break is shown disabled with the shortfall', (
      tester,
    ) async {
      // The wire flattens the server's decision into one bool, but reports what
      // the recovery WOULD cost when it refuses for balance — so "you are N
      // points short" is knowable, and silence would leave the player unable to
      // tell "too poor" from "too late".
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer(
        (_) async => _streak(
          current: 0,
          playedToday: false,
          recoverCost: 12, // reported despite recoverable: false
          recoverableStreak: 7,
        ),
      );
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('curator-chip-streak-tap')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-sheet-short')), findsOneWidget);
      final button = tester.widget<FilledButton>(
        find.byKey(const Key('streak-sheet-recover')),
      );
      expect(button.onPressed, isNull, reason: 'disabled, not hidden');
      verifyNever(service.recover());
    });

    testWidgets('a lapsed break offers nothing at all', (tester) async {
      // Past the grace window the wire reports no cost, so there is nothing to
      // explain and the honest answer is silence.
      final service = MockStreakService();
      when(
        service.getStreak(),
      ).thenAnswer((_) async => _streak(current: 0, playedToday: false));
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('curator-chip-streak-tap')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-sheet')), findsOneWidget);
      expect(find.byKey(const Key('streak-sheet-recovery')), findsNothing);
    });

    testWidgets('a guest is told to sign in rather than shown a zero', (
      tester,
    ) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => StreakView.none);
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(service, online: false),
          child: localizedApp(
            const Scaffold(body: Center(child: CuratorStandingPill())),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('curator-chip-streak-tap')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-sheet-guest')), findsOneWidget);
      expect(find.byKey(const Key('streak-sheet-recovery')), findsNothing);
    });

    testWidgets('becoming tappable leaves the chip rendering unchanged', (
      tester,
    ) async {
      // This chip is on every surface with a standing pill, so a regression in
      // its muted/lit states is a regression everywhere.
      final service = MockStreakService();
      when(
        service.getStreak(),
      ).thenAnswer((_) async => _streak(current: 5, playedToday: true));
      await tester.pumpWidget(_hostPill(service));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('curator-chip-streak')), findsOneWidget);
      expect(find.text('5'), findsOneWidget);
      final flame = tester.widget<Icon>(
        find.byIcon(Icons.local_fire_department),
      );
      expect(flame.color, CymbraColors.primary, reason: 'today secured = warm');
    });
  });

  group('the recovery cue', () {
    // The cue replaced a modal (change: make-streak-recovery-reachable). The
    // buy-back is use-it-or-lose-it — resuming restarts the run — so something
    // must still speak unprompted, but a modal fired the instant the standing
    // resolved, interrupting someone who came to practise; closing it to go and
    // play recorded a refusal that then destroyed the offer.

    testWidgets('a recoverable break is cued, and nothing is spent by it', (
      tester,
    ) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      // A cue, not a confirmation: it names the run and points at the chip.
      expect(find.byType(SnackBar), findsOneWidget);
      expect(find.textContaining('7'), findsWidgets);
      // The dialog is gone for good.
      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
      // Nothing here can debit anything — the confirmation lives with the money.
      expect(find.byKey(const Key('streak-recover-confirm')), findsNothing);
      verifyNever(service.recover());
    });

    testWidgets('the cue is raised once per break, across a relaunch', (
      tester,
    ) async {
      final prefs = FakePreferencesService();
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service, prefs: prefs));
      await tester.pumpAndSettle();
      // Written, not merely held in memory.
      expect(prefs.store[StreakRecoveryCue.prefsKey], '7');

      // A fresh app: same device storage, same break still on offer.
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pumpWidget(_hostListener(service, prefs: prefs));
      await tester.pumpAndSettle();

      expect(
        find.byType(SnackBar),
        findsNothing,
        reason: 'the interruption is spent; the offer is not',
      );
      verifyNever(service.recover());
    });

    testWidgets('a cued break is still recoverable from the chip', (
      tester,
    ) async {
      // The whole point of the change: silencing the cue must not withdraw the
      // offer. Saying "not now" should cost the interruption, not the option.
      final prefs = FakePreferencesService({StreakRecoveryCue.prefsKey: '7'});
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      when(service.recover()).thenAnswer(
        (_) async => StreakRecoveryView(
          streak: _streak(current: 7, playedToday: true),
          newBalance: 70,
        ),
      );
      await tester.pumpWidget(_hostPill(service, prefs: prefs));
      await tester.pumpAndSettle();

      // No cue — but the chip opens the offer.
      expect(find.byType(SnackBar), findsNothing);
      await tester.tap(find.byKey(const Key('curator-chip-streak-tap')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('streak-sheet-recovery')), findsOneWidget);

      await tester.tap(find.byKey(const Key('streak-sheet-recover')));
      await tester.pumpAndSettle();
      verify(service.recover()).called(1);
    });

    testWidgets('the same break is not cued again on a later day', (
      tester,
    ) async {
      // Guards a widening of `streak.grace_days`, which is a back-office flag:
      // a window wider than a day would re-raise the identical question every
      // morning under the old day-keyed record. Production reads `1` (checked
      // 2026-08-30), so this is protection rather than a fix for anything seen
      // — the beta report was a refusal lost to a `mounted` check.
      final prefs = FakePreferencesService({StreakRecoveryCue.prefsKey: '7'});
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(
        _hostListener(
          service,
          prefs: prefs,
          now: () => DateTime(2026, 8, 23, 9),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsNothing);
    });

    testWidgets('a genuinely new break is cued', (tester) async {
      // The other side of the rule: silencing one break must not silence every
      // future one.
      final prefs = FakePreferencesService({StreakRecoveryCue.prefsKey: '7'});
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer(
        (_) async => _streak(
          current: 3,
          playedToday: false,
          recoverable: true,
          recoverCost: 30,
          recoverableStreak: 3,
        ),
      );
      await tester.pumpWidget(_hostListener(service, prefs: prefs));
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsOneWidget);
    });

    testWidgets('an intact streak is never cued and offers nothing', (
      tester,
    ) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _streak());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsNothing);
      verifyNever(service.recover());
    });

    testWidgets('a break past the grace window is not cued', (tester) async {
      // The server reports it as non-recoverable AND as a zero live run — see
      // `display_streak`, "the LIVE run, zero once it is broken". A fixture that
      // reported the lapsed count instead would read to the client as a live
      // streak at risk, and pull in the at-risk nudge for the wrong reason.
      final service = MockStreakService();
      when(
        service.getStreak(),
      ).thenAnswer((_) async => _streak(current: 0, playedToday: false));
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsNothing);
      verifyNever(service.recover());
    });

    testWidgets('a refused recovery is reported without a raw error', (
      tester,
    ) async {
      // The outcome still reaches the player, even though the spend now starts
      // in the sheet and this listener is what reports it.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      when(
        service.recover(),
      ).thenThrow(Exception('gRPC FAILED_PRECONDITION: not enough points'));
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      await tester.runAsync(() async {});
      final ctx = tester.element(find.byType(Scaffold));
      final container = ProviderScope.containerOf(ctx);
      await container.read(streakProvider.notifier).recover();
      await tester.pumpAndSettle();

      verify(service.recover()).called(1);
      // A localized message, never the gRPC string.
      expect(find.textContaining('FAILED_PRECONDITION'), findsNothing);
      expect(find.textContaining('gRPC'), findsNothing);
    });
  });

  group('at-risk nudge', () {
    testWidgets('a live streak with no play today is nudged in-app', (
      tester,
    ) async {
      // The only reminder on platforms with no push token (design D4).
      final service = MockStreakService();
      when(
        service.getStreak(),
      ).thenAnswer((_) async => _streak(current: 4, playedToday: false));
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsOneWidget);
      expect(find.textContaining('4'), findsWidgets);
    });

    testWidgets('a streak already secured today is not nudged', (tester) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _streak());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsNothing);
    });

    testWidgets('the recovery cue replaces the nudge', (tester) async {
      // One interruption at a time: the cue IS the reminder. They are both
      // snackbars now, so this asserts on which one — a recoverable break is
      // never `atRisk` (the getter excludes it), so only the cue can speak.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byType(SnackBar), findsOneWidget);
      expect(find.textContaining('flame'), findsOneWidget);
      expect(find.textContaining('ends tonight'), findsNothing);
    });
  });

  group('streak notifier', () {
    test('recover reports through state, not a return value', () async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      when(service.recover()).thenAnswer(
        (_) async => StreakRecoveryView(
          streak: _streak(current: 7, playedToday: true),
          newBalance: 70,
        ),
      );
      final container = ProviderContainer(overrides: _overrides(service));
      addTearDown(container.dispose);

      await container.read(streakProvider.future);
      // The offer also waits on the persisted decline: no recorded "not this
      // time" here, so the question stands.
      await container.read(streakRecoveryCueProvider.future);
      expect(container.read(streakRecoveryCueDueProvider), isTrue);

      await container.read(streakProvider.notifier).recover();
      final after = container.read(streakProvider).requireValue;
      expect(after.current, 7);
      expect(after.playedToday, isTrue);
      expect(container.read(streakRecoveryCueDueProvider), isFalse);
    });

    test('a declined offer is withdrawn', () async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      final container = ProviderContainer(
        overrides: _overrides(service, now: () => DateTime(2026, 8, 23, 9)),
      );
      addTearDown(container.dispose);

      await container.read(streakProvider.future);
      await container.read(streakRecoveryCueProvider.future);
      expect(container.read(streakRecoveryCueDueProvider), isTrue);

      await container.read(streakRecoveryCueProvider.notifier).silence(7);

      expect(container.read(streakRecoveryCueDueProvider), isFalse);
      // The standing itself is untouched — the server still says it is
      // recoverable; only this device stopped asking.
      expect(container.read(streakProvider).requireValue.recoverable, isTrue);
    });

    test('the refusal outlives the day it was made on', () async {
      // The record must outlive the day it was made on, so that widening
      // `streak.grace_days` — a back-office flag, currently `1` — cannot bring
      // the same question back every morning.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      // One device: the same storage across both launches.
      final prefs = FakePreferencesService();
      final container = ProviderContainer(
        overrides: _overrides(
          service,
          prefs: prefs,
          now: () => DateTime(2026, 8, 23, 9),
        ),
      );
      addTearDown(container.dispose);
      await container.read(streakProvider.future);
      await container.read(streakRecoveryCueProvider.future);
      await container.read(streakRecoveryCueProvider.notifier).silence(7);
      expect(container.read(streakRecoveryCueDueProvider), isFalse);

      // A fresh launch, days later, on the SAME unresolved break: the recorded
      // refusal is read back from storage and still stands.
      final later = ProviderContainer(
        overrides: _overrides(
          service,
          prefs: prefs,
          now: () => DateTime(2026, 8, 26, 9),
        ),
      );
      addTearDown(later.dispose);
      await later.read(streakProvider.future);
      await later.read(streakRecoveryCueProvider.future);
      expect(
        later.read(streakRecoveryCueDueProvider),
        isFalse,
        reason: 'saying no ends the question, not just today\'s instance',
      );
    });

    test('a different break is a different question, and is asked', () async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer(
        (_) async => _streak(
          current: 3,
          playedToday: false,
          recoverable: true,
          recoverCost: 30,
          recoverableStreak: 3,
        ),
      );
      final container = ProviderContainer(overrides: _overrides(service));
      addTearDown(container.dispose);
      await container.read(streakProvider.future);
      await container.read(streakRecoveryCueProvider.future);
      // A refusal recorded against an earlier, longer run.
      await container.read(streakRecoveryCueProvider.notifier).silence(7);
      expect(
        container.read(streakRecoveryCueDueProvider),
        isTrue,
        reason: 'a new break the user has not answered yet is offered',
      );
    });

    test('no offer is derived before the decline has been read', () async {
      // Ordering matters: the standing resolves from the network, the decline
      // from disk. Offering while the decline is still loading is exactly how a
      // refused question comes back.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      final container = ProviderContainer(overrides: _overrides(service));
      addTearDown(container.dispose);

      await container.read(streakProvider.future);
      expect(
        container.read(streakRecoveryCueProvider),
        isA<AsyncLoading<int?>>(),
      );
      expect(container.read(streakRecoveryCueDueProvider), isFalse);
    });

    test('a refused recovery lands in the state as an error', () async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      when(service.recover()).thenThrow(Exception('grace elapsed'));
      final container = ProviderContainer(overrides: _overrides(service));
      addTearDown(container.dispose);

      await container.read(streakProvider.future);
      await container.read(streakProvider.notifier).recover();

      expect(container.read(streakProvider), isA<AsyncError<StreakView>>());
      // ...and no offer is derived from an errored read.
      expect(container.read(streakRecoveryCueDueProvider), isFalse);
    });

    test('refresh re-reads the server standing', () async {
      final service = MockStreakService();
      var calls = 0;
      when(service.getStreak()).thenAnswer((_) async {
        calls++;
        return _streak(current: calls);
      });
      final container = ProviderContainer(overrides: _overrides(service));
      addTearDown(container.dispose);

      expect((await container.read(streakProvider.future)).current, 1);
      await container.read(streakProvider.notifier).refresh();
      expect(container.read(streakProvider).requireValue.current, 2);
    });

    test('a signed-out session never reaches the server', () async {
      final service = MockStreakService();
      final container = ProviderContainer(
        overrides: _overrides(service, online: false),
      );
      addTearDown(container.dispose);

      expect(await container.read(streakProvider.future), StreakView.none);
      verifyNever(service.getStreak());
    });

    test('a delivered play makes the streak re-read itself', () async {
      // The outbox bumps the revision after the server acks; the streak reacts.
      final service = MockStreakService();
      var calls = 0;
      when(service.getStreak()).thenAnswer((_) async {
        calls++;
        return _streak(current: calls);
      });
      final container = ProviderContainer(overrides: _overrides(service));
      addTearDown(container.dispose);

      expect((await container.read(streakProvider.future)).current, 1);
      container.read(streakRevisionProvider.notifier).bump();
      expect((await container.read(streakProvider.future)).current, 2);
    });
  });
}
