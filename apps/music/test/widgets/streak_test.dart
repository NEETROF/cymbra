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

Widget _hostPill(StreakService service) => ProviderScope(
  overrides: _overrides(service),
  child: localizedApp(
    const Scaffold(body: Center(child: CuratorStandingPill())),
  ),
);

/// The same scope with no [StreakListener] — used to tear the listener's subtree
/// down while its dialog is still on screen.
Widget _hostBare(
  StreakService service, {
  PreferencesService? prefs,
  DateTime Function()? now,
}) => ProviderScope(
  overrides: _overrides(service, prefs: prefs, now: now),
  child: localizedApp(const Scaffold(body: SizedBox.shrink())),
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

  group('recovery flow', () {
    testWidgets('a broken streak is offered and restored on confirmation', (
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
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      // The offer names both what is restored and what it costs.
      expect(find.byKey(const Key('streak-recover-dialog')), findsOneWidget);
      expect(find.textContaining('7'), findsWidgets);
      expect(find.textContaining('30'), findsWidgets);
      // Nothing has been spent by merely showing it.
      verifyNever(service.recover());

      await tester.tap(find.byKey(const Key('streak-recover-confirm')));
      await tester.pumpAndSettle();

      verify(service.recover()).called(1);
      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
    });

    testWidgets('declining spends nothing', (tester) async {
      // The core "no silent debit" guarantee, from the user's side.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('streak-recover-dismiss')));
      await tester.pumpAndSettle();

      verifyNever(service.recover());
      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
    });

    testWidgets('declining is remembered across a relaunch', (tester) async {
      // The regression this guards: the decline used to live in the listener's
      // State, so the same question re-opened on every cold start (and on every
      // navigation to the other screen that mounts this listener) for as long as
      // the grace window held.
      final prefs = FakePreferencesService();
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service, prefs: prefs));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('streak-recover-dismiss')));
      await tester.pumpAndSettle();
      // The refusal is written, not just held in memory.
      expect(prefs.store[StreakRecoveryDecline.prefsKey], isNotNull);

      // A fresh app: same device storage, same standing still on offer.
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pumpWidget(_hostListener(service, prefs: prefs));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
      verifyNever(service.recover());
    });

    testWidgets('the same offer is not re-asked on a later day', (
      tester,
    ) async {
      // The beta report ("the popup comes back at every launch"): the decline
      // was filed under the calendar day, on the reasoning that the grace window
      // is one day wide — but that window is a back-office value, and a wider
      // one re-asked the identical question every morning. It is filed against
      // the offer now, so the standing break stays answered.
      final prefs = FakePreferencesService({
        StreakRecoveryDecline.prefsKey: '7', // the run _broken() would restore
      });
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

      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
    });

    testWidgets('a genuinely new break is asked about', (tester) async {
      // The other side of the same rule: refusing one offer must not silence
      // every future one.
      final prefs = FakePreferencesService({
        StreakRecoveryDecline.prefsKey: '7',
      });
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

      expect(find.byKey(const Key('streak-recover-dialog')), findsOneWidget);
    });

    testWidgets('the refusal is recorded even if the listener goes away', (
      tester,
    ) async {
      // The dialog outlives its listener: a route change, a rebuild, or the app
      // being backgrounded and killed all tear the subtree down while the
      // question is on screen. The refusal used to be written *after* a
      // `mounted` check, so it was simply lost and asked again next launch.
      final prefs = FakePreferencesService();
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service, prefs: prefs));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('streak-recover-dialog')), findsOneWidget);

      // The listener is gone; the dialog's own route is what answers.
      final dialogContext = tester.element(
        find.byKey(const Key('streak-recover-dismiss')),
      );
      await tester.pumpWidget(
        _hostBare(service, prefs: prefs),
        duration: Duration.zero,
      );
      Navigator.of(dialogContext).pop(false);
      await tester.pumpAndSettle();

      expect(
        prefs.store[StreakRecoveryDecline.prefsKey],
        '7',
        reason: 'the answer survives the widget that collected it',
      );
    });

    testWidgets('a refused recovery is reported without a raw error', (
      tester,
    ) async {
      // Insufficient balance / grace elapsed both come back as a failure.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      when(
        service.recover(),
      ).thenThrow(Exception('gRPC FAILED_PRECONDITION: not enough points'));
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('streak-recover-confirm')));
      await tester.pumpAndSettle();

      verify(service.recover()).called(1);
      // A localized message, never the gRPC string.
      expect(find.byType(SnackBar), findsOneWidget);
      expect(find.textContaining('FAILED_PRECONDITION'), findsNothing);
      expect(find.textContaining('gRPC'), findsNothing);
    });

    testWidgets('an intact streak never offers to spend anything', (
      tester,
    ) async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _streak());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
      verifyNever(service.recover());
    });

    testWidgets('a streak past the grace window is not offered', (
      tester,
    ) async {
      // The server reports it as non-recoverable; the app must not ask.
      final service = MockStreakService();
      when(
        service.getStreak(),
      ).thenAnswer((_) async => _streak(current: 7, playedToday: false));
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-recover-dialog')), findsNothing);
      verifyNever(service.recover());
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

    testWidgets('a recovery offer replaces the nudge', (tester) async {
      // One interruption at a time: the offer IS the reminder.
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      await tester.pumpWidget(_hostListener(service));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('streak-recover-dialog')), findsOneWidget);
      expect(find.byType(SnackBar), findsNothing);
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
      await container.read(streakRecoveryDeclineProvider.future);
      expect(container.read(streakRecoveryOfferedProvider), isTrue);

      await container.read(streakProvider.notifier).recover();
      final after = container.read(streakProvider).requireValue;
      expect(after.current, 7);
      expect(after.playedToday, isTrue);
      expect(container.read(streakRecoveryOfferedProvider), isFalse);
    });

    test('a declined offer is withdrawn', () async {
      final service = MockStreakService();
      when(service.getStreak()).thenAnswer((_) async => _broken());
      final container = ProviderContainer(
        overrides: _overrides(service, now: () => DateTime(2026, 8, 23, 9)),
      );
      addTearDown(container.dispose);

      await container.read(streakProvider.future);
      await container.read(streakRecoveryDeclineProvider.future);
      expect(container.read(streakRecoveryOfferedProvider), isTrue);

      await container.read(streakRecoveryDeclineProvider.notifier).decline(7);

      expect(container.read(streakRecoveryOfferedProvider), isFalse);
      // The standing itself is untouched — the server still says it is
      // recoverable; only this device stopped asking.
      expect(container.read(streakProvider).requireValue.recoverable, isTrue);
    });

    test('the refusal outlives the day it was made on', () async {
      // The beta report: the question came back at every launch. The decline was
      // filed under the calendar day, so a grace window wider than one day
      // re-asked the very same question every morning.
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
      await container.read(streakRecoveryDeclineProvider.future);
      await container.read(streakRecoveryDeclineProvider.notifier).decline(7);
      expect(container.read(streakRecoveryOfferedProvider), isFalse);

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
      await later.read(streakRecoveryDeclineProvider.future);
      expect(
        later.read(streakRecoveryOfferedProvider),
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
      await container.read(streakRecoveryDeclineProvider.future);
      // A refusal recorded against an earlier, longer run.
      await container.read(streakRecoveryDeclineProvider.notifier).decline(7);
      expect(
        container.read(streakRecoveryOfferedProvider),
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
        container.read(streakRecoveryDeclineProvider),
        isA<AsyncLoading<int?>>(),
      );
      expect(container.read(streakRecoveryOfferedProvider), isFalse);
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
      expect(container.read(streakRecoveryOfferedProvider), isFalse);
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
