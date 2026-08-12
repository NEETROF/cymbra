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

import 'package:freezed_annotation/freezed_annotation.dart';

import '../painters/keyboard_range.dart';
import '../src/rust/api/musicxml.dart' show BeamState;
import '../src/rust/api/score.dart';
import 'note_density_core.dart';

export '../painters/keyboard_range.dart'
    show KeyboardRangeMode, KeyboardRangeModeLabel;

part 'player_data.freezed.dart';

/// The score rendering modes: scrolling staff, Synthesia waterfall, and the
/// engraved Partition (sheet-music) view of a loaded MusicXML score.
enum RenderMode { staff, synthesia, partition }

/// Which hand(s) the player shows and awaits. Follows the engine's MusicXML
/// convention: staff 1 is the right hand, staff 2 (and above) the left hand.
enum Hand { left, right, both }

/// How much reading help the player wants while Wait Mode holds at an onset:
/// nothing, the awaited note's name, or its name plus the rhythmic figure.
/// Defaults to the note's name: someone who does not know the notes will not go
/// hunting for this setting, and the aid only ever appears once the gate has
/// already stopped play. The rhythm level and switching it off stay one tap
/// away in the play settings.
enum NoteReadingAid { off, name, nameAndRhythm }

/// Where a live note came from (change: add-audio-output-routing). Every source
/// converges on the player's note-on/note-off entry points, which would
/// otherwise discard the origin — but the instrument-sounds-itself rule needs
/// it: only notes played on the connected MIDI instrument were *already* sounded
/// by that instrument, so only those may be left unsynthesized. Scoring, key
/// feedback and Wait Mode never consult it.
enum NoteSource {
  /// A note read from the connected MIDI instrument's stream.
  midiDevice,

  /// A key pressed on the app's on-screen keyboard.
  onScreen,

  /// A note from the computer-keyboard assist (no instrument involved).
  computerKeyboard,
}

/// A score note with its time bounds in milliseconds (int), more convenient to
/// handle on the Dart side than the bridge's `BigInt`.
class TimedNote {
  final int pitch;
  final int startMs;
  final int durationMs;

  /// Staff the note belongs to (1 = treble/right hand, 2 = bass/left hand).
  /// Lets the Staff painter lay out a real grand staff.
  final int staff;

  /// Beam states carried from the parsed notation (begin/continue/end), so the
  /// Staff painter can beam eighth/sixteenth runs instead of drawing flags.
  final List<BeamState> beams;

  /// Clef in effect for this note's staff (sign + line), so the Staff painter
  /// positions it correctly through mid-piece clef changes (e.g. a left hand
  /// that starts in treble and moves to bass).
  final String clefSign;
  final int clefLine;

  /// Note-type token when known (e.g. `"whole"`, `"half"`, `"quarter"`,
  /// `"eighth"`), carried from the parsed notation so the Staff painter can pick
  /// an open vs filled notehead and drop the stem on whole notes — matching the
  /// engraved Partition view. Null for the demo score (the painter then infers
  /// the head from [durationMs]).
  final String? noteType;

  /// Number of augmentation dots (0 when none), so the Staff painter can draw
  /// dotted notes faithfully.
  final int dots;

  /// Written diatonic staff step (`octave*7 + step`, C=0…B=6) — the note's
  /// **line/space**, from its spelled step, not [pitch] (MIDI). Lets the Staff
  /// painter place e.g. an A♭ on the A line like the engraved Partition, instead
  /// of collapsing it onto G via the MIDI number. Null for the demo score and
  /// replay journal (MIDI-only), where the painter falls back to [pitch].
  final int? diatonic;

  /// Accidental token carried from the notation (`"sharp"`, `"flat"`,
  /// `"natural"`, …) when the score engraves one on this note, so the Staff
  /// painter draws it left of the head like the engraved Partition. Null when
  /// the note carries none (or for MIDI-only sources).
  final String? accidental;

  /// Stem direction carried from the notation (`true` = up, `false` = down), so
  /// the Staff painter beams and stems eighth runs the same way the Partition
  /// and the back office do. Null when the score leaves it implicit — the
  /// painter then derives it from the head's position on the staff.
  final bool? stemUp;

