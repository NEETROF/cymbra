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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/coaching_notifier.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container(FakePreferencesService prefs) {
  final c = ProviderContainer(
    overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
  );
  // Hold a subscription so the notifier builds now (kicking off the restore).
  final sub = c.listen(coachingProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

/// Let the async restore settle so the persisted flags resolve.
Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  group('one-time hints', () {
    test('nothing is shown until the stored flags are known', () {
      final c = _container(FakePreferencesService());
      // Pre-restore: not loaded → no hint flashes at a returning user.
      expect(c.read(coachingProvider).loaded, isFalse);
      expect(
        c.read(coachingProvider).shouldShow(CoachHint.ratingDeck),
        isFalse,
      );
    });

    test('a never-seen hint is shown once the flags load', () async {
      final c = _container(FakePreferencesService());
      await _flush();
      expect(c.read(coachingProvider).shouldShow(CoachHint.ratingDeck), isTrue);
      expect(c.read(coachingProvider).shouldShow(CoachHint.rewards), isTrue);
    });

    test('a dismissed hint is never shown again (persisted)', () async {
      final prefs = FakePreferencesService();
      final c = _container(prefs);
      await _flush();
      await c.read(coachingProvider.notifier).markSeen(CoachHint.ratingDeck);

      expect(
        c.read(coachingProvider).shouldShow(CoachHint.ratingDeck),
        isFalse,
      );
      expect(prefs.store[CoachHint.ratingDeck.prefsKey], 'true');
      // Other hints are unaffected — "seen" is per hint.
      expect(c.read(coachingProvider).shouldShow(CoachHint.rewards), isTrue);
    });

    test('a device that dismissed the legacy rating hint is not re-nagged', () {
      // The rating hint keeps its pre-existing preferences key.
      expect(CoachHint.ratingDeck.prefsKey, 'rating_coach_seen');
    });

    test('a returning user (flags stored) sees nothing', () async {
      final c = _container(
        FakePreferencesService({
          for (final hint in CoachHint.values) hint.prefsKey: 'true',
        }),
      );
      await _flush();
      for (final hint in CoachHint.values) {
        expect(c.read(coachingProvider).shouldShow(hint), isFalse);
      }
    });
  });

  group('guided player sequence', () {
    test(
      'walks piano sound → MIDI device → hands → measure rewind, then ends',
      () async {
        final prefs = FakePreferencesService();
        final c = _container(prefs);
        await _flush();
        final coaching = c.read(coachingProvider.notifier);

        coaching.startPlayerTour();
        expect(c.read(coachingProvider).step, PlayerCoachStep.pianoSound);
        coaching.nextStep();
        expect(c.read(coachingProvider).step, PlayerCoachStep.midiDevice);
        coaching.nextStep();
        expect(c.read(coachingProvider).step, PlayerCoachStep.hands);
        coaching.nextStep();
        expect(c.read(coachingProvider).step, PlayerCoachStep.measureRewind);

        coaching.nextStep();
        expect(c.read(coachingProvider).step, isNull);
        expect(c.read(coachingProvider).tourRunning, isFalse);
        expect(prefs.store[CoachHint.playerTour.prefsKey], 'true');
      },
    );

    test(
      'closing the setup surface ends the tour only on its own steps',
      () async {
        final c = _container(FakePreferencesService());
        await _flush();
        final coaching = c.read(coachingProvider.notifier);

        // On an in-modal step, closing the setup ends the sequence (it would
        // otherwise point at controls that are gone).
        coaching.startPlayerTour();
        coaching.nextStep(); // midiDevice — lives in the modal
        coaching.setupSurfaceClosed();
        expect(c.read(coachingProvider).step, isNull);

        // On the rewind step, the control lives BEHIND the modal: closing the
        // setup is what reveals it, so the sequence keeps running.
        coaching.armPlayerTourReplay();
        coaching.startPlayerTour();
        coaching.nextStep();
        coaching.nextStep();
        coaching.nextStep(); // measureRewind
        coaching.setupSurfaceClosed();
        expect(c.read(coachingProvider).step, PlayerCoachStep.measureRewind);
      },
    );

    test('skipping ends it immediately and it does not run again', () async {
      final c = _container(FakePreferencesService());
      await _flush();
      final coaching = c.read(coachingProvider.notifier);

      coaching.startPlayerTour();
      coaching.skipTour();
      expect(c.read(coachingProvider).step, isNull);

      // A later player visit does not re-walk the user.
      coaching.startPlayerTour();
      expect(c.read(coachingProvider).step, isNull);
    });

    test('does not start while the flags are still loading', () {
      final c = _container(FakePreferencesService());
      c.read(coachingProvider.notifier).startPlayerTour();
      expect(c.read(coachingProvider).step, isNull);
    });

    test('a replay armed from help runs once, then stops again', () async {
      final c = _container(
        FakePreferencesService({CoachHint.playerTour.prefsKey: 'true'}),
      );
      await _flush();
      final coaching = c.read(coachingProvider.notifier);

      // Already seen → no tour.
      coaching.startPlayerTour();
      expect(c.read(coachingProvider).step, isNull);

      coaching.armPlayerTourReplay();
      coaching.startPlayerTour();
      expect(c.read(coachingProvider).step, PlayerCoachStep.pianoSound);

      coaching.skipTour();
      expect(c.read(coachingProvider).replayArmed, isFalse);
      coaching.startPlayerTour();
      expect(c.read(coachingProvider).step, isNull);
    });
  });

  test('unusable storage suppresses every hint rather than nagging', () async {
    final c = _container(_BrokenPreferences());
    await _flush();
    expect(c.read(coachingProvider).loaded, isTrue);
    for (final hint in CoachHint.values) {
      expect(c.read(coachingProvider).shouldShow(hint), isFalse);
    }
  });
}

/// Preferences that always fail — models a device where storage is unavailable.
class _BrokenPreferences extends FakePreferencesService {
  @override
  Future<String?> getString(String key) async =>
      throw StateError('unavailable');

  @override
  Future<void> setString(String key, String value) async =>
      throw StateError('unavailable');
}
