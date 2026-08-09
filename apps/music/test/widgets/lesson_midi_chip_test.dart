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
import 'package:music/services/midi_service.dart';
import 'package:music/widgets/lesson_midi_chip.dart';

import '../support/fakes.dart';

Future<void> pumpChip(WidgetTester tester, FakeMidiService midi) async {
  addTearDown(midi.close);
  final container = ProviderContainer(
    overrides: [midiServiceProvider.overrideWithValue(midi)],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        locale: const Locale('fr'),
        home: const Scaffold(
          body: Align(alignment: Alignment.topRight, child: LessonMidiChip()),
        ),
      ),
    ),
  );
  await tester.pump();
}

void main() {
  testWidgets('reads "no device" like the game pill and lists nothing', (
    tester,
  ) async {
    await pumpChip(tester, FakeMidiService());
    expect(find.byIcon(Icons.usb_off), findsOneWidget);
    expect(find.text('Aucun appareil MIDI'), findsOneWidget);

    await tester.tap(find.byKey(const Key('lesson-midi-chip')));
    await tester.pumpAndSettle();
    expect(find.text('Aucun appareil détecté'), findsOneWidget);
  });

  testWidgets('picking a port connects it and the pill turns green', (
    tester,
  ) async {
    final midi = FakeMidiService(ports: ['Yamaha P-145', 'IAC Bus 1']);
    await pumpChip(tester, midi);
    // Ports exist but nothing is connected yet — the "connecting" state.
    expect(find.byIcon(Icons.usb), findsOneWidget);
    expect(find.text('Connexion…'), findsOneWidget);

    await tester.tap(find.byKey(const Key('lesson-midi-chip')));
    await tester.pumpAndSettle();
    expect(find.text('Yamaha P-145'), findsOneWidget);
    expect(find.text('IAC Bus 1'), findsOneWidget);

    await tester.tap(find.byKey(const Key('lesson-midi-port-Yamaha P-145')));
    await tester.pumpAndSettle();
    expect(midi.selectPortCalls, ['Yamaha P-145']);
    // Connected: the pill carries the chosen port's name.
    expect(find.text('Yamaha P-145'), findsOneWidget);
    expect(find.byIcon(Icons.usb), findsOneWidget);
    expect(find.byIcon(Icons.usb_off), findsNothing);
  });

  testWidgets('an incoming MIDI event refreshes a hot-plugged connection', (
    tester,
  ) async {
    final midi = FakeMidiService();
    await pumpChip(tester, midi);
    expect(find.byIcon(Icons.usb_off), findsOneWidget);

    // The instrument appears and plays a note — the pill catches up without
    // any polling.
    midi.ports = ['P-145'];
    midi.connected = 'P-145';
    midi.emit(noteOnEvent(60));
    await tester.pump();
    await tester.pump();
    expect(find.byIcon(Icons.usb), findsOneWidget);
    expect(find.text('P-145'), findsOneWidget);
  });
}
