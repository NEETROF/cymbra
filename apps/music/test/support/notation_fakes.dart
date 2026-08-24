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

/// [ScoreAssetSource] returning fixed in-memory bytes (no asset bundle). Set
/// [loadError] to make `load` throw instead (e.g. a backend `AuthException`), to
/// exercise the notifier's failure classification.
class FakeScoreAssetSource implements ScoreAssetSource {
  Uint8List bytes;
  Object? loadError;
  final List<String> loaded = <String>[];

  FakeScoreAssetSource([Uint8List? bytes])
    : bytes = bytes ?? Uint8List.fromList(const [1, 2, 3]);

  @override
  Future<Uint8List> load(String assetPath) async {
    loaded.add(assetPath);
    if (loadError != null) throw loadError!;
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

  /// Scripted validation outcome; defaults to a valid piano summary.
  ValidationOutcome? validateOutcome;

  FakeNotationEngine({this.document, this.parseError, this.validateOutcome});

  @override
  Future<ScoreDocument> parse(Uint8List bytes) async {
    if (parseError != null) throw parseError!;
    return document ?? sampleGrandStaffDocument();
  }

  @override
  Future<ValidationOutcome> validate(Uint8List bytes) async {
    return validateOutcome ??
        ValidationOutcome(
          summary: const ScoreSummary(
            title: 'Sample',
            composer: 'A. Composer',
            titleNorm: 'sample',
            workKey: 'a. composer::sample',
            instrument: InstrumentKind.keyboard,
            staves: 2,
            keyFifths: 0,
            timeSig: '4/4',
            measureCount: 4,
            noteCount: 8,
          ),
        );
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

/// No repeat notation — the fixture default for [NotationMeasure.repeats].
final RepeatMarks noRepeats = RepeatMarks(
  forward: false,
  backwardTimes: 0,
  endingStart: Uint32List(0),
  endingStop: false,
  endingDiscontinue: false,
  measureRepeatOf: null,
  measureRepeatSlashes: 0,
  segno: false,
  coda: false,
  soundDacapo: false,
  soundDalsegno: false,
  soundTocoda: false,
  soundFine: false,
  soundForwardRepeat: false,
);

/// [RepeatMarks] with fixture defaults — name only what the test needs.
RepeatMarks repeatMarks({
  bool forward = false,
  int backwardTimes = 0,
  List<int> endingStart = const [],
  bool endingStop = false,
  bool endingDiscontinue = false,
  int? measureRepeatOf,
  int measureRepeatSlashes = 0,
  bool segno = false,
  bool coda = false,
  bool soundDacapo = false,
  bool soundDalsegno = false,
  bool soundTocoda = false,
  bool soundFine = false,
  bool soundForwardRepeat = false,
}) => RepeatMarks(
  forward: forward,
  backwardTimes: backwardTimes,
  endingStart: Uint32List.fromList(endingStart),
  endingStop: endingStop,
  endingDiscontinue: endingDiscontinue,
  measureRepeatOf: measureRepeatOf,
  measureRepeatSlashes: measureRepeatSlashes,
  segno: segno,
  coda: coda,
  soundDacapo: soundDacapo,
  soundDalsegno: soundDalsegno,
  soundTocoda: soundTocoda,
  soundFine: soundFine,
  soundForwardRepeat: soundForwardRepeat,
);

/// A fully-populated [NoteEvent] with sensible defaults for tests.
NoteEvent noteEvent({
  int staff = 1,
  int voice = 1,
  int positionDivisions = 0,
  Pitch? pitch,
  bool isRest = false,
  bool isChord = false,
  bool isGrace = false,
  int durationDivisions = 4,
  String? noteType = 'quarter',
  int dots = 0,
  String? accidental,
  Lyric? lyric,
  StemDir? stem,
  bool tieStart = false,
  bool tieStop = false,
  bool slurStart = false,
  bool slurStop = false,
  List<BeamState> beams = const [],
  Unpitched? unpitched,
  String? instrumentId,
}) => NoteEvent(
  staff: staff,
  voice: voice,
  positionDivisions: positionDivisions,
  pitch: pitch,
  isRest: isRest,
  isChord: isChord,
  isGrace: isGrace,
  durationDivisions: durationDivisions,
  noteType: noteType,
  dots: dots,
  accidental: accidental,
  tieStart: tieStart,
  tieStop: tieStop,
  slurStart: slurStart,
  slurStop: slurStop,
  tuplet: null,
  stem: stem,
  beams: beams,
  lyric: lyric,
  unpitched: unpitched,
  instrumentId: instrumentId,
);

/// A treble document with a phrase slur over four notes and a tie between the
/// last two (same pitch) — to eyeball tie/slur arcs.
ScoreDocument sampleTieSlurDocument() => ScoreDocument(
  instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'TieSlur', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 200,
      directions: const [],
      notes: [
        noteEvent(
          positionDivisions: 0,
          pitch: const Pitch(step: 'C', octave: 5, alter: 0),
          stem: StemDir.down,
          slurStart: true,
        ),
        noteEvent(
          positionDivisions: 4,
          pitch: const Pitch(step: 'D', octave: 5, alter: 0),
          stem: StemDir.down,
        ),
        noteEvent(
          positionDivisions: 8,
          pitch: const Pitch(step: 'E', octave: 5, alter: 0),
          stem: StemDir.down,
          tieStart: true,
        ),
        noteEvent(
          positionDivisions: 12,
          pitch: const Pitch(step: 'E', octave: 5, alter: 0),
          stem: StemDir.down,
          tieStop: true,
          slurStop: true,
        ),
      ],
    ),
  ],
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
    isGrace: false,
    durationDivisions: 2,
    noteType: 'eighth',
    dots: 0,
    accidental: null,
    tieStart: false,
    tieStop: false,
    slurStart: false,
    slurStop: false,
    tuplet: const Tuplet(actual: 3, normal: 2), // two triplets → "3" markers
    stem: StemDir.up,
    beams: beams ?? const [],
    lyric: null,
  );
  return ScoreDocument(
    instruments: const [],
    playOrder: const [],
    meta: const ScoreMeta(title: 'Beamed', composer: 'Tester'),
    staves: 1,
    attributes: const Attributes(
      divisions: 4,
      clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
      keyFifths: 0,
      time: TimeSignature(beats: 4, beatType: 4),
    ),
    measures: [
      NotationMeasure(
        repeats: noRepeats,
        index: 0,
        clefs: const [],
        keyFifths: 0,
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

/// A grand-staff document whose left hand (staff 2) starts in treble clef and
/// switches to bass clef in the second measure (as in Debussy's Arabesque),
/// plus two overlapping word directions at the same beat — to eyeball the clef
/// change and the word de-overlap.
ScoreDocument sampleClefChangeDocument() => ScoreDocument(
  instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'ClefChange', composer: 'Tester'),
  staves: 2,
  attributes: const Attributes(
    divisions: 4,
    clefs: [
      Clef(staff: 1, sign: ClefSign.g, line: 2),
      Clef(staff: 2, sign: ClefSign.g, line: 2), // left hand starts in treble
    ],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 140,
      directions: const [
        Direction(
          staff: 1,
          positionDivisions: 0,
          kind: DirectionKind.words('stringendo'),
        ),
        Direction(
          staff: 1,
          positionDivisions: 0,
          kind: DirectionKind.words('cresc.'),
        ),
      ],
      notes: [
        noteEvent(staff: 1, pitch: const Pitch(step: 'G', octave: 4, alter: 0)),
        noteEvent(staff: 2, pitch: const Pitch(step: 'B', octave: 3, alter: 0)),
      ],
    ),
    NotationMeasure(
      repeats: noRepeats,
      index: 1,
      clefs: const [Clef(staff: 2, sign: ClefSign.f, line: 4)], // → bass clef
      keyFifths: 0,
      minWidth: 140,
      directions: const [],
      notes: [
        noteEvent(staff: 1, pitch: const Pitch(step: 'A', octave: 4, alter: 0)),
        noteEvent(staff: 2, pitch: const Pitch(step: 'C', octave: 3, alter: 0)),
      ],
    ),
  ],
);

/// A treble document with [count] one-note 4/4 measures, so the fake layout
/// (one system per measure) produces many systems — tall enough to scroll.
ScoreDocument tallDocument(int count) => ScoreDocument(
  instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'Tall', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    for (var i = 0; i < count; i++)
      NotationMeasure(
        repeats: noRepeats,
        index: i,
        clefs: const [],
        keyFifths: 0,
        minWidth: 120,
        directions: const [],
        notes: [
          noteEvent(
            staff: 1,
            pitch: const Pitch(step: 'C', octave: 5, alter: 0),
            durationDivisions: 16,
            noteType: 'whole',
          ),
        ],
      ),
  ],
);

/// A small two-staff (grand-staff) document: one 4/4 measure with a treble note,
/// a bass note, a `words` direction, and a `dynamics` direction.

/// A minimal percussion document (change: add-drum-kit-view): closed hi-hat
/// eighths + a snare on beat 2 in voice 1 (hands, stems up), the kick on
/// beats 1 and 3 in voice 2 (feet) — under a percussion clef, with the
/// part-list instrument table resolved (GM numbers already 0-based).
ScoreDocument sampleDrumDocument() => ScoreDocument(
  instruments: const [
    InstrumentDecl(id: 'P1-I36', name: 'Bass Drum 1', gmNumber: 36),
    InstrumentDecl(id: 'P1-I38', name: 'Snare Drum', gmNumber: 38),
    InstrumentDecl(id: 'P1-I42', name: 'Closed Hi-hat', gmNumber: 42),
  ],
  playOrder: const [],
  meta: const ScoreMeta(title: 'Groove', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 2,
    clefs: [Clef(staff: 1, sign: ClefSign.percussion, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 120,
      directions: const [
        Direction(
          staff: 1,
          positionDivisions: 0,
          kind: DirectionKind.metronome(beatUnit: 'quarter', perMinute: 120),
        ),
      ],
      notes: [
        for (var i = 0; i < 8; i++)
          noteEvent(
            positionDivisions: i,
            durationDivisions: 1,
            noteType: 'eighth',
            unpitched: const Unpitched(
              displayStep: 'G',
              displayOctave: 5,
              gmNumber: 42,
              headClass: HeadClass.x,
            ),
            instrumentId: 'P1-I42',
          ),
        noteEvent(
          positionDivisions: 2,
          durationDivisions: 1,
          isChord: true,
          noteType: 'eighth',
          unpitched: const Unpitched(
            displayStep: 'C',
            displayOctave: 5,
            gmNumber: 38,
            headClass: HeadClass.oval,
          ),
          instrumentId: 'P1-I38',
        ),
        noteEvent(
          positionDivisions: 0,
          voice: 2,
          durationDivisions: 2,
          noteType: 'quarter',
          stem: StemDir.down,
          unpitched: const Unpitched(
            displayStep: 'F',
            displayOctave: 4,
            gmNumber: 36,
            headClass: HeadClass.oval,
          ),
          instrumentId: 'P1-I36',
        ),
        noteEvent(
          positionDivisions: 4,
          voice: 2,
          durationDivisions: 2,
          noteType: 'quarter',
          stem: StemDir.down,
          unpitched: const Unpitched(
            displayStep: 'F',
            displayOctave: 4,
            gmNumber: 36,
            headClass: HeadClass.oval,
          ),
          instrumentId: 'P1-I36',
        ),
      ],
    ),
  ],
);

/// A percussion document mirroring the first bar of the real
/// `assets/scores/intermediate/groove_ouvert.musicxml` fixture (change:
/// add-drum-notation-render) — widget tests cannot run the native parser, so
/// the fake reproduces the shipped file's content: voice 1 (hands, stems up)
/// opens with a crash (A5, GM 49) + closed hi-hat (G5, GM 42) quarter chord,
/// runs hi-hat eighths with snare (C5, GM 38) chords on beats 2 and 4, and
/// closes on an **open** hi-hat (GM 46); voice 2 (feet, stems down) plays
/// kick (F4, GM 36) eighths with eighth/quarter rests between them. The
/// `headClass` values are set explicitly to mirror the crate's table
/// (cymbals → x, GM 46 → xOpen, drums → oval) — production code always takes
/// the class from the bridge, and the drift test pins this mirror.
///
/// [keyFifths] builds the synthetic `fifths ≠ 2` variant of the engraving
/// spec: a percussion staff must never draw an armature even then.
ScoreDocument sampleOpenGrooveDocument({int keyFifths = 0}) {
  const closedHat = Unpitched(
    displayStep: 'G',
    displayOctave: 5,
    gmNumber: 42,
    headClass: HeadClass.x,
  );
  const openHat = Unpitched(
    displayStep: 'G',
    displayOctave: 5,
    gmNumber: 46,
    headClass: HeadClass.xOpen,
  );
  const crash = Unpitched(
    displayStep: 'A',
    displayOctave: 5,
    gmNumber: 49,
    headClass: HeadClass.x,
  );
  const snare = Unpitched(
    displayStep: 'C',
    displayOctave: 5,
    gmNumber: 38,
    headClass: HeadClass.oval,
  );
  const kick = Unpitched(
    displayStep: 'F',
    displayOctave: 4,
    gmNumber: 36,
    headClass: HeadClass.oval,
  );
  NoteEvent hat(int pos, {Unpitched piece = closedHat}) => noteEvent(
    positionDivisions: pos,
    durationDivisions: 2,
    noteType: 'eighth',
    stem: StemDir.up,
    unpitched: piece,
    instrumentId: 'P1-I${piece.gmNumber}',
  );
  NoteEvent snareChord(int pos) => noteEvent(
    positionDivisions: pos,
    durationDivisions: 2,
    isChord: true,
    noteType: 'eighth',
    stem: StemDir.up,
    unpitched: snare,
    instrumentId: 'P1-I38',
  );
  NoteEvent kickAt(int pos) => noteEvent(
    positionDivisions: pos,
    voice: 2,
    durationDivisions: 2,
    noteType: 'eighth',
    stem: StemDir.down,
    unpitched: kick,
    instrumentId: 'P1-I36',
  );
  NoteEvent restAt(int pos, int duration, String type) => noteEvent(
    positionDivisions: pos,
    voice: 2,
    isRest: true,
    durationDivisions: duration,
    noteType: type,
  );
  return ScoreDocument(
    instruments: const [
      InstrumentDecl(id: 'P1-I36', name: 'Bass Drum 1', gmNumber: 36),
      InstrumentDecl(id: 'P1-I38', name: 'Snare Drum', gmNumber: 38),
      InstrumentDecl(id: 'P1-I42', name: 'Closed Hi-Hat', gmNumber: 42),
      InstrumentDecl(id: 'P1-I46', name: 'Open Hi-Hat', gmNumber: 46),
      InstrumentDecl(id: 'P1-I49', name: 'Crash Cymbal 1', gmNumber: 49),
    ],
    playOrder: const [],
    meta: const ScoreMeta(title: 'Groove ouvert', composer: 'Cymbra'),
    staves: 1,
    attributes: Attributes(
      divisions: 4,
      clefs: const [Clef(staff: 1, sign: ClefSign.percussion, line: 2)],
      keyFifths: keyFifths,
      time: const TimeSignature(beats: 4, beatType: 4),
    ),
    measures: [
      NotationMeasure(
        repeats: noRepeats,
        index: 0,
        clefs: const [],
        keyFifths: keyFifths,
        minWidth: 200,
        directions: const [
          Direction(
            staff: 1,
            positionDivisions: 0,
            kind: DirectionKind.metronome(beatUnit: 'quarter', perMinute: 100),
          ),
        ],
        notes: [
          // Voice 1 — hands: crash + closed hat chord on beat 1…
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 4,
            noteType: 'quarter',
            stem: StemDir.up,
            unpitched: crash,
            instrumentId: 'P1-I49',
          ),
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 4,
            isChord: true,
            noteType: 'quarter',
            stem: StemDir.up,
            unpitched: closedHat,
            instrumentId: 'P1-I42',
          ),
          // …hat eighths with snare chords on beats 2 and 4…
          hat(4),
          snareChord(4),
          hat(6),
          hat(8),
          hat(10),
          hat(12),
          snareChord(12),
          // …and the closing OPEN hi-hat stroke.
          hat(14, piece: openHat),
          // Voice 2 — feet: kick eighths with the groove's rests between.
          kickAt(0),
          restAt(2, 2, 'eighth'),
          restAt(4, 4, 'quarter'),
          kickAt(8),
          kickAt(10),
          restAt(12, 4, 'quarter'),
        ],
      ),
      // A second bar — the straight version of the groove — so range pickers
      // (measure select) have more than one written measure to resolve.
      NotationMeasure(
        repeats: noRepeats,
        index: 1,
        clefs: const [],
        keyFifths: keyFifths,
        minWidth: 200,
        directions: const [],
        notes: [
          for (var i = 0; i < 8; i++) hat(i * 2),
          snareChord(4),
          snareChord(12),
          for (var beat = 0; beat < 4; beat++)
            noteEvent(
              positionDivisions: beat * 4,
              voice: 2,
              durationDivisions: 4,
              noteType: 'quarter',
              stem: StemDir.down,
              unpitched: kick,
              instrumentId: 'P1-I36',
            ),
        ],
      ),
    ],
  );
}

