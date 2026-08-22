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
      clefs: const [],
      keyFifths: 0,
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

    test('rests produce no played note but land on the rests channel', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
          noteEvent(
            positionDivisions: 4,
            durationDivisions: 4,
            isRest: true,
            pitch: null,
            noteType: 'quarter',
            dots: 1,
          ),
          noteEvent(
            positionDivisions: 8,
            pitch: const Pitch(step: 'E', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      // The rest is kept out of the playable/scored notes...
      expect(d.notes, hasLength(2));
      expect(d.notes.map((n) => n.pitch), [60, 64]);
      // ...and surfaced on its own render-only channel with its type/dots.
      expect(d.rests, hasLength(1));
      final quarterMs = (60000 / kDefaultBpm).round();
      expect(d.rests.single.startMs, quarterMs);
      expect(d.rests.single.noteType, 'quarter');
      expect(d.rests.single.dots, 1);
    });

    test('note type and dots are carried onto the timed note', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 12,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
            noteType: 'half',
            dots: 1,
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes.single.noteType, 'half');
      expect(d.notes.single.dots, 1);
      expect(d.rests, isEmpty);
    });

    // The scrolling Portée engraves the accidental and follows the notation's
    // stem direction, so both have to survive the flattening — dropping them is
    // what made a D♯ read as a plain D there while the Partition showed it.
    test('accidental and stem direction are carried onto the timed note', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'D', octave: 5, alter: 1),
            noteType: 'eighth',
            accidental: 'sharp',
            stem: StemDir.down,
          ),
          noteEvent(
            positionDivisions: 4,
            pitch: const Pitch(step: 'D', octave: 5, alter: 0),
            noteType: 'eighth',
            accidental: 'natural',
            stem: StemDir.up,
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes.map((n) => n.accidental), ['sharp', 'natural']);
      expect(d.notes.map((n) => n.stemUp), [false, true]);
    });

    test('a note with no accidental or stem carries neither', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes.single.accidental, isNull);
      expect(d.notes.single.stemUp, isNull);
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
      const halfMs = (60000 / kDefaultBpm) * 2; // 8 divisions = 2 quarters
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
              clefs: const [],
              keyFifths: 0,
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
              clefs: const [],
              keyFifths: 0,
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
        const measureMs = (60000 / kDefaultBpm) * 4;
        expect(d.notes[1].startMs, closeTo(measureMs, 1));
      },
    );

    test('exposes the per-measure key signature for a modulating piece', () {
      NotationMeasure measure(int index, int keyFifths) => NotationMeasure(
        index: index,
        clefs: const [],
        keyFifths: keyFifths,
        minWidth: 100,
        directions: const [],
        notes: [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 16,
            pitch: const Pitch(step: 'C', octave: 5, alter: 0),
          ),
        ],
      );
      final doc = ScoreDocument(
        meta: const ScoreMeta(title: 'T', composer: 'C'),
        staves: 1,
        attributes: const Attributes(
          divisions: 4,
          clefs: [],
          keyFifths: -1,
          time: TimeSignature(beats: 4, beatType: 4),
        ),
        // Starts in 4 flats, modulates to 1 flat — like Haydn's canzonet.
        measures: [measure(0, -4), measure(1, -4), measure(2, -1)],
      );
      final d = notationToTimedNotes(doc);
      expect(d.measureKeyFifths, [-4, -4, -1]);
      expect(d.measureKeyFifths, hasLength(d.measureStartMs.length));
    });

    // A tied note is a single attack: the continuation must extend the note it
    // prolongs, never become a playable onset of its own — gating it made Wait
    // Mode demand a re-attack of a key the score says to keep holding (the
    // "Mariage d'Amour" freeze on chords with tied members).
    group('tie merging', () {
      const quarterMs = 60000 / kDefaultBpm; // divisions 4 → one quarter

      test('a tie continuation extends the note instead of adding an onset', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStart: true,
            ),
            noteEvent(
              positionDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStop: true,
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        expect(d.notes, hasLength(1));
        expect(d.notes.single.startMs, 0);
        // End-aligned: round(end) − round(start), not a sum of rounded halves.
        expect(d.notes.single.durationMs, (quarterMs * 2).round());
        expect(d.songEndMs, closeTo(quarterMs * 2, 0.001));
      });

      test('a three-link chain (stop+start middle) collapses into one', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStart: true,
            ),
            noteEvent(
              positionDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStop: true,
              tieStart: true,
            ),
            noteEvent(
              positionDivisions: 8,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStop: true,
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        expect(d.notes, hasLength(1));
        expect(d.notes.single.durationMs, (quarterMs * 3).round());
      });

      test('a tie across the barline merges', () {
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
              clefs: const [],
              keyFifths: 0,
              minWidth: 100,
              directions: const [],
              notes: [
                noteEvent(
                  positionDivisions: 0,
                  durationDivisions: 16,
                  noteType: 'whole',
                  pitch: const Pitch(step: 'C', octave: 4, alter: 0),
                  tieStart: true,
                ),
              ],
            ),
            NotationMeasure(
              index: 1,
              clefs: const [],
              keyFifths: 0,
              minWidth: 100,
              directions: const [],
              notes: [
                noteEvent(
                  positionDivisions: 0,
                  pitch: const Pitch(step: 'C', octave: 4, alter: 0),
                  tieStop: true,
                ),
              ],
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        expect(d.notes, hasLength(1));
        expect(d.notes.single.durationMs, (quarterMs * 5).round());
      });

      test('a chord with one tied member gates only the fresh member', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStart: true,
            ),
            noteEvent(
              positionDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStop: true,
            ),
            noteEvent(
              positionDivisions: 4,
              isChord: true,
              pitch: const Pitch(step: 'E', octave: 4, alter: 0),
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        // The tied C is one long note; only the fresh E starts at the second
        // onset — so Wait Mode awaits E alone there.
        expect(d.notes, hasLength(2));
        final c = d.notes.singleWhere((n) => n.pitch == 60);
        final e = d.notes.singleWhere((n) => n.pitch == 64);
        expect(c.startMs, 0);
        expect(c.durationMs, (quarterMs * 2).round());
        expect(e.startMs, quarterMs.round());
      });

      test('a dangling tie stop stays a playable note', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              pitch: const Pitch(step: 'D', octave: 4, alter: 0),
            ),
            noteEvent(
              positionDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStop: true,
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        expect(d.notes, hasLength(2));
      });

      test('repeated same-pitch notes without ties stay separate onsets', () {
        // The Wait-Mode honesty rule depends on this: a repeat is two attacks.
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
            ),
            noteEvent(
              positionDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
            ),
          ],
        );
        expect(notationToTimedNotes(doc).notes, hasLength(2));
      });

      test('ties do not merge across voices', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              voice: 1,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStart: true,
            ),
            noteEvent(
              positionDivisions: 4,
              voice: 2,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              tieStop: true,
            ),
          ],
        );
        expect(notationToTimedNotes(doc).notes, hasLength(2));
      });

      // The merged continuations must NOT vanish from the notation: dropping
      // them left the prolonged measures looking empty (an eighth "stretching"
      // over the next bar — the River Flows In You report). They come back on a
      // render-only channel, engraved as written and anchored for the tie arc.
      test(
        'continuations surface on the render-only channel with arc anchors',
        () {
          final doc = _docWith(
            notes: [
              // River-like chain: eighth (tie start) → dotted half (stop+start)
              // → eighth (stop). One attack, three engraved notes.
              noteEvent(
                positionDivisions: 0,
                durationDivisions: 2,
                noteType: 'eighth',
                pitch: const Pitch(step: 'D', octave: 5, alter: 0),
                tieStart: true,
              ),
              noteEvent(
                positionDivisions: 2,
                durationDivisions: 12,
                noteType: 'half',
                dots: 1,
                pitch: const Pitch(step: 'D', octave: 5, alter: 0),
                tieStop: true,
                tieStart: true,
              ),
              noteEvent(
                positionDivisions: 14,
                durationDivisions: 2,
                noteType: 'eighth',
                pitch: const Pitch(step: 'D', octave: 5, alter: 0),
                tieStop: true,
              ),
            ],
          );
          final d = notationToTimedNotes(doc);
          // Playback: one merged attack spanning the whole chain, keeping the
          // first note's engraved figure.
          expect(d.notes, hasLength(1));
          expect(d.notes.single.noteType, 'eighth');
          expect(d.notes.single.durationMs, (quarterMs * 4).round());
          // Notation: both continuations kept, as written, each anchored to the
          // engraved note it prolongs (a chain arcs link-to-link, not all-to-first).
          expect(d.tieContinuations, hasLength(2));
          final half = d.tieContinuations[0];
          expect(half.noteType, 'half');
          expect(half.dots, 1);
          expect(half.startMs, (quarterMs / 2).round());
          expect(half.tieFromMs, 0);
          final eighth = d.tieContinuations[1];
          expect(eighth.noteType, 'eighth');
          expect(eighth.startMs, (quarterMs * 3.5).round());
          expect(eighth.tieFromMs, half.startMs);
        },
      );

      test('a plain playable note carries no arc anchor', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        expect(d.notes.single.tieFromMs, isNull);
        expect(d.tieContinuations, isEmpty);
      });
    });

    test('carries the written diatonic step, not the MIDI collapse', () {
      // A♭4 (step A, octave 4, alter −1) must sit on the A line/space, like the
      // engraved Partition — never collapsed onto G via its MIDI number. The
      // Staff painter positions by this value so the two views agree.
      final doc = _docWith(
        notes: [
          noteEvent(
            durationDivisions: 16,
            pitch: const Pitch(step: 'A', octave: 4, alter: -1),
          ),
        ],
      );
      final aFlat = notationToTimedNotes(doc).notes.single;
      expect(aFlat.diatonic, 4 * 7 + 5, reason: 'written A4 position');
      // A G4 would be one diatonic step lower — the two must not coincide.
      expect(aFlat.diatonic, isNot(4 * 7 + 4));
    });
  });
}