  const TimedNote({
    required this.pitch,
    required this.startMs,
    required this.durationMs,
    this.staff = 1,
    this.beams = const [],
    this.clefSign = 'G',
    this.clefLine = 2,
    this.noteType,
    this.dots = 0,
    this.diatonic,
    this.accidental,
    this.stemUp,
  });
}

/// A rest in the score, time-positioned like a [TimedNote] but carrying no
/// pitch. Kept in a channel **separate** from the playable notes so it feeds the
/// notation painters (Staff/Partition) without ever polluting the Wait-Mode gate
/// or the scoring — a rest is drawn, never awaited or judged.
class TimedRest {
  final int startMs;
  final int durationMs;

  /// Staff the rest belongs to (1 = treble/right hand, 2 = bass/left hand), so
  /// the Staff painter routes it to the right staff and the hand filter hides it
  /// with its hand.
  final int staff;

  /// Note-type token when known (e.g. `"whole"`, `"half"`, `"quarter"`), used to
  /// pick the rest glyph. Null → inferred from [durationMs].
  final String? noteType;

  /// Number of augmentation dots (0 when none).
  final int dots;

  const TimedRest({
    required this.startMs,
    required this.durationMs,
    this.staff = 1,
    this.noteType,
    this.dots = 0,
  });
}

/// Lead-in (ms) kept before the first note when trimming leading silence, so the
/// first note visibly approaches (falls in / scrolls in) instead of appearing
/// already on the hit line. A small fixed budget — on the order of the waterfall
/// fall-in — bounded and clamped to no earlier than time zero in
/// [effectiveStartMs]. Kept as a named constant so it is easy to tune.
const double kStartLeadInMs = 1000.0;

/// The playhead position a fresh run/transport should start at for [visibleNotes]:
/// a short [leadInMs] before the first note's onset, clamped to no earlier than
/// time zero, so leading rests / empty leading measures are skipped while the
/// first note still approaches. Returns `0` when there are no notes (nothing to
/// trim). Pure and host-testable; [visibleNotes] need not be sorted.
double effectiveStartMs(
  List<TimedNote> visibleNotes, {
  double leadInMs = kStartLeadInMs,
}) {
  if (visibleNotes.isEmpty) return 0;
  var firstOnset = visibleNotes.first.startMs;
  for (final n in visibleNotes) {
    if (n.startMs < firstOnset) firstOnset = n.startMs;
  }
  final start = firstOnset - leadInMs;
  return start < 0 ? 0 : start;
}

/// Slowest / fastest playback speed the transport is bounded by (0.25× … 2×).
const double kMinSpeed = 0.25;
const double kMaxSpeed = 2.0;

/// Normalizes a requested practice measure range to valid indices of a piece
/// with [measureCount] measures: each bound is clamped to `[0, measureCount-1]`
/// and the pair is reordered so `start ≤ end`. A single-measure range
/// (`start == end`) is valid. Returns null when the piece has no measure table
/// (e.g. the demo score), where a measure range is meaningless.
///
/// Pure and host-testable — the single normalization both range pickers (setup
/// steppers, tap-on-score) and the persistence load path go through.
({int start, int end})? normalizePracticeRange({
  required int start,
  required int end,
  required int measureCount,
}) {
  if (measureCount <= 0) return null;
  final last = measureCount - 1;
  final a = start.clamp(0, last);
  final b = end.clamp(0, last);
  return a <= b ? (start: a, end: b) : (start: b, end: a);
}

/// The playhead position a fresh run/transport should stop (finish or loop) at
/// for [visibleNotes]: the resolution of the last note — the largest
/// `startMs + durationMs` — so trailing rests / empty trailing measures after
/// the last note are skipped. Clamped to no later than [songEndMs] so a note
/// held past the raw end can never push it out, and falls back to [songEndMs]
/// when there are no notes (nothing to trim). Pure and host-testable;
/// [visibleNotes] need not be sorted.
double effectiveEndMs(
  List<TimedNote> visibleNotes, {
  required double songEndMs,
}) {
  if (visibleNotes.isEmpty) return songEndMs;
  var lastEnd = (visibleNotes.first.startMs + visibleNotes.first.durationMs)
      .toDouble();
  for (final n in visibleNotes) {
    final end = (n.startMs + n.durationMs).toDouble();
    if (end > lastEnd) lastEnd = end;
  }
  return lastEnd < songEndMs ? lastEnd : songEndMs;
}

