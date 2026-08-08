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
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/painters/staff_hit_index.dart';
import 'package:music/widgets/notation_help_area.dart';

/// A stub painter that records one symbol covering the whole area, so a
/// long-press anywhere resolves to it — decoupling this gesture/overlay test
/// from exact glyph geometry (covered by the painter geometry tests).
class _StubPainter extends CustomPainter {
  _StubPainter(this.hitIndex, this.descriptor);

  final StaffHitIndex hitIndex;
  final SymbolDescriptor descriptor;

  @override
  void paint(Canvas canvas, Size size) {
    hitIndex
      ..clear()
      ..add(Offset.zero & size, descriptor);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}

Future<void> _pump(
  WidgetTester tester, {
  required SymbolDescriptor descriptor,
  required VoidCallback onTap,
  bool enabled = true,
}) {
  return tester.pumpWidget(
    ProviderScope(
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        locale: const Locale('en'),
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 300,
              height: 300,
              child: NotationHelpArea(
                enabled: enabled,
                builder: (context, hitIndex) => GestureDetector(
                  onTap: onTap,
                  child: CustomPaint(
                    painter: _StubPainter(hitIndex, descriptor),
                    size: const Size(300, 300),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

void main() {
  testWidgets('long-pressing a symbol opens its help bubble', (tester) async {
    await _pump(
      tester,
      descriptor: const SymbolDescriptor.clef(sign: 'G'),
      onTap: () {},
    );
    await tester.longPress(find.byType(CustomPaint).first);
    await tester.pumpAndSettle();

    // The treble-clef help title (English) is shown.
    expect(find.text('Treble clef'), findsOneWidget);
  });

  testWidgets('the close button dismisses the bubble', (tester) async {
    await _pump(
      tester,
      descriptor: const SymbolDescriptor.clef(sign: 'G'),
      onTap: () {},
    );
    await tester.longPress(find.byType(CustomPaint).first);
    await tester.pumpAndSettle();
    expect(find.text('Treble clef'), findsOneWidget);

    await tester.tap(find.byKey(const Key('notation-help-close')));
    await tester.pumpAndSettle();
    expect(find.text('Treble clef'), findsNothing);
  });

  testWidgets('a plain tap does not open a bubble and reaches the child', (
    tester,
  ) async {
    var tapped = false;
    await _pump(
      tester,
      descriptor: const SymbolDescriptor.clef(sign: 'G'),
      onTap: () => tapped = true,
    );

    await tester.tap(find.byType(CustomPaint).first);
    await tester.pumpAndSettle();

    // The tap reached the underlying child (play/scrub gestures keep working)…
    expect(tapped, isTrue);
    // …and no help bubble was opened by a mere tap.
    expect(find.text('Treble clef'), findsNothing);
  });

  testWidgets('an accidental long-press shows the matching help', (
    tester,
  ) async {
    await _pump(
      tester,
      descriptor: const SymbolDescriptor.accidental(token: 'flat'),
      onTap: () {},
    );
    await tester.longPress(find.byType(CustomPaint).first);
    await tester.pumpAndSettle();
    expect(find.text('Flat (♭)'), findsOneWidget);
  });

  testWidgets('disabled area shows no bubble on long-press', (tester) async {
    await _pump(
      tester,
      descriptor: const SymbolDescriptor.clef(sign: 'G'),
      onTap: () {},
      enabled: false,
    );
    await tester.longPress(find.byType(CustomPaint).first);
    await tester.pumpAndSettle();
    expect(find.text('Treble clef'), findsNothing);
  });
}
