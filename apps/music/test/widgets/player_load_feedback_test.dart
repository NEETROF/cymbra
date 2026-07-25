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
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';

/// A [Notation] whose state is fixed, so the player renders a chosen load state
/// (loading / error) without touching the byte sources.
class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
}

/// A [SelectedScore] pinned to a given entry (or null), so the screen can tell
/// "loading" (a score is selected) from "no selection".
class _FixedSelected extends SelectedScore {
  _FixedSelected(this._entry);
  final CatalogEntry? _entry;
  @override
  CatalogEntry? build() => _entry;
}

const _entry = CatalogEntry(
  id: 'sel',
  title: 'Selected',
  composer: 'X',
  level: PracticeLevel.beginner,
  catalogId: 'sel',
);

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  required NotationData notation,
  CatalogEntry? selected,
}) async {
  await tester.binding.setSurfaceSize(const Size(1600, 900));
  final container = ProviderContainer(
    overrides: [
      midiServiceProvider.overrideWithValue(
        FakeMidiService(ports: const [], connected: null),
      ),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      notationProvider.overrideWith(() => _FixedNotation(notation)),
      selectedScoreProvider.overrideWith(() => _FixedSelected(selected)),
    ],
  );
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const PlayerScreen(), locale: const Locale('en')),
    ),
  );
  await tester.pump();
  return container;
}

/// Unmounts the screen and disposes the container so the player's periodic
/// status timer is cancelled before the test ends (mirrors player_screen_test).
Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
  await tester.binding.setSurfaceSize(null);
}

void main() {
  testWidgets('Synthesia shows a spinner while a selected score is loading', (
    tester,
  ) async {
    // A selected score with neither document nor error yet = loading in flight.
    final c = await _pump(
      tester,
      notation: const NotationData(),
      selected: _entry,
    );
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    expect(find.text('Loading score…'), findsOneWidget);
    await _teardown(tester, c);
  });

  testWidgets('Synthesia shows an error banner when the load fails', (
    tester,
  ) async {
    final c = await _pump(
      tester,
      notation: const NotationData(error: 'boom'),
      selected: _entry,
    );
    expect(find.textContaining('boom'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);
    await _teardown(tester, c);
  });

  testWidgets('Staff mode also shows the error banner', (tester) async {
    final container = await _pump(
      tester,
      notation: const NotationData(error: 'boom'),
      selected: _entry,
    );
    container.read(playerProvider.notifier).setMode(RenderMode.staff);
    await tester.pump();
    expect(find.textContaining('boom'), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('no spinner or error when nothing is selected', (tester) async {
    final c = await _pump(
      tester,
      notation: const NotationData(),
      selected: null,
    );
    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(find.textContaining('Could not load'), findsNothing);
    await _teardown(tester, c);
  });
}
