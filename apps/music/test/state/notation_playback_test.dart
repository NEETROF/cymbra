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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/notation_playback.dart';

import '../support/notation_fakes.dart';

ScoreDocument _docWith({
  required List<NoteEvent> notes,
  int divisions = 4,
  int beats = 4,
  int beatType = 4,
  List<Direction> directions = const [],
}) => ScoreDocument(
  meta: const ScoreMeta(title: 'T', composer: 'C'),
  staves: 1,
  attributes: Attributes(
    divisions: divisions,
    clefs: const [],
    keyFifths: 0,
    time: TimeSignature(beats: beats, beatType: beatType),
  ),
  measures: [
    NotationMeasure(
      index: 0,
      minWidth: 100,
      directions: directions,
      notes: notes,
    ),
  ],
);

void main() {
  group('midiOfPitch', () {
    test('C4 is middle C (60)', () {
      expect(midiOfPitch(const Pitch(step: 'C', octave: 4, alter: 0)), 60);
    });
    test('alteration shifts by semitones', () {
      expect(midiOfPitch(const Pitch(step: 'C', octave: 4, alter: 1)), 61);
      expect(midiOfPitch(const Pitch(step: 'A', octave: 4, alter: 0)), 69);
      expect(midiOfPitch(const Pitch(step: 'B', octave: 3, alter: -1)), 58);
    });
  });

  group('notationToTimedNotes', () {
    test('two quarter notes are spaced by one quarter at the given tempo', () {
      // divisions 4, default 90 bpm → quarter = 60000/90 ≈ 666.67 ms.
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
          noteEvent(
            positionDivisions: 4,
            pitch: const Pitch(step: 'D', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.bpm, kDefaultBpm);
      expect(d.notes, hasLength(2));
      expect(d.notes[0].startMs, 0);
      final quarterMs = (60000 / kDefaultBpm).round();
      expect(d.notes[1].startMs, quarterMs);
      expect(d.notes[0].pitch, 60);
      expect(d.notes[1].pitch, 62);
    });

    test('rests produce no played note', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
          noteEvent(positionDivisions: 4, isRest: true, pitch: null),
          noteEvent(
            positionDivisions: 8,
            pitch: const Pitch(step: 'E', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes, hasLength(2));
      expect(d.notes.map((n) => n.pitch), [60, 64]);
    });

    test('a metronome direction overrides the default tempo', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
        ],
        directions: const [
          Direction(
            staff: 1,
            positionDivisions: 0,
            kind: DirectionKind.metronome(beatUnit: 'quarter', perMinute: 120),
          ),
        ],
      );
      expect(notationToTimedNotes(doc).bpm, 120);
    });

    test('chord members share the onset of their note', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
          noteEvent(
            positionDivisions: 0,
            isChord: true,
            pitch: const Pitch(step: 'E', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes, hasLength(2));
      expect(d.notes[0].startMs, d.notes[1].startMs);
    });

    test('songEndMs reaches the end of the last note', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 8,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      final halfMs = (60000 / kDefaultBpm) * 2; // 8 divisions = 2 quarters
      expect(d.songEndMs, closeTo(halfMs, 1));
    });

    test(
      'measures accumulate so the second measure starts after the first',
      () {
        final doc = ScoreDocument(
          meta: const ScoreMeta(title: 'T', composer: 'C'),
          staves: 1,
          attributes: const Attributes(
            divisions: 4,
            clefs: [],
            keyFifths: 0,
            time: TimeSignature(beats: 4, beatType: 4),
          ),
          measures: [
            NotationMeasure(
              index: 0,
              minWidth: 100,
              directions: const [],
              notes: [
                noteEvent(
                  positionDivisions: 0,
                  durationDivisions: 16,
                  pitch: const Pitch(step: 'C', octave: 4, alter: 0),
                ),
              ],
            ),
            NotationMeasure(
              index: 1,
              minWidth: 100,
              directions: const [],
              notes: [
                noteEvent(
                  positionDivisions: 0,
                  durationDivisions: 16,
                  pitch: const Pitch(step: 'D', octave: 4, alter: 0),
                ),
              ],
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        expect(d.notes, hasLength(2));
        // Full 4/4 measure = 16 divisions = 4 quarters.
        final measureMs = (60000 / kDefaultBpm) * 4;
        expect(d.notes[1].startMs, closeTo(measureMs, 1));
      },
    );
  });
}
