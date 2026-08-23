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

  /// Engraved tie continuations flattened alongside [notes], on their own
  /// render-only channel like [rests]: each `tie stop` note whose duration was
  /// merged into its chain's first note (see [notationToTimedNotes]) is kept
  /// here, carrying [TimedNote.tieFromMs], so the staff still engraves the
  /// written note and its tie arc while the gate/scorer/waterfall see a single
  /// merged attack.
  final List<TimedNote> tieContinuations;

  final double songEndMs;
  final int bpm;

  /// Whether the source document classified as percussion (every non-rest
  /// note unpitched) — the player routes to the drum cascade + pad strip on
  /// it (change: add-drum-kit-view).
  final bool isPercussion;

  /// Start time (ms) of each measure, in document order; the first is 0. Lets the
  /// Partition cursor map a playhead position to a measure and a fraction within
  /// it.
  final List<int> measureStartMs;

  /// Key signature (fifths) in force during each measure, aligned with
  /// [measureStartMs]. Lets the scrolling staff show the armure at the playhead
  /// so a mid-piece modulation is reflected as you scroll past it.
  final List<int> measureKeyFifths;

  /// The written measure each played slot performs, aligned with
  /// [measureStartMs] — a repeated written measure appears once per pass.
  /// Empty means identity (no repeats): slot i plays written measure i.
  final List<int> writtenMeasureOf;

  /// Repeat notation to draw at each played slot of the scrolling staff
  /// (repeat barlines, volta label, `%`, segno/coda), aligned with
  /// [measureStartMs]. Render-only.
  final List<MeasureDecor> measureDecors;

  const DerivedPlayback({
    required this.notes,
    this.rests = const [],
    this.tieContinuations = const [],
    required this.songEndMs,
    required this.bpm,
    this.isPercussion = false,
    this.measureStartMs = const [],
    this.measureKeyFifths = const [],
    this.writtenMeasureOf = const [],
    this.measureDecors = const [],
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

/// The clef sign as the single-letter token the painters and
/// [TimedNote.clefSign] compare against (`'G'`/`'F'`/`'C'`), or
/// `'percussion'`. Keeps the render path `String`-typed while the bridged
/// model carries a typed sign.
String clefSignLetter(ClefSign sign) => switch (sign) {
  ClefSign.g => 'G',
  ClefSign.f => 'F',
  ClefSign.c => 'C',
  ClefSign.percussion => 'percussion',
};

/// Whether the score classifies as percussion: every non-rest note is
/// unpitched (and there is at least one). Mirrors the crate's
/// `instrument_of` — mixed content is NOT percussion, so a mixed score
/// admissible through today's gate keeps its existing playback.
bool _isPercussion(ScoreDocument document) {
  var pitched = false;
  var unpitched = false;
  for (final measure in document.measures) {
    for (final note in measure.notes) {
      if (note.isRest) continue;
      pitched |= note.pitch != null;
      unpitched |= note.unpitched != null;
    }
  }
  return unpitched && !pitched;
}

/// Converts a parsed score into visual playback notes.
///
/// Each non-rest note becomes a [TimedNote] whose start/duration come from its
/// running division position (accumulated across measures) scaled by
/// `ms_per_division = (60000 / bpm) / divisions`. Chord members share their
/// onset (they already carry the same `position_divisions`); rests are skipped.
/// The tempo is the first `metronome` `per-minute` found, else [kDefaultBpm].
///
/// **Tie chains are merged**: a `tie stop` continuation that abuts the note it
/// prolongs (same staff/voice/pitch) extends that note's duration instead of
/// becoming a playable note of its own. A tied note is a single attack, so the
/// Wait-Mode gate and the scorer must see a single note — emitting the
/// continuation as its own onset made the gate demand a fresh attack of a key
/// the score says to keep holding, deadlocking Wait Mode until the player
/// released and re-struck it (or turned Wait Mode off).
///
/// The merged continuations are **not discarded**: each is emitted on the
/// render-only [DerivedPlayback.tieContinuations] channel (with
/// [TimedNote.tieFromMs] pointing at the engraved note it prolongs), so the
/// staff view still shows the notation as written — dropping them left the
/// prolonged measures looking empty, as if the first note stretched over them.
DerivedPlayback notationToTimedNotes(
  ScoreDocument document, {
  bool unroll = true,
}) {
  final divisions = document.attributes.divisions < 1
      ? 1
      : document.attributes.divisions;
  // Unpitched notes are emitted — carrying their resolved General MIDI
  // percussion number in the MIDI slot — only when the score classifies as
  // percussion, mirroring the crate's schedule. A mixed score keeps today's
  // behaviour: its unpitched notes stay skipped.
  final percussion = _isPercussion(document);
  final bpm = _tempoOf(document);
  final msPerDivision = (60000.0 / bpm) / divisions;
  // Nominal audible duration of one grace note: an eighth of a quarter —
  // mirrors the Rust `grace_ms_of`, so the app, the back-office Play preview
  // and the server audio previews time ornaments identically.
  final graceMs = (60000.0 / bpm) / 8;

  final time = document.attributes.time;
  final beatType = time.beatType == 0 ? 4 : time.beatType;
  final divisionsPerMeasure = divisions * time.beats * 4 ~/ beatType;

  final notes = <TimedNote>[];
  final rests = <TimedRest>[];
  final tieContinuations = <TimedNote>[];
  final measureStartMs = <int>[];
  final measureKeyFifths = <int>[];
  final writtenMeasureOf = <int>[];
  final measureDecors = <MeasureDecor>[];
  var songEndMs = 0.0;
  var measureStartDiv = 0;

  // The playback order resolved by the engine at parse time (repeats per
  // pass, voltas selected, jumps followed). Fixtures may carry none, and a
  // selective practice run deliberately stays linear ([unroll] false): the
  // written order one-to-one then.
  final order = unroll && document.playOrder.isNotEmpty
      ? document.playOrder
      : [
          for (var i = 0; i < document.measures.length; i++)
            PlayedMeasure(writtenIndex: i, pass: 1),
        ];

  // Open tie chains, keyed by staff/voice/MIDI pitch: the index (into [notes])
  // of the chain's first note, where the chain currently ends (in absolute
  // divisions), and the onset (ms) of the chain's latest engraved note — the
  // arc anchor for the next continuation. Junctions are matched exactly in
  // divisions (integers accumulated across measures), so cross-barline ties
  // merge and anything that does not abut falls back to a normal playable note.
  final openTies = <String, ({int index, int endDiv, int lastStartMs})>{};

  // Running clef per staff, honouring mid-piece clef changes.
  final clef = <int, Clef>{};
  for (final c in document.attributes.clefs) {
    clef[c.staff] = c;
  }

  for (final slot in order) {
    if (slot.writtenIndex >= document.measures.length) continue;
    final written = document.measures[slot.writtenIndex];
    // A measure-repeat (`%`) slot replays its referenced measure's content.
    final measure =
        document.measures[(written.repeats.measureRepeatOf ?? slot.writtenIndex)
            .clamp(0, document.measures.length - 1)];
    measureStartMs.add((measureStartDiv * msPerDivision).round());
    measureKeyFifths.add(written.keyFifths);
    writtenMeasureOf.add(slot.writtenIndex);
    measureDecors.add(_decorOf(written.repeats));
    for (final c in written.clefs) {
      clef[c.staff] = c;
    }
    var measureSpan = divisionsPerMeasure > 0 ? divisionsPerMeasure : 0;
    final graceBack = _graceBackOffsets(measure.notes);
    for (var noteIndex = 0; noteIndex < measure.notes.length; noteIndex++) {
      final note = measure.notes[noteIndex];
      final end = note.positionDivisions + note.durationDivisions;
      if (end > measureSpan) measureSpan = end;

      final startDiv = measureStartDiv + note.positionDivisions;
      final endDiv = startDiv + note.durationDivisions;
      var startMs = startDiv * msPerDivision;
      var durationMs = note.durationDivisions * msPerDivision;
      // A grace note occupies no musical time (duration 0 at its principal's
      // position): give it the nominal duration, played just before the
      // principal, stacking consecutive graces backwards. Scheduling it
      // verbatim made a zero-length note glued onto the principal — inaudible,
      // invisible on the waterfall, and engraved on top of the principal.
      if (graceBack[noteIndex] > 0) {
        startMs = startMs - graceBack[noteIndex] * graceMs;
        if (startMs < 0) startMs = 0;
        durationMs = graceMs;
      }

      final pitch = note.pitch;
      // The unpitched channel sounds only for a percussion-classified score,
      // and only when its General MIDI number was resolved — an unresolvable
      // note is omitted, never fabricated.
      final gm = percussion ? note.unpitched?.gmNumber : null;
      // Rests go to their own channel (render-only); pitchless non-rests with
      // nothing to sound are skipped entirely.
      if (note.isRest || (pitch == null && gm == null)) {
        if (note.isRest) {
          rests.add(
            TimedRest(
              startMs: startMs.round(),
              durationMs: durationMs.round(),
              staff: note.staff,
              noteType: note.noteType,
              dots: note.dots,
              // The rest's voice rides along so the notation painters can
              // displace two-voice percussion rests per voice (change:
              // add-drum-notation-render).
              voice: note.voice,
            ),
          );
          if (startMs + durationMs > songEndMs) {
            songEndMs = startMs + durationMs;
          }
        }
        continue;
      }

      final midi = pitch != null ? midiOfPitch(pitch) : gm!;
      // Distinct key spaces: a GM number is not a MIDI pitch, so an unpitched
      // chain must never merge with a pitched one.
      final tieKey = pitch != null
          ? '${note.staff}/${note.voice}/$midi'
          : '${note.staff}/${note.voice}/u$midi';

      // An unpitched note's written position is a staff placement: its
      // diatonic index comes from the display step/octave, exactly as a
      // pitch's does from its step/octave.
      final diatonic = pitch != null
          ? pitch.octave * 7 + (_diatonicOfStep[pitch.step] ?? 0)
          : note.unpitched!.displayOctave * 7 +
                (_diatonicOfStep[note.unpitched!.displayStep] ?? 0);

      // The engraved note as a TimedNote — used both by the playable path and
      // by a tie continuation (which keeps its written figure but is routed to
      // the render-only channel with the arc anchor).
      TimedNote timed({int? tieFromMs}) {
        final c = clef[note.staff];
        return TimedNote(
          pitch: midi,
          startMs: startMs.round(),
          durationMs: durationMs.round(),
          staff: note.staff,
          voice: note.voice,
          beams: note.beams,
          clefSign: c != null
              ? clefSignLetter(c.sign)
              : (note.staff >= 2 ? 'F' : 'G'),
          clefLine: c?.line ?? (note.staff >= 2 ? 4 : 2),
          noteType: note.noteType,
          dots: note.dots,
          diatonic: diatonic,
          accidental: note.accidental,
          stemUp: switch (note.stem) {
            StemDir.up => true,
            StemDir.down => false,
            null => null,
          },
          tieFromMs: tieFromMs,
          isGrace: note.isGrace,
          isChord: note.isChord,
          // The engraved head class comes from the bridge (the shared crate
          // derived it beside the GM number) — never re-derived here.
          headClass: pitch != null ? null : note.unpitched!.headClass,
        );
      }

      if (note.tieStop) {
        final open = openTies.remove(tieKey);
        if (open != null && open.endDiv == startDiv) {
          // A continuation: extend the chain's first note to this note's end
          // (end-aligned in ms, so rounding never drifts across a long chain).
          final first = notes[open.index];
          final endMs = endDiv * msPerDivision;
          notes[open.index] = _withDuration(
            first,
            endMs.round() - first.startMs,
            // The first continuation marks where the written attack ends and
            // the tied sustain begins (kept across later links of the chain).
            sustainFromMs: first.sustainFromMs ?? startMs.round(),
          );
          // Keep the engraved figure for the notation painters, anchored to the
          // chain's previous engraved note so the tie arc can be drawn.
          tieContinuations.add(timed(tieFromMs: open.lastStartMs));
          // A stop that also starts is the middle of a chain: keep it open.
          if (note.tieStart) {
            openTies[tieKey] = (
              index: open.index,
              endDiv: endDiv,
              lastStartMs: startMs.round(),
            );
          }
          if (endMs > songEndMs) songEndMs = endMs;
          continue;
        }
        // A dangling stop (no abutting chain) stays a normal playable note.
      }

      notes.add(timed());
      if (note.tieStart) {
        openTies[tieKey] = (
          index: notes.length - 1,
          endDiv: endDiv,
          lastStartMs: startMs.round(),
        );
      } else {
        // A fresh un-tied attack of the same key closes any stale chain.
        openTies.remove(tieKey);
      }
      if (startMs + durationMs > songEndMs) songEndMs = startMs + durationMs;
    }
    measureStartDiv += measureSpan;
  }

  notes.sort((a, b) => a.startMs.compareTo(b.startMs));
  rests.sort((a, b) => a.startMs.compareTo(b.startMs));
  tieContinuations.sort((a, b) => a.startMs.compareTo(b.startMs));
  return DerivedPlayback(
    notes: notes,
    rests: rests,
    tieContinuations: tieContinuations,
    songEndMs: songEndMs,
    bpm: bpm,
    isPercussion: percussion,
    measureStartMs: measureStartMs,
    measureKeyFifths: measureKeyFifths,
    writtenMeasureOf: writtenMeasureOf,
    measureDecors: measureDecors,
  );
}

/// [n] with its duration replaced — how a tie chain's first note absorbs each
/// continuation while keeping its own onset, spelling and rhythmic figure.
TimedNote _withDuration(TimedNote n, int durationMs, {int? sustainFromMs}) =>
    TimedNote(
      pitch: n.pitch,
      startMs: n.startMs,
      durationMs: durationMs,
      staff: n.staff,
      voice: n.voice,
      beams: n.beams,
      clefSign: n.clefSign,
      clefLine: n.clefLine,
      noteType: n.noteType,
      dots: n.dots,
      diatonic: n.diatonic,
      accidental: n.accidental,
      stemUp: n.stemUp,
      tieFromMs: n.tieFromMs,
      isGrace: n.isGrace,
      isChord: n.isChord,
      sustainFromMs: sustainFromMs ?? n.sustainFromMs,
      headClass: n.headClass,
    );

/// For each note of a measure, how many grace slots *before its position* it
/// occupies: 0 for ordinary notes; a run of consecutive pitched graces sharing
/// staff/voice/position stacks backwards (the first of the run is furthest
/// back), so they play in document order and resolve onto the principal's
/// beat. Mirrors the Rust `grace_back_offsets` in `musicxml-core/playback.rs`.
/// The scrolling staff's repeat decorations for one written measure's marks.
MeasureDecor _decorOf(RepeatMarks r) {
  final hasAny =
      r.forward ||
      r.backwardTimes > 0 ||
      r.endingStart.isNotEmpty ||
      r.measureRepeatOf != null ||
      r.segno ||
      r.coda;
  if (!hasAny) return MeasureDecor.none;
  return MeasureDecor(
    repeatForward: r.forward,
    repeatBackward: r.backwardTimes > 0,
    voltaLabel: r.endingStart.isEmpty ? null : '${r.endingStart.join('.')}.',
    measureRepeat: r.measureRepeatOf != null,
    segno: r.segno,
    coda: r.coda,
  );
}

List<int> _graceBackOffsets(List<NoteEvent> notes) {
  final back = List<int>.filled(notes.length, 0);
  final run = <int>[];
  (int, int, int)? runKey;
  void flush() {
    for (var k = 0; k < run.length; k++) {
      back[run[k]] = run.length - k;
    }
    run.clear();
  }

  for (var i = 0; i < notes.length; i++) {
    final n = notes[i];
    final isGrace = n.isGrace && !n.isRest && n.pitch != null;
    if (isGrace) {
      final key = (n.staff, n.voice, n.positionDivisions);
      if (runKey != key) {
        flush();
        runKey = key;
      }
      run.add(i);
    } else {
      flush();
      runKey = null;
    }
  }
  flush();
  return back;
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