/// Immutable player state (replaces the former `ChangeNotifier`).
///
/// Held by the `Player` Riverpod notifier; the UI watches it and mutates it via
/// `copyWith` only.
@freezed
abstract class PlayerData with _$PlayerData {
  const PlayerData._();

  const factory PlayerData({
    /// MIDI notes currently pressed (real MIDI keyboard + keyboard fallback).
    @Default(<int>{}) Set<int> activeNotes,

    /// Detected MIDI devices.
    @Default(<String>[]) List<String> midiPorts,

    /// Currently connected port (null if none).
    String? connectedDevice,

    /// The loaded demo score (null when a MusicXML partition is loaded instead).
    Score? score,

    /// Title of the piece currently loaded (null → the built-in demo).
    String? title,

    /// Tempo in BPM used to place staff bar-lines and for the tempo readout.
    @Default(80) int bpm,

    /// Key signature (fifths) of the loaded piece, for the staff armature.
    @Default(0) int keyFifths,

    /// Time signature of the loaded piece (beats / beat-type).
    @Default(4) int beats,
    @Default(4) int beatType,

    /// Score notes flattened and sorted by start.
    @Default(<TimedNote>[]) List<TimedNote> notes,

    /// Score rests flattened and sorted by start — a render-only channel for the
    /// notation painters. Deliberately separate from [notes] so rests are never
    /// awaited by the Wait-Mode gate nor scored.
    @Default(<TimedRest>[]) List<TimedRest> rests,

    /// End of the song (ms).
    @Default(0.0) double songEndMs,

    /// Start time (ms) of each measure, in order (Partition cursor placement).
    /// Empty for the demo score; populated from a parsed MusicXML document.
    @Default(<int>[]) List<int> measureStartMs,

    /// Key signature (fifths) in force during each measure, aligned with
    /// [measureStartMs] — so the scrolling staff shows the armure at the playhead
    /// and a mid-piece modulation is reflected. Empty for the demo score.
    @Default(<int>[]) List<int> measureKeyFifths,

    @Default(RenderMode.synthesia) RenderMode mode,
    @Default(true) bool waitMode,
    @Default(false) bool isPlaying,

    /// Remaining pre-start countdown in ms (0 = none). While > 0, playback is
    /// "armed" ([isPlaying] is true) but the playhead is frozen so the player has
    /// time to get ready; the screen shows a 5…1…GO countdown. Counts down in
    /// [advance] using real frame time, then playback proceeds normally.
    @Default(0.0) double countdownMs,

    /// Playback position (playhead), in milliseconds.
    @Default(0.0) double elapsedMs,

    /// The furthest the playhead has ever reached on the CURRENT score, in
    /// milliseconds (change: add-post-play-rating-prompt). Monotonic within a
    /// score: pausing, seeking back, looping, restarting, or switching hands never
    /// lowers it — only loading another score resets it. Feeds
    /// `playedNoteFraction`, which decides whether the player has heard enough of
    /// the piece to be asked to rate it on the way out.
    @Default(0.0) double furthestElapsedMs,

    /// Speed multiplier (1.0 = 100%).
    @Default(1.0) double speed,

    /// True when Wait Mode is currently blocking progression.
    @Default(false) bool blocked,

    /// Pitches already pressed for the onset the playhead is currently waiting
    /// at (Wait Mode). Latched on key-down so a note counts even once released —
    /// validation is by attack, not sustained hold. Reset when the gate advances.
    @Default(<int>{}) Set<int> gateSatisfied,

    /// Currently-held pitches whose *current* press has already satisfied an
    /// onset (Wait Mode). A held key counts for at most one onset: this set lets
    /// a sustained/tied note satisfy the onset it is carried into while a
    /// repeated pitch still requires a fresh attack. Cleared for a pitch on a new
    /// note-on (fresh attack) and on note-off; wholesale on gate re-arm.
    @Default(<int>{}) Set<int> consumedHeld,

    /// On-screen keyboard range mode. Defaults to auto-fit (sized to the loaded
    /// piece); the user can pin a fixed controller size from the chooser. Seeded
    /// from (and written back to) the persisted play preferences.
    @Default(KeyboardRangeMode.auto) KeyboardRangeMode keyboardRange,

    /// Whether the on-screen keyboard is shown. Only honoured in the notation
    /// modes (Staff/Partition), where hiding it hands the freed height to the
    /// score; Synthesia always shows the keyboard because its cascade aligns to
    /// it. Session-only, defaults to visible.
    @Default(true) bool keyboardVisible,

    /// Which hand(s) the player shows and awaits. Session-only (resets to
    /// [Hand.both] on launch); drives [showsStaff]/[visibleNotes] so every mode
    /// and the gate filter out the unselected hand together.
    @Default(Hand.both) Hand selectedHands,

    /// Whether the metronome is enabled. A single app-wide preference (kept across
    /// pause and across score changes) toggled from the header Tempo chip. When on
    /// *and* [isPlaying], [advance] sounds a click and pulses the chip on each beat
    /// of the measure; while paused it stays on but is silent.
    @Default(false) bool metronomeEnabled,

    /// Monotonic count of metronome beats fired so far — the visual pulse signal
    /// the Tempo chip watches to animate one pulse per beat. Paired with
    /// [lastBeatAccent] so the chip can pulse harder on the downbeat.
    @Default(0) int beatCount,

    /// Whether the most recent beat ([beatCount]) was an accented downbeat.
    @Default(false) bool lastBeatAccent,

    /// How much reading help to show while Wait Mode holds at an onset. Seeded
    /// from the persisted play preferences (like [metronomeEnabled]) and changed
    /// through the setup modal / in-game settings.
    @Default(NoteReadingAid.name) NoteReadingAid readingAid,

    /// Whether the connected MIDI instrument produces its own sound, so the app
    /// must not duplicate it (change: add-audio-output-routing). Seeded from the
    /// persisted play preferences. Suppresses **only** the synthesis of notes
    /// whose source is [NoteSource.midiDevice]: the on-screen and computer
    /// keyboards keep sounding, and score playback, metronome clicks and preview
    /// clips are untouched. Scoring, key feedback and Wait Mode are identical
    /// either way.
    @Default(false) bool instrumentSoundsItself,

    /// Output latency compensation in milliseconds (change:
    /// add-audio-output-routing), seeded from the persisted play preferences.
    /// The audio the user hears at any instant is what the engine emitted this
    /// many milliseconds ago, so [referenceMs] — the position they are actually
    /// hearing — is what the playhead is drawn at and what attacks are judged
    /// against. 0 (the default) makes it a no-op.
    @Default(0) int outputOffsetMs,

    /// First measure of the **active practice range** (index into
    /// [measureStartMs]), or null for the whole piece (change: add-measure-range-
    /// practice, D1). Paired with [practiceEndMeasure]: both set and narrower
    /// than the whole piece ⇒ [isSelectiveRun].
    int? practiceStartMeasure,

    /// Last measure (inclusive) of the active practice range, or null for the
    /// whole piece.
    int? practiceEndMeasure,
  }) = _PlayerData;

