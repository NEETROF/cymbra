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
  instruments: const [],
  playOrder: const [],
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
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 100,
      directions: directions,
      notes: notes,
    ),
  ],
);

/// A multi-measure document with an explicit engine-resolved [playOrder].
ScoreDocument _docWithMeasures(
  List<NotationMeasure> measures, {
  List<PlayedMeasure> playOrder = const [],
}) => ScoreDocument(
  instruments: const [],
  playOrder: playOrder,
  meta: const ScoreMeta(title: 'T', composer: 'C'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: measures,
);

NotationMeasure _measure(
  int index,
  List<NoteEvent> notes, {
  RepeatMarks? repeats,
  int keyFifths = 0,
}) => NotationMeasure(
  repeats: repeats ?? noRepeats,
  index: index,
  clefs: const [],
  keyFifths: keyFifths,
  minWidth: 100,
  directions: const [],
  notes: notes,
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
      // The chord flag survives the flattening: the Staff painter needs it so a
      // member never grows a stem/flag of its own next to the principal's.
      expect(d.notes.map((n) => n.isChord), [false, true]);
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
          instruments: const [],
          playOrder: const [],
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
              repeats: noRepeats,
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
              repeats: noRepeats,
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
        repeats: noRepeats,
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
        instruments: const [],
        playOrder: const [],
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
          instruments: const [],
          playOrder: const [],
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
              repeats: noRepeats,
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
              repeats: noRepeats,
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
          // The waterfall's attack/sustain split: the sustain starts where the
          // FIRST continuation begins, and later links never move it.
          expect(d.notes.single.sustainFromMs, (quarterMs / 2).round());
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

    // A grace note parses with duration 0 at its principal's position (the
    // cursor does not advance). Scheduling it verbatim made a zero-length note
    // glued onto the principal — inaudible, invisible on the waterfall, and
    // engraved on top of it (the River Flows In You measure-6 report). It gets
    // a short nominal duration just before the principal instead — the same
    // rule as the Rust schedule (grace_ms = quarter/8), so app and back-office
    // previews agree.
    group('grace notes', () {
      const quarterMs = 60000 / kDefaultBpm;
      const graceMs = quarterMs / 8;

      test(
        'a grace plays briefly before its principal, which is unchanged',
        () {
          final doc = _docWith(
            notes: [
              noteEvent(
                positionDivisions: 0,
                durationDivisions: 8,
                noteType: 'half',
                pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              ),
              noteEvent(
                positionDivisions: 8,
                durationDivisions: 0,
                isGrace: true,
                noteType: 'eighth',
                pitch: const Pitch(step: 'B', octave: 4, alter: 0),
              ),
              noteEvent(
                positionDivisions: 8,
                durationDivisions: 4,
                noteType: 'eighth',
                pitch: const Pitch(step: 'C', octave: 5, alter: 1),
              ),
            ],
          );
          final d = notationToTimedNotes(doc);
          final grace = d.notes.singleWhere((n) => n.pitch == 71);
          final principal = d.notes.singleWhere((n) => n.pitch == 73);
          expect(grace.isGrace, isTrue);
          expect(grace.startMs, (quarterMs * 2 - graceMs).round());
          expect(grace.durationMs, graceMs.round());
          expect(principal.isGrace, isFalse);
          expect(principal.startMs, (quarterMs * 2).round());
          expect(principal.durationMs, quarterMs.round());
        },
      );

      test('consecutive graces stack backwards in document order', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 8,
              durationDivisions: 0,
              isGrace: true,
              pitch: const Pitch(step: 'A', octave: 4, alter: 0),
            ),
            noteEvent(
              positionDivisions: 8,
              durationDivisions: 0,
              isGrace: true,
              pitch: const Pitch(step: 'B', octave: 4, alter: 0),
            ),
            noteEvent(
              positionDivisions: 8,
              durationDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 5, alter: 0),
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        final a = d.notes.singleWhere((n) => n.pitch == 69);
        final b = d.notes.singleWhere((n) => n.pitch == 71);
        final c = d.notes.singleWhere((n) => n.pitch == 72);
        expect(a.startMs, (quarterMs * 2 - 2 * graceMs).round());
        expect(b.startMs, (quarterMs * 2 - graceMs).round());
        expect(c.startMs, (quarterMs * 2).round());
      });

      test('a grace opening the piece is clamped at time zero', () {
        final doc = _docWith(
          notes: [
            noteEvent(
              positionDivisions: 0,
              durationDivisions: 0,
              isGrace: true,
              pitch: const Pitch(step: 'B', octave: 4, alter: 0),
            ),
            noteEvent(
              positionDivisions: 0,
              durationDivisions: 4,
              pitch: const Pitch(step: 'C', octave: 5, alter: 0),
            ),
          ],
        );
        final d = notationToTimedNotes(doc);
        final grace = d.notes.singleWhere((n) => n.pitch == 71);
        expect(grace.startMs, 0);
        expect(grace.durationMs, graceMs.round());
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

  // The engine resolves the playback order at parse time; the derivation must
  // follow it — repeated sections once per pass, `%` slots replaying their
  // source, the played tables carrying the written-measure mapping — and a
  // selective practice run must be able to opt back into the written order.
  group('repeat unrolling', () {
    const quarterMs = 60000 / kDefaultBpm;
    const wholeMs = quarterMs * 4;

    NoteEvent whole(String step) => noteEvent(
      positionDivisions: 0,
      durationDivisions: 16,
      noteType: 'whole',
      pitch: Pitch(step: step, octave: 4, alter: 0),
    );

    test('a repeated measure sounds once per played slot', () {
      final doc = _docWithMeasures(
        [
          _measure(0, [whole('C')], repeats: repeatMarks(backwardTimes: 2)),
          _measure(1, [whole('D')]),
        ],
        playOrder: const [
          PlayedMeasure(writtenIndex: 0, pass: 1),
          PlayedMeasure(writtenIndex: 0, pass: 2),
          PlayedMeasure(writtenIndex: 1, pass: 1),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes.map((n) => (n.pitch, n.startMs)), [
        (60, 0),
        (60, wholeMs.round()),
        (62, (wholeMs * 2).round()),
      ]);
      expect(d.measureStartMs, [0, wholeMs.round(), (wholeMs * 2).round()]);
      expect(d.writtenMeasureOf, [0, 0, 1]);
      expect(d.songEndMs, closeTo(wholeMs * 3, 1));
      // The scrolling staff's decorations follow the written marks per slot.
      expect(d.measureDecors.map((m) => m.repeatBackward), [true, true, false]);
    });

    test('a measure-repeat slot replays its source content', () {
      final doc = _docWithMeasures(
        [
          _measure(0, [whole('C')]),
          _measure(1, const [], repeats: repeatMarks(measureRepeatOf: 0)),
        ],
        playOrder: const [
          PlayedMeasure(writtenIndex: 0, pass: 1),
          PlayedMeasure(writtenIndex: 1, pass: 1),
        ],
      );
      final d = notationToTimedNotes(doc);
      // The `%` slot sounds the source measure instead of a bar of silence.
      expect(d.notes.map((n) => (n.pitch, n.startMs)), [
        (60, 0),
        (60, wholeMs.round()),
      ]);
      expect(d.writtenMeasureOf, [0, 1]);
      expect(d.measureDecors[1].measureRepeat, isTrue);
    });

    test('unroll: false plays the written order once (practice)', () {
      final doc = _docWithMeasures(
        [
          _measure(0, [whole('C')], repeats: repeatMarks(backwardTimes: 2)),
          _measure(1, [whole('D')]),
        ],
        playOrder: const [
          PlayedMeasure(writtenIndex: 0, pass: 1),
          PlayedMeasure(writtenIndex: 0, pass: 2),
          PlayedMeasure(writtenIndex: 1, pass: 1),
        ],
      );
      final d = notationToTimedNotes(doc, unroll: false);
      expect(d.notes.map((n) => (n.pitch, n.startMs)), [
        (60, 0),
        (62, wholeMs.round()),
      ]);
      expect(d.measureStartMs, [0, wholeMs.round()]);
    });

    test('ties merge across played slots, not written adjacency', () {
      // C4 whole tied forward in a repeated measure: on each pass the chain
      // re-attacks (the continuation only abuts within the same pass).
      final first = noteEvent(
        positionDivisions: 0,
        durationDivisions: 16,
        noteType: 'whole',
        pitch: const Pitch(step: 'C', octave: 4, alter: 0),
        tieStart: true,
      );
      final cont = noteEvent(
        positionDivisions: 0,
        durationDivisions: 16,
        noteType: 'whole',
        pitch: const Pitch(step: 'C', octave: 4, alter: 0),
        tieStop: true,
      );
      final doc = _docWithMeasures(
        [
          _measure(0, [first]),
          _measure(1, [cont]),
        ],
        playOrder: const [
          PlayedMeasure(writtenIndex: 0, pass: 1),
          PlayedMeasure(writtenIndex: 1, pass: 1),
        ],
      );
      final d = notationToTimedNotes(doc);
      // Played adjacency holds here: one merged attack spanning both slots.
      expect(d.notes, hasLength(1));
      expect(d.notes.single.durationMs, (wholeMs * 2).round());
    });
  });

  group('percussion (add-unpitched-notation)', () {
    // The Dart mirror of the crate's ROCK_GROOVE fixture (task 1.3): two
    // measures at 120 bpm, divisions 2 — closed hi-hat (GM 42) eighths with
    // snare (GM 38) chords on beats 2 and 4 in voice 1, kick (GM 36) on
    // beats 1 and 3 in voice 2. The parity test below asserts the exact
    // numbers the crate test `percussion_score_schedules_gm_numbers` asserts,
    // so a drift between the two schedules fails a test on whichever side
    // moved.
    NoteEvent hat(int pos) => noteEvent(
      positionDivisions: pos,
      durationDivisions: 1,
      noteType: 'eighth',
      unpitched: const Unpitched(
        displayStep: 'G',
        displayOctave: 5,
        gmNumber: 42,
        headClass: HeadClass.x,
      ),
      instrumentId: 'P1-I42',
    );
    NoteEvent snare(int pos) => noteEvent(
      positionDivisions: pos,
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
    );
    NoteEvent kick(int pos) => noteEvent(
      positionDivisions: pos,
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
    );

    ScoreDocument grooveDoc() => ScoreDocument(
      instruments: const [
        InstrumentDecl(id: 'P1-I36', name: 'Bass Drum 1', gmNumber: 36),
        InstrumentDecl(id: 'P1-I38', name: 'Snare Drum', gmNumber: 38),
        InstrumentDecl(id: 'P1-I42', name: 'Closed Hi-hat', gmNumber: 42),
      ],
      playOrder: const [],
      meta: const ScoreMeta(title: 'Groove', composer: null),
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
          minWidth: 100,
          directions: [
            Direction(
              staff: 1,
              positionDivisions: 0,
              kind: DirectionKind.metronome(
                beatUnit: 'quarter',
                perMinute: 120,
              ),
            ),
          ],
          notes: [
            for (var i = 0; i < 8; i++) ...[
              hat(i),
              if (i == 2 || i == 6) snare(i),
            ],
            kick(0),
            noteEvent(
              positionDivisions: 2,
              voice: 2,
              durationDivisions: 2,
              isRest: true,
            ),
            kick(4),
            noteEvent(
              positionDivisions: 6,
              voice: 2,
              durationDivisions: 2,
              isRest: true,
            ),
          ],
        ),
        NotationMeasure(
          repeats: noRepeats,
          index: 1,
          clefs: const [],
          keyFifths: 0,
          minWidth: 100,
          directions: const [],
          notes: [
            kick(0),
            noteEvent(
              positionDivisions: 2,
              voice: 2,
              durationDivisions: 6,
              isRest: true,
              noteType: 'half',
              dots: 1,
            ),
          ],
        ),
      ],
    );

    test('a percussion score schedules its General MIDI numbers (parity)', () {
      final d = notationToTimedNotes(grooveDoc());
      // Same expectations as the crate's percussion_score_schedules_gm_numbers.
      final kicks = d.notes
          .where((n) => n.pitch == 36)
          .map((n) => n.startMs)
          .toList();
      expect(kicks, [0, 1000, 2000]);
      final snares = d.notes
          .where((n) => n.pitch == 38)
          .map((n) => n.startMs)
          .toList();
      expect(snares, [500, 1500]);
      final hats = d.notes
          .where((n) => n.pitch == 42)
          .map((n) => n.startMs)
          .toList();
      expect(hats, [0, 250, 500, 750, 1000, 1250, 1500, 1750]);
      // The written position feeds the diatonic index like a pitch would.
      expect(
        d.notes.firstWhere((n) => n.pitch == 42).diatonic,
        5 * 7 + 4, // G5
      );
      expect(d.notes.firstWhere((n) => n.pitch == 42).clefSign, 'percussion');
    });

    test('an unresolvable unpitched note is omitted, never fabricated', () {
      final doc = _docWith(
        divisions: 2,
        notes: [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 2,
            unpitched: const Unpitched(
              displayStep: 'C',
              displayOctave: 5,
              gmNumber: 38,
              headClass: HeadClass.oval,
            ),
          ),
          noteEvent(
            positionDivisions: 2,
            durationDivisions: 2,
            unpitched: const Unpitched(
              displayStep: 'E',
              displayOctave: 5,
              gmNumber: null,
              headClass: HeadClass.oval,
            ),
          ),
          noteEvent(
            positionDivisions: 4,
            durationDivisions: 2,
            unpitched: const Unpitched(
              displayStep: 'C',
              displayOctave: 5,
              gmNumber: 38,
              headClass: HeadClass.oval,
            ),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      // The middle note is dropped; its neighbour keeps its computed time.
      expect(d.notes, hasLength(2));
      expect(d.notes[1].startMs, greaterThan(d.notes[0].startMs));
    });

    test('a mixed score keeps today\'s behaviour: unpitched stays silent', () {
      final doc = _docWith(
        notes: [
          noteEvent(
            positionDivisions: 0,
            pitch: const Pitch(step: 'C', octave: 4, alter: 0),
          ),
          noteEvent(
            positionDivisions: 4,
            unpitched: const Unpitched(
              displayStep: 'C',
              displayOctave: 5,
              gmNumber: 38,
              headClass: HeadClass.oval,
            ),
          ),
          noteEvent(
            positionDivisions: 8,
            pitch: const Pitch(step: 'E', octave: 4, alter: 0),
          ),
        ],
      );
      final d = notationToTimedNotes(doc);
      expect(d.notes.map((n) => n.pitch), [60, 64]);
    });

    test('tied unpitched chains merge into one prolonged note', () {
      // A crash (GM 49) whole tied across the barline, then a fresh attack —
      // mirrors the crate's tied_unpitched_chain_merges test: one 4000 ms
      // attack, then a separate 2000 ms one.
      const crash = Unpitched(
        displayStep: 'A',
        displayOctave: 5,
        gmNumber: 49,
        headClass: HeadClass.x,
      );
      final doc = _docWithMeasures([
        _measure(0, [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 16,
            noteType: 'whole',
            tieStart: true,
            unpitched: crash,
          ),
        ]),
        _measure(1, [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 16,
            noteType: 'whole',
            tieStop: true,
            unpitched: crash,
          ),
        ]),
        _measure(2, [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 16,
            noteType: 'whole',
            unpitched: crash,
          ),
        ]),
      ]);
      // divisions 4, default 90 bpm → whole (16 div) = 16 * 666.67 ms.
      final d = notationToTimedNotes(doc);
      expect(d.notes, hasLength(2));
      const wholeMs = 16 * (60000.0 / 90) / 4;
      expect(d.notes[0].startMs, 0);
      expect(d.notes[0].durationMs, (wholeMs * 2).round());
      expect(d.notes[1].startMs, (wholeMs * 2).round());
      // The merged continuation is kept on the render-only channel.
      expect(d.tieContinuations, hasLength(1));
    });
  });
}
