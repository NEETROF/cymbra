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
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

final _bar = find.byKey(const Key('playback-progress'));

double _fillFraction(WidgetTester tester) {
  final fill = tester.widget<FractionallySizedBox>(
    find.descendant(of: _bar, matching: find.byType(FractionallySizedBox)),
  );
  return fill.widthFactor!;
}

void main() {
  testWidgets('the bar tracks the playhead over the piece duration', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    final container = ProviderContainer(
      overrides: [
        scoreCatalogProvider.overrideWithValue(const [_entry]),
        scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
        midiServiceProvider.overrideWithValue(FakeMidiService()),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );
    container.read(selectedScoreProvider.notifier).select(_entry);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const PlayerScreen()),
      ),
    );
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    // Timed score loaded → the bar is present, empty at the start.
    expect(_bar, findsOneWidget);
    expect(_fillFraction(tester), 0.0);

    final notifier = container.read(playerProvider.notifier);
    notifier.toggleWaitMode(); // off, so the playhead advances freely
    notifier.togglePlay();
    // 50ms steps: the ticker guard ignores dt >= 100ms.
    for (var i = 0; i < 40; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    final playing = _fillFraction(tester);
    expect(playing, greaterThan(0.0));

    // Pausing freezes the fill.
    notifier.togglePlay();
    await tester.pump();
    final paused = _fillFraction(tester);
    for (var i = 0; i < 6; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(_fillFraction(tester), paused);

    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    container.dispose();
  });
}