  bool get midiConnected => connectedDevice != null;

  /// Whether the app should synthesize a live note coming from [source]
  /// (change: add-audio-output-routing). The single predicate behind the
  /// instrument-sounds-itself rule: everything else the note triggers — scoring,
  /// key feedback, the Wait Mode gate — runs regardless of what this returns.
  bool synthesizes(NoteSource source) =>
      !(instrumentSoundsItself && source == NoteSource.midiDevice);

  /// The score position the player is **hearing** right now — the playhead
  /// shifted back by [outputOffsetMs] (change: add-audio-output-routing).
  ///
  /// [elapsedMs] is the emission clock: it is what decides when a note is handed
  /// to the audio engine. On a delayed route that sound only reaches the ear
  /// [outputOffsetMs] later, so this is the position the highlight must show and
  /// the reference an attack must be judged against — one number, so the two can
  /// never drift apart. With the default offset of 0 it *is* [elapsedMs].
  double get referenceMs => elapsedMs - outputOffsetMs;

  /// Whether the instrument-sounds-itself setting can do anything right now: it
  /// only ever suppresses notes arriving from an instrument, so with no MIDI
  /// port connected it is offered disabled rather than silently inert.
  bool get instrumentSoundsItselfAvailable => midiConnected;

  /// Whether notes on [staff] are shown for the current [selectedHands] — the
  /// single visibility predicate shared by the painters and the gate. Staff 1
  /// is the right hand, staff 2+ the left hand.
  bool showsStaff(int staff) => switch (selectedHands) {
    Hand.both => true,
    Hand.right => staff == 1,
    Hand.left => staff >= 2,
  };

