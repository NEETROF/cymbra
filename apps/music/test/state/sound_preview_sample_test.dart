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
import 'package:music/services/notation_engine.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/sound_preview_sample.dart';

import '../support/notation_fakes.dart';

void main() {
  // rootBundle asset loading needs the flutter test binding.
  TestWidgetsFlutterBinding.ensureInitialized();

  test('loads + parses the bundled sample into a playback-ready score', () async {
    // The notation engine is native (FFI-absent in unit tests): fake the parse to
    // return a small document so the provider's load → parse → derive path runs.
    final doc = ScoreDocument(
      playOrder: const [],
      meta: const ScoreMeta(title: 'Ode to Joy', composer: ''),
      staves: 1,
      attributes: Attributes(
        divisions: 4,
        clefs: const [],
        keyFifths: 0,
        time: TimeSignature(beats: 3, beatType: 4),
      ),
      measures: const [],
    );
    final container = ProviderContainer(
      overrides: [
        notationEngineProvider.overrideWithValue(
          FakeNotationEngine(document: doc),
        ),
      ],
    );
    addTearDown(container.dispose);

    final score = await container.read(soundPreviewSampleProvider.future);

    // The provider mapped the parsed document's timing attributes.
    expect(score.beats, 3);
    expect(score.beatType, 4);
  });
}