/// A percussion measure where both voices strike the SAME written position at
/// the same instant — a low floor tom (F4, voice 1) over the kick (F4,
/// voice 2) — so the shared-onset offsetting rule is observable: without it
/// one head hides the other completely.
ScoreDocument samplePercussionSharedPositionDocument() => ScoreDocument(
  instruments: const [
    InstrumentDecl(id: 'P1-I36', name: 'Bass Drum 1', gmNumber: 36),
    InstrumentDecl(id: 'P1-I41', name: 'Low Floor Tom', gmNumber: 41),
  ],
  playOrder: const [],
  meta: const ScoreMeta(title: 'SharedOnset', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.percussion, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 160,
      directions: const [],
      notes: [
        noteEvent(
          positionDivisions: 0,
          durationDivisions: 4,
          noteType: 'quarter',
          unpitched: const Unpitched(
            displayStep: 'F',
            displayOctave: 4,
            gmNumber: 41,
            headClass: HeadClass.oval,
          ),
          instrumentId: 'P1-I41',
        ),
        noteEvent(
          positionDivisions: 0,
          voice: 2,
          durationDivisions: 4,
          noteType: 'quarter',
          unpitched: const Unpitched(
            displayStep: 'F',
            displayOctave: 4,
            gmNumber: 36,
            headClass: HeadClass.oval,
          ),
          instrumentId: 'P1-I36',
        ),
      ],
    ),
  ],
);