  /// Notes belonging to the selected hand(s) — the input every render mode and
  /// the gate derive from, so display and Wait Mode stay consistent.
  ///
  /// During a **selective run** this is the passage's notes only. The run cannot
  /// reach the rest of the piece, so showing it is misleading: the player cannot
  /// see where the passage ends, and the Wait-Mode gate would otherwise hold at
  /// an onset outside the loop. Restricting at this single source keeps the
  /// display, the gate and the score audio in agreement by construction.
  List<TimedNote> get visibleNotes => notes
      .where((n) => showsStaff(n.staff) && _withinRun(n.startMs.toDouble()))
      .toList();

  /// Rests belonging to the selected hand(s) — the render-only companion to
  /// [visibleNotes], so the Staff painter hides a muted hand's rests with its
  /// notes (and, in a selective run, everything outside the passage).
  List<TimedRest> get visibleRests => rests
      .where((r) => showsStaff(r.staff) && _withinRun(r.startMs.toDouble()))
      .toList();

  /// Whether onset [t] falls inside the run. Always true for a full run; for a
  /// selective one, the half-open span of the chosen measures. Deliberately keyed
  /// on the ONSET: a note is in the passage if it starts there.
  bool _withinRun(double t) {
    if (!isSelectiveRun) return true;
    final from = measureStartMs[practiceStartMeasure!].toDouble();
    return t >= from && t < measureEndMs(practiceEndMeasure!);
  }

  /// Number of engraved measures with known timing (0 for the demo score, which
  /// carries no measure table).
  int get measureCount => measureStartMs.length;

  /// Index of the piece's last measure, or null when there is no measure table.
  int? get lastMeasureIndex => measureStartMs.isEmpty ? null : measureCount - 1;

  /// End (ms) of measure [i] — the next measure's start, or [songEndMs] for the
  /// last one. The symmetric companion of `measureStartMs[i]`, so a measure range
  /// `[a, b]` maps to `measureStartMs[a] … measureEndMs(b)`. Pure and
  /// host-testable; returns [songEndMs] for an out-of-range index.
  double measureEndMs(int i) => (i + 1 >= 0 && i + 1 < measureStartMs.length)
      ? measureStartMs[i + 1].toDouble()
      : songEndMs;

  /// Whether a practice range is set at all (both bounds, on a piece that has a
  /// measure table). A range covering the whole piece is set but **not**
  /// selective — it is a full run.
  bool get hasPracticeRange =>
      practiceStartMeasure != null &&
      practiceEndMeasure != null &&
      measureStartMs.isNotEmpty;

  /// Whether the run is a **selective (practice) run** — its active measure range
  /// is narrower than the whole piece (change: add-measure-range-practice, D1/D2).
  /// A selective run is never scored: it plays (and can loop) just its measures.
  bool get isSelectiveRun =>
      hasPracticeRange &&
      !(practiceStartMeasure == 0 && practiceEndMeasure == lastMeasureIndex);

  /// Effective start of the current selection — where a fresh run/transport
  /// places the playhead. For a selective run, the first measure of the active
  /// range; otherwise a short lead-in before the first visible note's onset (see
  /// [effectiveStartMs]), so leading rests / empty measures are trimmed. `0` when
  /// the selection has no notes or already starts near the beginning.
  double get startMs => isSelectiveRun
      ? measureStartMs[practiceStartMeasure!].toDouble()
      : effectiveStartMs(visibleNotes);

  /// Effective end of the current selection — where a fresh run finishes (scored),
  /// loops, or stops. For a selective run, the end of the range's last measure;
  /// otherwise the last visible note's resolution (see [effectiveEndMs]), so
  /// trailing rests / empty measures are trimmed. Falls back to [songEndMs] when
  /// the selection has no notes.
  double get endMs => isSelectiveRun
      ? measureEndMs(practiceEndMeasure!)
      : effectiveEndMs(visibleNotes, songEndMs: songEndMs);

