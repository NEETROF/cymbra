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

import '../src/rust/api/musicxml.dart';
import 'player_data.dart';

/// Visual playback derived from a parsed [ScoreDocument]: the flattened
/// time-based notes (for the Synthesia/Staff painters), the song end, and the
/// tempo. This is *visual* timing only — no audio or MIDI-out.
class DerivedPlayback {
  final List<TimedNote> notes;

  /// Rests flattened alongside [notes], on their own channel so they render on
  /// the staff without ever entering the playable/scored note set.
  final List<TimedRest> rests;

  final double songEndMs;
  final int bpm;

  /// Start time (ms) of each measure, in document order; the first is 0. Lets the
  /// Partition cursor map a playhead position to a measure and a fraction within
  /// it.
  final List<int> measureStartMs;

  /// Key signature (fifths) in force during each measure, aligned with
  /// [measureStartMs]. Lets the scrolling staff show the armure at the playhead
  /// so a mid-piece modulation is reflected as you scroll past it.
  final List<int> measureKeyFifths;

  const DerivedPlayback({
    required this.notes,
    this.rests = const [],
    required this.songEndMs,
    required this.bpm,
    this.measureStartMs = const [],
    this.measureKeyFifths = const [],
  });
}

/// Default tempo when the score carries no `metronome` direction.
const int kDefaultBpm = 90;

const Map<String, int> _semitoneOfStep = {
  'C': 0,
  'D': 2,
  'E': 4,
  'F': 5,
  'G': 7,
  'A': 9,
  'B': 11,
};

/// Diatonic index of a step within its octave (C=0…B=6) — for the note's
/// written staff position (line/space), independent of any alteration.
const Map<String, int> _diatonicOfStep = {
  'C': 0,
  'D': 1,
  'E': 2,
  'F': 3,
  'G': 4,
  'A': 5,
  'B': 6,
};

/// MIDI note number for a [Pitch] (C4 = 60). Octave 4, step C, alter 0 → 60.
int midiOfPitch(Pitch pitch) {
  final base = _semitoneOfStep[pitch.step] ?? 0;
  return (pitch.octave + 1) * 12 + base + pitch.alter;
}

/// Converts a parsed score into visual playback notes.
///
/// Each non-rest note becomes a [TimedNote] whose start/duration come from its
/// running division position (accumulated across measures) scaled by
/// `ms_per_division = (60000 / bpm) / divisions`. Chord members share their
/// onset (they already carry the same `position_divisions`); rests are skipped.
/// The tempo is the first `metronome` `per-minute` found, else [kDefaultBpm].
DerivedPlayback notationToTimedNotes(ScoreDocument document) {
  final divisions = document.attributes.divisions < 1
      ? 1
      : document.attributes.divisions;
  final bpm = _tempoOf(document);
  final msPerDivision = (60000.0 / bpm) / divisions;

  final time = document.attributes.time;
  final beatType = time.beatType == 0 ? 4 : time.beatType;
  final divisionsPerMeasure = divisions * time.beats * 4 ~/ beatType;

  final notes = <TimedNote>[];
  final rests = <TimedRest>[];
  final measureStartMs = <int>[];
  final measureKeyFifths = <int>[];
  var songEndMs = 0.0;
  var measureStartDiv = 0;

  // Running clef per staff, honouring mid-piece clef changes.
  final clef = <int, Clef>{};
  for (final c in document.attributes.clefs) {
    clef[c.staff] = c;
  }

  for (final measure in document.measures) {
    measureStartMs.add((measureStartDiv * msPerDivision).round());
    measureKeyFifths.add(measure.keyFifths);
    for (final c in measure.clefs) {
      clef[c.staff] = c;
    }
    var measureSpan = divisionsPerMeasure > 0 ? divisionsPerMeasure : 0;
    for (final note in measure.notes) {
      final end = note.positionDivisions + note.durationDivisions;
      if (end > measureSpan) measureSpan = end;

      final startMs =
          (measureStartDiv + note.positionDivisions) * msPerDivision;
      final durationMs = note.durationDivisions * msPerDivision;

      final pitch = note.pitch;
      // Rests go to their own channel (render-only); pitchless non-rests are
      // skipped entirely.
      if (note.isRest || pitch == null) {
        if (note.isRest) {
          rests.add(
            TimedRest(
              startMs: startMs.round(),
              durationMs: durationMs.round(),
              staff: note.staff,
              noteType: note.noteType,
              dots: note.dots,
            ),
          );
          if (startMs + durationMs > songEndMs) {
            songEndMs = startMs + durationMs;
          }
        }
        continue;
      }

      final c = clef[note.staff];
      notes.add(
        TimedNote(
          pitch: midiOfPitch(pitch),
          startMs: startMs.round(),
          durationMs: durationMs.round(),
          staff: note.staff,
          beams: note.beams,
          clefSign: c?.sign ?? (note.staff >= 2 ? 'F' : 'G'),
          clefLine: c?.line ?? (note.staff >= 2 ? 4 : 2),
          noteType: note.noteType,
          dots: note.dots,
          diatonic: pitch.octave * 7 + (_diatonicOfStep[pitch.step] ?? 0),
          accidental: note.accidental,
          stemUp: switch (note.stem) {
            StemDir.up => true,
            StemDir.down => false,
            null => null,
          },
        ),
      );
      if (startMs + durationMs > songEndMs) songEndMs = startMs + durationMs;
    }
    measureStartDiv += measureSpan;
  }

  notes.sort((a, b) => a.startMs.compareTo(b.startMs));
  rests.sort((a, b) => a.startMs.compareTo(b.startMs));
  return DerivedPlayback(
    notes: notes,
    rests: rests,
    songEndMs: songEndMs,
    bpm: bpm,
    measureStartMs: measureStartMs,
    measureKeyFifths: measureKeyFifths,
  );
}

/// First `metronome` `per-minute` in the score, or [kDefaultBpm] if none.
int _tempoOf(ScoreDocument document) {
  for (final measure in document.measures) {
    for (final dir in measure.directions) {
      final kind = dir.kind;
      if (kind is DirectionKind_Metronome && kind.perMinute > 0) {
        return kind.perMinute;
      }
    }
  }
  return kDefaultBpm;
}
