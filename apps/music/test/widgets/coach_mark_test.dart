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
import 'package:music/services/preferences_service.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/widgets/coach_mark.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';

/// A target control inside a screen, plus the overlay stacked over it — the same
/// shape as the real wiring (the layer sits above the routes).
class _SpotlightHost extends StatelessWidget {
  const _SpotlightHost({
    required this.hole,
    required this.passThrough,
    required this.onTargetTap,
    this.onNext,
    this.onSkip,
  });

  final Rect? hole;
  final bool passThrough;
  final VoidCallback onTargetTap;
  final VoidCallback? onNext;
  final VoidCallback? onSkip;

  @override
  Widget build(BuildContext context) => Stack(
    children: [
      Positioned.fromRect(
        rect: const Rect.fromLTWH(100, 200, 200, 60),
        child: GestureDetector(
          key: const Key('real-control'),
          behavior: HitTestBehavior.opaque,
          onTap: onTargetTap,
          child: const ColoredBox(color: Colors.blue),
        ),
      ),
      Positioned.fill(
        child: CoachMarkOverlay(
          hole: hole,
          title: 'Choose your piano sound',
          body: 'Pick the instrument you hear while playing.',
          nextLabel: 'Next',
          skipLabel: 'Skip',
          stepLabel: '1 / 3',
          onNext: onNext,
          onSkip: onSkip,
          passThrough: passThrough,
        ),
      ),
    ],
  );
}

Future<void> _pumpSpotlight(
  WidgetTester tester,
  Widget host, {
  Size size = const Size(1000, 700),
}) async {
  await tester.binding.setSurfaceSize(size);
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(localizedApp(Scaffold(body: host)));
  await tester.pumpAndSettle();
}

void main() {
  group('spotlight overlay', () {
    testWidgets('shows the copy and both actions', (tester) async {
      final events = <String>[];
      await _pumpSpotlight(
        tester,
        _SpotlightHost(
          hole: const Rect.fromLTWH(100, 200, 200, 60),
          passThrough: true,
          onTargetTap: () => events.add('control'),
          onNext: () => events.add('next'),
          onSkip: () => events.add('skip'),
        ),
      );

      expect(find.text('Choose your piano sound'), findsOneWidget);
      expect(find.text('1 / 3'), findsOneWidget);

      await tester.tap(find.byKey(const Key('coach-mark-next')));
      await tester.tap(find.byKey(const Key('coach-mark-skip')));
      expect(events, ['next', 'skip']);
    });

    testWidgets('a "do it now" step leaves the real control tappable', (
      tester,
    ) async {
      final events = <String>[];
      await _pumpSpotlight(
        tester,
        _SpotlightHost(
          hole: const Rect.fromLTWH(100, 200, 200, 60),
          passThrough: true,
          onTargetTap: () => events.add('control'),
          onNext: () => events.add('next'),
        ),
      );

      // Inside the cut-out: reaches the control underneath.
      await tester.tapAt(const Offset(200, 230));
      // Outside it: absorbed by the scrim, so nothing behind is triggered.
      await tester.tapAt(const Offset(800, 600));
      expect(events, ['control']);
    });

    testWidgets('a passive hint is dismissed by tapping anywhere', (
      tester,
    ) async {
      final events = <String>[];
      await _pumpSpotlight(
        tester,
        _SpotlightHost(
          hole: null,
          passThrough: false,
          onTargetTap: () => events.add('control'),
          onNext: () => events.add('next'),
        ),
      );

      await tester.tapAt(const Offset(200, 230));
      // The tap dismissed the hint instead of falling through to the control.
      expect(events, ['next']);
    });

    testWidgets('the bubble stays on screen on a short landscape viewport', (
      tester,
    ) async {
      await _pumpSpotlight(
        tester,
        const _SpotlightHost(
          // A target in the middle of a short viewport: neither vertical band
          // can hold the bubble, so it must move beside the target.
          hole: Rect.fromLTWH(100, 140, 200, 90),
          passThrough: true,
          onTargetTap: _noop,
        ),
        size: const Size(880, 360),
      );

      final bubble = tester.getRect(find.byKey(const Key('coach-mark-bubble')));
      expect(bubble.left, greaterThanOrEqualTo(0));
      expect(bubble.top, greaterThanOrEqualTo(0));
      expect(bubble.right, lessThanOrEqualTo(880));
      expect(bubble.bottom, lessThanOrEqualTo(360));
      expect(tester.takeException(), isNull); // no overflow
    });
  });

  group('one-time inline hint', () {
    Future<ProviderContainer> pumpHint(
      WidgetTester tester,
      FakePreferencesService prefs,
    ) async {
      final container = ProviderContainer(
        overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
      );
      addTearDown(container.dispose);
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(
            const Scaffold(body: CoachHintCallout(hint: CoachHint.rewards)),
          ),
        ),
      );
      await tester.pumpAndSettle();
      return container;
    }

    testWidgets('shows once, then never again after dismissal', (tester) async {
      final prefs = FakePreferencesService();
      await pumpHint(tester, prefs);

      expect(find.byKey(const Key('coach-hint-rewards')), findsOneWidget);
      expect(find.text('Points, badges and rewards'), findsOneWidget);

      await tester.tap(find.byKey(const Key('coach-hint-dismiss-rewards')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('coach-hint-rewards')), findsNothing);
      expect(prefs.store[CoachHint.rewards.prefsKey], 'true');
    });

    testWidgets('a returning user never sees it', (tester) async {
      await pumpHint(
        tester,
        FakePreferencesService({CoachHint.rewards.prefsKey: 'true'}),
      );
      expect(find.byKey(const Key('coach-hint-rewards')), findsNothing);
    });
  });
}

void _noop() {}