  /// The loaded piece's characteristic tightest note spacing (see
  /// [cachedOnsetGapMs]), which the scrolling Portée caps its look-ahead window
  /// against so a dense score is not engraved tighter than it can be read.
  /// Measured over **all** the notes, not [visibleNotes]: the engraving scale is
  /// a property of the piece, and muting a hand must not rescale it. `null` when
  /// there is nothing to measure.
  double? get onsetGapMs => cachedOnsetGapMs(notes);

  /// The loaded piece's typical measure duration (see [cachedMedianMeasureMs]),
  /// which the Portée caps its look-ahead against so no more than
  /// [kMaxVisibleMeasures] measures are ever in front of the reader. `null` for
  /// the demo score, which carries no measure table.
  double? get measureMs =>
      cachedMedianMeasureMs(measureStartMs, songEndMs: songEndMs);

  /// Whether the loaded piece has any left-hand (staff 2+) notes, so isolating a
  /// hand is meaningful. The hand selector is shown only then — a single-staff
  /// piece offers nothing to separate (and the [Hand.both] default is harmless).
  bool get hasMultipleStaves => notes.any((n) => n.staff >= 2);

  /// Inclusive (low, high) MIDI pitches the on-screen keyboard should show for
  /// the current [keyboardRange] and loaded [notes]. Feeds the shared
  /// `PianoLayout` so the keyboard and waterfall stay aligned.
  ({int low, int high}) get keyboardBounds =>
      computeKeyboardRange(keyboardRange, [for (final n in notes) n.pitch]);

  /// The fixed-size window the current [keyboardRange] represents (exactly N
  /// keys), or null in auto mode. Keys in [keyboardBounds] outside this are the
  /// extra keys drawn to avoid clipping notes; the keyboard marks that boundary.
  ({int low, int high})? get keyboardChosenWindow =>
      chosenSizeWindow(keyboardRange, [for (final n in notes) n.pitch]);

  /// Notes that should be held at instant [t] (playhead within the window
  /// [start, start+duration]). Acts as the "gate" for Wait Mode. Restricted to
  /// [visibleNotes] so a hidden hand is neither awaited nor shown as expected.
  Set<int> requiredNotesAt(double t) {
    final result = <int>{};
    for (final n in visibleNotes) {
      if (n.startMs <= t + 1 && t < n.startMs + n.durationMs) {
        result.add(n.pitch);
      }
    }
    return result;
  }

  /// Pitches of notes whose onset is at instant [t] (their start coincides with
  /// the playhead, within a 1ms tolerance). This is the Wait Mode gate set: the
  /// notes that must be *attacked* here, regardless of their duration.
  /// Restricted to [visibleNotes] so the hidden hand never freezes the cascade.
  Set<int> onsetPitchesAt(double t) {
    final result = <int>{};
    for (final n in visibleNotes) {
      if ((n.startMs - t).abs() <= 1.0) result.add(n.pitch);
    }
    return result;
  }

  /// The next note onset strictly after [t] (ms), or null if there are none.
  /// Restricted to [visibleNotes] so the playhead does not pause at a hidden
  /// hand's onset in Wait Mode.
  double? nextOnsetAfter(double t) {
    double? best;
    for (final n in visibleNotes) {
      if (n.startMs > t + 1 && (best == null || n.startMs < best)) {
        best = n.startMs.toDouble();
      }
    }
    return best;
  }

  /// The instant the "expected" set refers to: in Wait Mode the onset the
  /// playhead sits on, or — while travelling between onsets — the upcoming one,
  /// so the preview shows the next note to play; outside Wait Mode the playhead
  /// itself. Null when Wait Mode has no onset left to point at.
  ///
  /// The single source of that instant: [expectedKeys], [expectedKeysForHand]
  /// and [expectedNotes] all resolve it here, so the keyboard highlight, the
  /// gate and the reading aid can never disagree about which notes are expected.
  double? get expectedTimeMs {
    if (!waitMode) return elapsedMs;
    if (onsetPitchesAt(elapsedMs).isNotEmpty) return elapsedMs;
    return nextOnsetAfter(elapsedMs);
  }