ScoreDocument sampleGrandStaffDocument() => ScoreDocument(
  instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'Sample', composer: 'Tester'),
  staves: 2,
  attributes: const Attributes(
    divisions: 4,
    clefs: [
      Clef(staff: 1, sign: ClefSign.g, line: 2),
      Clef(staff: 2, sign: ClefSign.f, line: 4),
    ],
    keyFifths: 3, // 3 sharps → shows the key signature (armature)
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      // The parser carries the key in force into every measure, opening one
      // included (see `key_fifths` in crates/musicxml-core/src/lib.rs), and the
      // painters read the measure's key rather than the document's. A measure
      // declaring 0 under a 3-sharp document is not something a real parse can
      // produce, and it silently cost this fixture its armature coverage.
      keyFifths: 3,
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

/// A four-measure treble document with **round** playback timing, for the
/// measure-range practice tests (change: add-measure-range-practice): a
/// `quarter = 60` metronome mark with `divisions = 4` gives 250 ms per division
/// and 16 divisions per 4/4 measure, so `measureStartMs` is `[0, 4000, 8000,
/// 12000]` and `songEndMs` is 16000. Each measure holds one whole note, so every
/// measure has an onset the Wait-Mode gate and the scorer can see.
ScoreDocument sampleFourMeasureDocument() => ScoreDocument(
  instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'FourBars', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    for (var i = 0; i < 4; i++)
      NotationMeasure(
        repeats: noRepeats,
        index: i,
        clefs: const [],
        keyFifths: 0,
        minWidth: 120,
        directions: i == 0
            ? const [
                Direction(
                  staff: 1,
                  positionDivisions: 0,
                  kind: DirectionKind.metronome(
                    beatUnit: 'quarter',
                    perMinute: 60,
                  ),
                ),
              ]
            : const [],
        notes: [
          noteEvent(
            staff: 1,
            // C5, D5, E5, F5 — one distinct onset per measure.
            pitch: Pitch(step: ['C', 'D', 'E', 'F'][i], octave: 5, alter: 0),
            durationDivisions: 16,
            noteType: 'whole',
          ),
        ],
      ),
  ],
);

