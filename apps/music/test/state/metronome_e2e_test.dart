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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

/// Minuet in G, reconstructed from the real Rust parse: 3/4, divisions = 4,
/// three measures (two staves), each spanning 12 divisions.
ScoreDocument _threeFour() {
  NoteEvent n(int pos, int dur, int staff) => noteEvent(
    positionDivisions: pos,
    durationDivisions: dur,
    staff: staff,
    pitch: Pitch(step: 'C', octave: staff == 1 ? 5 : 3, alter: 0),
  );
  NotationMeasure measure(int index, List<NoteEvent> notes) => NotationMeasure(
    index: index,
    clefs: const [],
    minWidth: 120,
    directions: const [],
    notes: notes,
  );
  return ScoreDocument(
    meta: const ScoreMeta(title: 'Minuet', composer: 'Bach'),
    staves: 2,
    attributes: const Attributes(
      divisions: 4,
      clefs: [
        Clef(staff: 1, sign: 'G', line: 2),
        Clef(staff: 2, sign: 'F', line: 4),
      ],
      keyFifths: 1,
      time: TimeSignature(beats: 3, beatType: 4),
    ),
    measures: [
      measure(0, [
        n(0, 4, 1),
        n(4, 2, 1),
        n(6, 2, 1),
        n(8, 2, 1),
        n(10, 2, 1),
        n(0, 4, 2),
        n(4, 4, 2),
        n(8, 4, 2),
      ]),
      measure(1, [
        n(0, 4, 1),
        n(4, 4, 1),
        n(8, 4, 1),
        n(0, 4, 2),
        n(4, 4, 2),
        n(8, 4, 2),
      ]),
      measure(2, [
        n(0, 8, 1),
        n(8, 2, 1),
        n(10, 1, 1),
        n(11, 1, 1),
        n(0, 4, 2),
        n(4, 4, 2),
        n(8, 4, 2),
      ]),
    ],
  );
}

void main() {
  test('3/4 score ticks three times per measure end-to-end', () async {
    final audio = RecordingAudioService();
    final container = ProviderContainer(
      overrides: [
        scoreCatalogProvider.overrideWithValue(const [_entry]),
        scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
        notationEngineProvider.overrideWithValue(
          FakeNotationEngine(document: _threeFour()),
        ),
        midiServiceProvider.overrideWithValue(FakeMidiService()),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(audio),
      ],
    );
    addTearDown(container.dispose);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    container.read(selectedScoreProvider.notifier).select(_entry);
    // Let the async parse + notation listener apply the document.
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);

    final notifier = container.read(playerProvider.notifier);
    final s = container.read(playerProvider);
    // The 3/4 time signature is propagated to playback state.
    expect(s.beats, 3);
    expect(s.measureStartMs, [0, 2000, 4000]);
    expect(s.songEndMs, 6000);

    // WAIT MODE ON (the app default): the playhead freezes at each onset until
    // its notes are played. Drive it like a player would and count the beats.
    expect(s.waitMode, isTrue);
    notifier.togglePlay();
    notifier.toggleMetronome();

    var guard = 0;
    while (container.read(playerProvider).elapsedMs < s.songEndMs - 30 &&
        guard++ < 100000) {
      final st = container.read(playerProvider);
      // If frozen on an onset, satisfy it by pressing the awaited notes.
      for (final p in st.onsetPitchesAt(st.elapsedMs)) {
        notifier.noteOn(p);
        notifier.noteOff(p);
      }
      notifier.advance(16);
    }

    // Three 3/4 measures → exactly 9 ticks (3 per measure), one accent per
    // measure on the downbeat — not 4 per measure.
    expect(
      audio.metronomeClicks.length,
      9,
      reason: 'three 3/4 measures should tick 9 times (3 each), not 12',
    );
    expect(
      audio.metronomeClicks.where((a) => a).length,
      3,
    ); // one accent/measure
  });
}
