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
import 'package:music/widgets/score_propose_sheet.dart';

import '../support/localized.dart';

void main() {
  // Pumps a button, opens the dialog, and returns a getter for its captured result.
  Future<Future<ScoreProposeResult?> Function()> openDialog(
    WidgetTester tester, {
    bool rejected = false,
    String? reason,
  }) async {
    ScoreProposeResult? result;
    await tester.pumpWidget(
      localizedApp(
        Builder(
          builder: (context) => Center(
            child: ElevatedButton(
              onPressed: () async => result = await showScoreProposeDialog(
                context,
                rejected: rejected,
                rejectionReason: reason,
              ),
              child: const Text('open'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    return () async => result;
  }

  Finder submitBtn() => find.widgetWithText(FilledButton, 'Propose');
  bool enabled(WidgetTester t) =>
      t.widget<FilledButton>(submitBtn()).onPressed != null;

  group('ScoreProposalTag', () {
    testWidgets('renders the short status label', (tester) async {
      await tester.pumpWidget(
        localizedApp(const Scaffold(body: ScoreProposalTag(status: 'pending'))),
      );
      expect(find.text('Pending'), findsOneWidget);

      await tester.pumpWidget(
        localizedApp(
          const Scaffold(body: ScoreProposalTag(status: 'accepted')),
        ),
      );
      expect(find.text('Accepted'), findsOneWidget);
    });
  });

  group('showScoreProposeDialog', () {
    testWidgets('submit is gated on the attestation, returns the licence', (
      tester,
    ) async {
      final result = await openDialog(tester);

      // Disabled until the right-to-distribute attestation is ticked.
      expect(enabled(tester), isFalse);
      await tester.tap(find.byType(Checkbox));
      await tester.pumpAndSettle();
      expect(enabled(tester), isTrue);

      await tester.tap(submitBtn());
      await tester.pumpAndSettle();
      // The licence dropdown defaults to the first predefined choice; no
      // justification on a first proposal.
      final r = await result();
      expect(r?.license, 'CC0-1.0');
      expect(r?.justification, isNull);
    });

    testWidgets(
      'rejected re-proposal shows the reason and needs a justification',
      (tester) async {
        final result = await openDialog(
          tester,
          rejected: true,
          reason: 'blurry scan',
        );

        // The moderator's reason is shown.
        expect(find.textContaining('blurry scan'), findsOneWidget);

        // The justification is the last text field (after attribution); enterText
        // works even when the field is scrolled off the test viewport.
        await tester.enterText(find.byType(TextField).last, 'fixed key sig');
        await tester.pumpAndSettle();

        final checkbox = find.byType(Checkbox);
        await tester.ensureVisible(checkbox);
        // Still disabled before the attestation is ticked.
        expect(enabled(tester), isFalse);
        await tester.tap(checkbox);
        await tester.pumpAndSettle();
        expect(enabled(tester), isTrue);

        await tester.ensureVisible(submitBtn());
        await tester.tap(submitBtn());
        await tester.pumpAndSettle();
        final r = await result();
        expect(r?.justification, 'fixed key sig');
      },
    );

    testWidgets('cancel returns null', (tester) async {
      final result = await openDialog(tester);
      await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
      await tester.pumpAndSettle();
      expect(find.byType(AlertDialog), findsNothing);
      expect(await result(), isNull);
    });
  });
}