/// Two whole-note bars at 60 bpm with the first repeated (engine-resolved
/// playOrder [0, 0, 1]) — the smallest repeat-carrying piece: unrolled tables
/// have three 4000ms slots, the written-linear (practice) ones two.
ScoreDocument sampleRepeatDocument() => ScoreDocument(
  instruments: const [],
  playOrder: const [
    PlayedMeasure(writtenIndex: 0, pass: 1),
    PlayedMeasure(writtenIndex: 0, pass: 2),
    PlayedMeasure(writtenIndex: 1, pass: 1),
  ],
  meta: const ScoreMeta(title: 'RepeatBars', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: repeatMarks(backwardTimes: 2),
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 120,
      directions: const [
        Direction(
          staff: 1,
          positionDivisions: 0,
          kind: DirectionKind.metronome(beatUnit: 'quarter', perMinute: 60),
        ),
      ],
      notes: [
        noteEvent(
          positionDivisions: 0,
          durationDivisions: 16,
          noteType: 'whole',
          pitch: const Pitch(step: 'C', octave: 4, alter: 0),
        ),
      ],
    ),
    NotationMeasure(
      repeats: noRepeats,
      index: 1,
      clefs: const [],
      keyFifths: 0,
      minWidth: 120,
      directions: const [],
      notes: [
        noteEvent(
          positionDivisions: 0,
          durationDivisions: 16,
          noteType: 'whole',
          pitch: const Pitch(step: 'D', octave: 4, alter: 0),
        ),
      ],
    ),
  ],
);

/// Like [sampleFourMeasureDocument] but with [bars] measures, so a test can make
/// the practice ribbon genuinely scrollable (four bars fit without scrolling,
/// which hid the auto-scroll bug).
ScoreDocument sampleManyMeasureDocument({int bars = 24}) => ScoreDocument(
  instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'ManyBars', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    for (var i = 0; i < bars; i++)
      NotationMeasure(
        repeats: noRepeats,
        index: i,
        clefs: const [],
        keyFifths: 0,
        minWidth: 120,
        directions: i == 0
            ? const [
                Direction(
                  staff: 1,
                  positionDivisions: 0,
                  kind: DirectionKind.metronome(
                    beatUnit: 'quarter',
                    perMinute: 60,
                  ),
                ),
              ]
            : const [],
        notes: [
          noteEvent(
            staff: 1,
            pitch: Pitch(step: 'CDEFGAB'[i % 7], octave: 5, alter: 0),
            durationDivisions: 16,
            noteType: 'whole',
          ),
        ],
      ),
  ],
);
