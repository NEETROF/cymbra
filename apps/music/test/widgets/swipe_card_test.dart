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
import 'package:music/widgets/swipe_card.dart';

void main() {
  late List<String> fired;

  Future<void> pump(WidgetTester tester) async {
    fired = [];
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 320,
              height: 460,
              child: SwipeCard(
                onDislike: () => fired.add('dislike'),
                onLike: () => fired.add('like'),
                onLove: () => fired.add('love'),
                child: Container(
                  color: Colors.blue,
                  alignment: Alignment.center,
                  child: const Text('card'),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  /// Drag the card past the commit distance in [direction] (two moves so the pan
  /// recognizer engages), then let the exit animation + callback run.
  Future<void> swipe(WidgetTester tester, Offset direction) async {
    final center = tester.getCenter(find.byType(SwipeCard));
    final g = await tester.startGesture(center);
    await g.moveBy(direction / 10);
    await tester.pump();
    await g.moveBy(direction);
    await tester.pump();
    await g.up();
    await tester.pumpAndSettle();
  }

  testWidgets('swipe left fires dislike', (tester) async {
    await pump(tester);
    await swipe(tester, const Offset(-400, 0));
    expect(fired, ['dislike']);
  });

  testWidgets('swipe right fires like', (tester) async {
    await pump(tester);
    await swipe(tester, const Offset(400, 0));
    expect(fired, ['like']);
  });

  testWidgets('swipe up fires love', (tester) async {
    await pump(tester);
    await swipe(tester, const Offset(0, -400));
    expect(fired, ['love']);
  });

  testWidgets('a small drag snaps back without firing', (tester) async {
    await pump(tester);
    final center = tester.getCenter(find.byType(SwipeCard));
    final g = await tester.startGesture(center);
    await g.moveBy(const Offset(-30, 0)); // below the commit distance
    await tester.pump();
    await g.up();
    await tester.pumpAndSettle();
    expect(fired, isEmpty);
  });

  testWidgets('a fast horizontal fling commits even on a short drag', (
    tester,
  ) async {
    await pump(tester);
    // A quick flick: little distance but high velocity → commits.
    await tester.fling(find.byType(SwipeCard), const Offset(-60, 0), 1500);
    await tester.pumpAndSettle();
    expect(fired, ['dislike']);
  });
}
