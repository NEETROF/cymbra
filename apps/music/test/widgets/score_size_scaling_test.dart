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
import 'package:music/painters/staff_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/player_preferences.dart';
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

StaffPainter _staffPainter(WidgetTester tester) =>
    tester
            .widgetList<CustomPaint>(find.byType(CustomPaint))
            .firstWhere((w) => w.painter is StaffPainter)
            .painter
        as StaffPainter;

void main() {
  testWidgets('the score size scales the Portée notation and its window', (
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
    container.read(playerProvider.notifier).setMode(RenderMode.staff);
    await tester.pump();

    // Default (medium): historical scale and look-ahead.
    var painter = _staffPainter(tester);
    expect(painter.noteScale, 1.0);
    expect(painter.lookAheadMs, StaffPainter.defaultLookAheadMs);

    // Large: notes grow and the visible window narrows by the same factor.
    container
        .read(playerPreferencesProvider.notifier)
        .setScoreSize(ScoreSize.large);
    await tester.pump();
    painter = _staffPainter(tester);
    expect(painter.noteScale, ScoreSize.large.factor);
    expect(
      painter.lookAheadMs,
      StaffPainter.defaultLookAheadMs / ScoreSize.large.factor,
    );

    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    container.dispose();
  });
}
