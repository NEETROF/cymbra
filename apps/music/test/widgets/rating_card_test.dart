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
import 'package:flutter_test/flutter_test.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/widgets/rating_card.dart';

// The pending "potential new score" badge is a sibling of the preview in the
// card's Stack, so it renders from `entry.isPending` alone. These tests use an
// entry WITHOUT a catalogId (so the in-card audio preview is not mounted) to
// exercise the badge branch in isolation (change: rate-pending-scores).
CatalogEntry _entry({required String? moderationStatus}) => CatalogEntry(
  id: 'catalog-x',
  title: 'A Candidate',
  composer: 'Anon',
  level: PracticeLevel.beginner,
  moderationStatus: moderationStatus,
);

Future<void> _pump(
  WidgetTester tester,
  CatalogEntry entry,
) => tester.pumpWidget(
  MaterialApp(
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    home: Scaffold(
      // Bound the card in both dimensions so the ScoreCard's cover-ratio + flex
      // layout resolves in the test surface (unbounded width overflows its title
      // column; unbounded height breaks its inner Expanded).
      body: Center(
        child: SizedBox(
          width: 320,
          height: 460,
          child: RatingCard(entry: entry),
        ),
      ),
    ),
  ),
);

void main() {
  testWidgets('a pending card shows the "potential new score" badge', (
    tester,
  ) async {
    await _pump(tester, _entry(moderationStatus: 'pending'));
    expect(find.text('Potential new score'), findsOneWidget);
  });

  testWidgets('an accepted card shows no pending badge', (tester) async {
    await _pump(tester, _entry(moderationStatus: 'accepted'));
    expect(find.text('Potential new score'), findsNothing);
  });
}
