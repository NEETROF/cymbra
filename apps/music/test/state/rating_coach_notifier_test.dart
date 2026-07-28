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
import 'package:music/state/rating_coach_notifier.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container(
  FakePreferencesService prefs, {
  bool build = true,
}) {
  final c = ProviderContainer(
    overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
  );
  // Hold a subscription so the notifier builds now (kicking off `_restore`),
  // unless the test wants to observe the pre-build `null` state itself.
  if (build) {
    final sub = c.listen(ratingCoachMarkProvider, (_, _) {});
    addTearDown(sub.close);
  }
  addTearDown(c.dispose);
  return c;
}

/// Let the async `_restore` settle so the tri-state flag resolves.
Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  test('first open shows the coach mark (never seen)', () async {
    // Observe the pre-build `null` (nothing flashes), then the resolved state.
    final c = _container(FakePreferencesService(), build: false);
    expect(c.read(ratingCoachMarkProvider), isNull);
    await _flush();
    // Absent flag → not seen → show once.
    expect(c.read(ratingCoachMarkProvider), isFalse);
  });

  test('a returning user (flag stored) never sees it again', () async {
    final c = _container(
      FakePreferencesService({RatingCoachMark.prefsKey: 'true'}),
    );
    await _flush();
    expect(c.read(ratingCoachMarkProvider), isTrue);
  });

  test('markSeen persists and suppresses the hint', () async {
    final prefs = FakePreferencesService();
    final c = _container(prefs);
    await _flush();
    expect(c.read(ratingCoachMarkProvider), isFalse);
    await c.read(ratingCoachMarkProvider.notifier).markSeen();
    // In-memory flag flips and the choice is persisted for next launch.
    expect(c.read(ratingCoachMarkProvider), isTrue);
    expect(prefs.store[RatingCoachMark.prefsKey], 'true');
  });
}