  /// Whether [n] belongs to the expected set at instant [t]: its *attack* lands
  /// there in Wait Mode (the gate validates by onset), else it is sounding under
  /// the playhead.
  bool _isExpectedAt(TimedNote n, double t) => waitMode
      ? (n.startMs - t).abs() <= 1.0
      : (n.startMs <= t + 1 && t < n.startMs + n.durationMs);

  /// Keys to highlight as "expected" on the keyboard.
  Set<int> get expectedKeys {
    final t = expectedTimeMs;
    if (t == null) return const {};
    return waitMode ? onsetPitchesAt(t) : requiredNotesAt(t);
  }

  /// The expected notes themselves, rather than just their pitches — the reading
  /// aid needs each note's written spelling and rhythmic figure, which a
  /// `Set<int>` of MIDI pitches throws away. Restricted to [visibleNotes], so a
  /// muted hand is never named.
  List<TimedNote> get expectedNotes {
    final t = expectedTimeMs;
    if (t == null) return const [];
    return [
      for (final n in visibleNotes)
        if (_isExpectedAt(n, t)) n,
    ];
  }

  /// The subset of [expectedKeys] belonging to one hand (staff 1 = right, staff
  /// 2+ = left), so the keyboard can colour expected keys per hand.
  Set<int> expectedKeysForHand({required bool rightHand}) {
    final t = expectedTimeMs;
    if (t == null) return const {};
    final result = <int>{};
    for (final n in visibleNotes) {
      final isRight = n.staff == 1;
      if (rightHand != isRight) continue;
      if (_isExpectedAt(n, t)) result.add(n.pitch);
    }
    return result;
  }

  /// Duration (ms) of one beat at playhead [t]: the measure's span divided by
  /// [beats] when the measure table is known, else derived from [bpm]. Mirrors
  /// how [metronomeBeatsCrossed] derives its ticks, so the reading aid's beat
  /// count and the metronome agree on what a beat is. Returns 0 when neither is
  /// known.
  double beatDurationMsAt(double t) {
    final starts = measureStartMs;
    final m = measureAt(t);
    if (m != null && beats > 0) {
      final start = starts[m.index].toDouble();
      final end = (m.index + 1 < starts.length
          ? starts[m.index + 1].toDouble()
          : songEndMs);
      if (end > start) return (end - start) / beats;
    }
    return bpm > 0 ? 60000.0 / bpm : 0;
  }

  /// The measure containing playhead [t] and the fraction (0..1) elapsed within
  /// it, or null when no timing is known (e.g. the demo score) or [t] is outside
  /// the piece. Drives the Partition playhead cursor.
  ({int index, double fraction})? measureAt(double t) {
    final starts = measureStartMs;
    if (starts.isEmpty || t < starts.first) return null;
    for (var i = 0; i < starts.length; i++) {
      final start = starts[i];
      final end = (i + 1 < starts.length ? starts[i + 1] : songEndMs)
          .toDouble();
      if (t >= start && t < end) {
        final span = end - start;
        final frac = span > 0 ? ((t - start) / span).clamp(0.0, 1.0) : 0.0;
        return (index: i, fraction: frac);
      }
    }
    return null;
  }

  /// Expected notes at instant [t] for one hand: staff 1 is the right hand,
  /// staff 2+ the left hand. Same window as [requiredNotesAt], split by staff,
  /// so the assist keys play exactly the hand's due notes.
  Set<int> expectedNotesForHand(double t, {required bool rightHand}) {
    final result = <int>{};
    for (final n in visibleNotes) {
      final isRight = n.staff == 1;
      if (rightHand != isRight) continue;
      if (n.startMs <= t + 1 && t < n.startMs + n.durationMs) {
        result.add(n.pitch);
      }
    }
    return result;
  }
}

/// The note edges crossed by the playhead moving from [from] to [to] (ms),
/// used to drive score audio: which pitches to **start** (an onset entered the
/// half-open span `[from, to)`) and which currently-[sounding] pitches to
/// **stop** (no visible note still covers the new playhead [to]).
///
/// Pure and host-testable. Pass [visible] (the selected hand's notes) so a
/// hidden hand is neither sounded nor stopped here. The span is half-open so an
/// onset sounds exactly once and only **after** the playhead advances past it —
/// a frozen Wait Mode onset (`from == to`) yields no starts.
({List<int> starts, List<int> stops}) scoreNoteEdges({
  required List<TimedNote> visible,
  required double from,
  required double to,
  required Set<int> sounding,
}) {
  final starts = <int>[];
  if (to > from) {
    for (final n in visible) {
      if (from <= n.startMs && n.startMs < to) starts.add(n.pitch);
    }
  }
  final stops = <int>[];
  for (final p in sounding) {
    final covered = visible.any(
      (n) => n.pitch == p && n.startMs <= to && to < n.startMs + n.durationMs,
    );
    if (!covered) stops.add(p);
  }
  return (starts: starts, stops: stops);
}

