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
import 'package:music/screens/rating_deck_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/state/rating_activity_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/notice_callout.dart';
import 'package:music/widgets/rating_invite_banner.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/rating_fakes.dart';

Future<void> _pump(
  WidgetTester tester, {
  required FakePreferencesService prefs,
  bool signedIn = true,
}) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        preferencesServiceProvider.overrideWithValue(prefs),
        canUseOnlineServicesProvider.overrideWithValue(signedIn),
        nowFnProvider.overrideWithValue(() => DateTime(2026, 7, 28)),
        // For the pushed deck when the link is tapped.
        catalogServiceProvider.overrideWithValue(
          FakeDeckCatalogService(deckCorpus(2)),
        ),
        ratingServiceProvider.overrideWithValue(FakeRatingService()),
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
      child: localizedApp(const Scaffold(body: RatingInviteBanner())),
    ),
  );
  // Resolve the async invite (persisted state + a one-row deck probe).
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 16));
  }
}

void main() {
  testWidgets('shows the callout when the user has never rated', (
    tester,
  ) async {
    await _pump(tester, prefs: FakePreferencesService());
    expect(find.byType(NoticeCallout), findsOneWidget);
  });

  testWidgets('renders nothing when signed out', (tester) async {
    await _pump(tester, prefs: FakePreferencesService(), signedIn: false);
    expect(find.byType(NoticeCallout), findsNothing);
  });

  testWidgets('renders nothing when recently rated', (tester) async {
    final prefs = FakePreferencesService({
      RatingActivity.prefsKey: DateTime(
        2026,
        7,
        27,
      ).millisecondsSinceEpoch.toString(),
    });
    await _pump(tester, prefs: prefs);
    expect(find.byType(NoticeCallout), findsNothing);
  });

  testWidgets('closing snoozes the invite (hides it + persists)', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    await _pump(tester, prefs: prefs);
    expect(find.byType(NoticeCallout), findsOneWidget);
    await tester.tap(find.byIcon(Icons.close));
    for (var i = 0; i < 4; i++) {
      await tester.pump(const Duration(milliseconds: 16)); // async recompute
    }
    expect(find.byType(NoticeCallout), findsNothing); // snoozed
    expect(prefs.store.containsKey(RatingActivity.prefsKey), isTrue);
  });

  testWidgets('the action link opens the rating deck', (tester) async {
    await _pump(tester, prefs: FakePreferencesService());
    await tester.tap(find.byIcon(Icons.arrow_forward)); // the action link
    await tester.pump(); // start the route transition
    await tester.pump(const Duration(milliseconds: 350));
    expect(find.byType(RatingDeckScreen), findsOneWidget);
  });
}
