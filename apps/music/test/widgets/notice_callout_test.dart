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
import 'package:music/widgets/notice_callout.dart';

import '../support/localized.dart';

void main() {
  testWidgets('renders the title, message and action link', (tester) async {
    await tester.pumpWidget(
      localizedApp(
        Scaffold(
          body: NoticeCallout(
            title: 'A title',
            message: 'A message.',
            actionLabel: 'Do it',
            onAction: () {},
          ),
        ),
      ),
    );
    expect(find.text('A title'), findsOneWidget);
    expect(find.text('A message.'), findsOneWidget);
    expect(find.text('Do it'), findsOneWidget);
    expect(find.byIcon(Icons.arrow_forward), findsOneWidget);
  });

  testWidgets('tapping the link fires onAction', (tester) async {
    var acted = false;
    await tester.pumpWidget(
      localizedApp(
        Scaffold(
          body: NoticeCallout(
            title: 'T',
            message: 'M',
            actionLabel: 'Go',
            onAction: () => acted = true,
          ),
        ),
      ),
    );
    await tester.tap(find.text('Go'));
    await tester.pump();
    expect(acted, isTrue);
  });

  testWidgets('no close button unless onClose is given', (tester) async {
    await tester.pumpWidget(
      localizedApp(
        Scaffold(
          body: NoticeCallout(
            title: 'T',
            message: 'M',
            actionLabel: 'Go',
            onAction: () {},
          ),
        ),
      ),
    );
    expect(find.byIcon(Icons.close), findsNothing);
  });

  testWidgets('the close button fires onClose', (tester) async {
    var closed = false;
    await tester.pumpWidget(
      localizedApp(
        Scaffold(
          body: NoticeCallout(
            title: 'T',
            message: 'M',
            actionLabel: 'Go',
            onAction: () {},
            onClose: () => closed = true,
          ),
        ),
      ),
    );
    expect(find.byIcon(Icons.close), findsOneWidget);
    await tester.tap(find.byIcon(Icons.close));
    await tester.pump();
    expect(closed, isTrue);
  });
}