/// One metronome beat: the playhead time (ms) it falls on and whether it is the
/// accented downbeat (the first beat of its measure).
typedef MetronomeBeat = ({double timeMs, bool accent});

/// The metronome beats crossed by the playhead moving from [from] to [to] (ms),
/// in order — one tick per beat of the measure, so the number of ticks per
/// measure follows the time signature ([beats]).
///
/// Pure and host-testable. Beats are derived from the score's own timing so they
/// stay in sync with the notes:
/// - When [measureStartMs] is known, each measure `[start, end)` is divided into
///   [beats] equal beats; the beat on a measure start is the accented downbeat.
///   This keeps ticks aligned to the engraved measures.
/// - For the demo score (no [measureStartMs]) it falls back to a steady beat from
///   [bpm], accenting every [beats]-th beat from the origin.
///
/// The span is half-open `[from, to)` so each beat fires exactly once and a frozen
/// playhead (`from == to`) yields none. Callers skip this across a loop/seek seam.
List<MetronomeBeat> metronomeBeatsCrossed({
  required List<int> measureStartMs,
  required int beats,
  required int bpm,
  required double songEndMs,
  required double from,
  required double to,
}) {
  if (to <= from || beats < 1) return const [];

  final result = <MetronomeBeat>[];
  if (measureStartMs.isNotEmpty) {
    for (var i = 0; i < measureStartMs.length; i++) {
      final start = measureStartMs[i].toDouble();
      final end = (i + 1 < measureStartMs.length
          ? measureStartMs[i + 1].toDouble()
          : songEndMs);
      if (end <= start) continue;
      final beatDur = (end - start) / beats;
      for (var b = 0; b < beats; b++) {
        final t = start + b * beatDur;
        if (from <= t && t < to) result.add((timeMs: t, accent: b == 0));
      }
    }
    return result;
  }

  // Fallback: no measure table — derive a steady beat straight from the tempo.
  if (bpm <= 0) return const [];
  final beatDur = 60000.0 / bpm;
  // First beat index at/after [from], and the last strictly before [to].
  final firstIndex = (from / beatDur).ceil();
  for (var g = firstIndex; ; g++) {
    final t = g * beatDur;
    if (t >= to) break;
    if (t >= from) result.add((timeMs: t, accent: g % beats == 0));
  }
  return result;
}

/// A pitch near [expected] (within ±[spread] semitones) that is **not** in
/// [avoid] and lies within `[lowBound, highBound]` — a deliberate near-miss that
/// never matches an expected note, so it cannot satisfy the Wait Mode gate.
///
/// [nextRandom] is called with an exclusive upper bound to choose among the
/// candidates; injecting it keeps the pick deterministic in tests. When no
/// candidate exists within [spread], the nearest in-range non-avoided pitch is
/// returned; if even that is impossible, [expected] is returned unchanged.
int nearMissPitch(
  int expected, {
  required int lowBound,
  required int highBound,
  required Set<int> avoid,
  required int Function(int) nextRandom,
  int spread = 3,
}) {
  final candidates = <int>[];
  for (var d = 1; d <= spread; d++) {
    for (final p in [expected - d, expected + d]) {
      if (p >= lowBound && p <= highBound && !avoid.contains(p)) {
        candidates.add(p);
      }
    }
  }
  if (candidates.isNotEmpty) {
    return candidates[nextRandom(candidates.length)];
  }
  // Fallback: nearest in-range pitch that is not avoided.
  for (var d = 1; d <= highBound - lowBound; d++) {
    for (final p in [expected - d, expected + d]) {
      if (p >= lowBound && p <= highBound && !avoid.contains(p)) return p;
    }
  }
  return expected;
}
