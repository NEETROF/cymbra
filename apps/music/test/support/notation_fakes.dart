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

import 'dart:typed_data';

import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart';

/// [ScoreAssetSource] returning fixed in-memory bytes (no asset bundle).
class FakeScoreAssetSource implements ScoreAssetSource {
  Uint8List bytes;
  final List<String> loaded = <String>[];

  FakeScoreAssetSource([Uint8List? bytes])
    : bytes = bytes ?? Uint8List.fromList(const [1, 2, 3]);

  @override
  Future<Uint8List> load(String assetPath) async {
    loaded.add(assetPath);
    return bytes;
  }
}

/// [NotationEngine] that returns a scripted document (or throws), so notation
/// state can be exercised without the native library.
class FakeNotationEngine implements NotationEngine {
  ScoreDocument? document;
  Object? parseError;
  int layoutCalls = 0;
  double? lastWidth;

  FakeNotationEngine({this.document, this.parseError});

  @override
  Future<ScoreDocument> parse(Uint8List bytes) async {
    if (parseError != null) throw parseError!;
    return document ?? sampleGrandStaffDocument();
  }

  @override
  List<System> layout(ScoreDocument document, double availableWidth) {
    layoutCalls++;
    lastWidth = availableWidth;
    return [
      for (var i = 0; i < document.measures.length; i++)
        System(measures: Uint32List.fromList([i]), staves: document.staves),
    ];
  }
}

/// A fully-populated [NoteEvent] with sensible defaults for tests.
NoteEvent noteEvent({
  int staff = 1,
  int voice = 1,
  int positionDivisions = 0,
  Pitch? pitch,
  bool isRest = false,
  bool isChord = false,
  int durationDivisions = 4,
  String? noteType = 'quarter',
  int dots = 0,
  String? accidental,
  Lyric? lyric,
  StemDir? stem,
}) => NoteEvent(
  staff: staff,
  voice: voice,
  positionDivisions: positionDivisions,
  pitch: pitch,
  isRest: isRest,
  isChord: isChord,
  durationDivisions: durationDivisions,
  noteType: noteType,
  dots: dots,
  accidental: accidental,
  tieStart: false,
  tieStop: false,
  tuplet: null,
  stem: stem,
  beams: const [],
  lyric: lyric,
);

/// A treble-only document with a beamed run of eighth notes that rises then
/// falls (the arpeggio contour that previously produced a "tent" beam), plus a
/// pair of beamed sixteenths — to eyeball beaming/flags.
ScoreDocument sampleBeamedDocument() {
  NoteEvent eighth(
    String step,
    int octave,
    int pos, {
    List<BeamState>? beams,
  }) => NoteEvent(
    staff: 1,
    voice: 1,
    positionDivisions: pos,
    pitch: Pitch(step: step, octave: octave, alter: 0),
    isRest: false,
    isChord: false,
    durationDivisions: 2,
    noteType: 'eighth',
    dots: 0,
    accidental: null,
    tieStart: false,
    tieStop: false,
    tuplet: null,
    stem: StemDir.up,
    beams: beams ?? const [],
    lyric: null,
  );
  return ScoreDocument(
    meta: const ScoreMeta(title: 'Beamed', composer: 'Tester'),
    staves: 1,
    attributes: const Attributes(
      divisions: 4,
      clefs: [Clef(staff: 1, sign: 'G', line: 2)],
      keyFifths: 0,
      time: TimeSignature(beats: 4, beatType: 4),
    ),
    measures: [
      NotationMeasure(
        index: 0,
        minWidth: 200,
        directions: const [],
        notes: [
          eighth('E', 4, 0, beams: const [BeamState.begin]),
          eighth('G', 4, 2, beams: const [BeamState.continue_]),
          eighth('C', 5, 4, beams: const [BeamState.continue_]),
          eighth('G', 4, 6, beams: const [BeamState.continue_]),
          eighth('E', 4, 8, beams: const [BeamState.continue_]),
          eighth('C', 4, 10, beams: const [BeamState.end]),
        ],
      ),
    ],
  );
}

/// A small two-staff (grand-staff) document: one 4/4 measure with a treble note,
/// a bass note, a `words` direction, and a `dynamics` direction.
ScoreDocument sampleGrandStaffDocument() => ScoreDocument(
  meta: const ScoreMeta(title: 'Sample', composer: 'Tester'),
  staves: 2,
  attributes: const Attributes(
    divisions: 4,
    clefs: [
      Clef(staff: 1, sign: 'G', line: 2),
      Clef(staff: 2, sign: 'F', line: 4),
    ],
    keyFifths: 3, // 3 sharps → shows the key signature (armature)
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      index: 0,
      minWidth: 120,
      directions: const [
        Direction(
          staff: 1,
          positionDivisions: 0,
          kind: DirectionKind.words('Andante'),
        ),
        Direction(
          staff: 1,
          positionDivisions: 0,
          kind: DirectionKind.dynamics('mf'),
        ),
      ],
      notes: [
        noteEvent(
          staff: 1,
          pitch: const Pitch(step: 'C', octave: 5, alter: 0),
          lyric: const Lyric(syllabic: 'single', text: 'la'),
          stem: StemDir.up,
        ),
        noteEvent(
          staff: 2,
          pitch: const Pitch(step: 'C', octave: 3, alter: 0),
          durationDivisions: 16,
          noteType: 'whole',
        ),
      ],
    ),
  ],
);
