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
import '../src/rust/api/musicxml.dart' show BeamState, HeadClass;
import '../src/rust/api/score.dart';
import 'drum_kit.dart';
import 'note_density_core.dart';
import 'performance_scoring_core.dart' show ScoreClocks, judgmentClock;

export '../painters/keyboard_range.dart'
    show KeyboardRangeMode, KeyboardRangeModeLabel;

part 'player_data.freezed.dart';

/// The score rendering modes: scrolling staff, Synthesia waterfall, and the
/// engraved Partition (sheet-music) view of a loaded MusicXML score.
/// The play surfaces. `stage` is the perspective reading of the cascade and is
/// offered for PERCUSSION ONLY (experiment: drum-highway): a keyboard score has
/// 88 lanes, which no vanishing point survives.
enum RenderMode { staff, synthesia, partition, stage }

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

  /// Voice the note belongs to (change: add-drum-kit-view). A drum part is
  /// written on a single staff in two voices — hands with stems up (voice 1),
  /// feet with stems down (voice 2) — so the hands/feet split keys on this,
  /// never on staff and never on the stem-direction proxy next to it.
  final int voice;

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

  /// Set only on a **render-only tie continuation** (an engraved `tie stop` note
  /// whose duration was merged into its chain's first note for playback): the
  /// start (ms) of the engraved note this one prolongs, so the Staff painter can
  /// draw the tie arc between the two heads. Null on every playable note.
  final int? tieFromMs;

  /// True for a grace note (ornamental small note, `<grace/>`): played with a
  /// short nominal duration just before its principal, and engraved smaller by
  /// the Staff painter.
  final bool isGrace;

  /// True for a chord member (`<chord/>`): it sounds with the preceding
  /// principal note and shares its stem — the Staff painter draws its head but
  /// never a stem or flag of its own, matching the engraved Partition.
  final bool isChord;

  /// Set only on a **merged tie chain's first note**: the start (ms) of the
  /// chain's first continuation — where the written attack ends and the tied
  /// sustain begins. The waterfall renders the bar's attack segment (up to
  /// here) full-strength and the sustain tail slimmer/quieter, so "strike now"
  /// and "keep holding" read differently. Null on ordinary notes.
  final int? sustainFromMs;

  /// Engraved head class of an **unpitched** (percussion) note, carried
  /// verbatim from the bridged `Unpitched.headClass` (change:
  /// add-drum-notation-render): the shared crate classifies cymbals as x
  /// heads (the open hi-hat additionally marked), drums as ovals — the
  /// painters consume it and never re-derive GM ranges of their own. Null for
  /// pitched notes and MIDI-only sources.
  final HeadClass? headClass;

  const TimedNote({
    required this.pitch,
    required this.startMs,
    required this.durationMs,
    this.staff = 1,
    this.voice = 1,
    this.beams = const [],
    this.clefSign = 'G',
    this.clefLine = 2,
    this.noteType,
    this.dots = 0,
    this.diatonic,
    this.accidental,
    this.stemUp,
    this.tieFromMs,
    this.isGrace = false,
    this.isChord = false,
    this.sustainFromMs,
    this.headClass,
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

  /// Voice the rest belongs to (change: add-drum-notation-render). On a
  /// two-voice percussion measure the painters displace rests by voice —
  /// voice 1 above the middle line, voice 2 below — so a rest never sits on
  /// the midline where the other voice's material runs.
  final int voice;

  const TimedRest({
    required this.startMs,
    required this.durationMs,
    this.staff = 1,
    this.noteType,
    this.dots = 0,
    this.voice = 1,
  });
}

/// Repeat notation to draw at one played slot of the scrolling staff, aligned
/// with [PlayerData.measureStartMs]. Render-only — playback already follows
/// the unrolled order; these are the glyphs that tell the reader why.
class MeasureDecor {
  /// A forward repeat (`‖:`) opens this slot's written measure.
  final bool repeatForward;

  /// A backward repeat (`:‖`) closes it.
  final bool repeatBackward;

  /// Volta label ("1." / "1.2.") when an ending bracket starts here.
  final String? voltaLabel;

  /// The written measure is a measure-repeat (`%`) sign.
  final bool measureRepeat;

  /// Segno / coda signs placed at this measure.
  final bool segno;
  final bool coda;

  const MeasureDecor({
    this.repeatForward = false,
    this.repeatBackward = false,
    this.voltaLabel,
    this.measureRepeat = false,
    this.segno = false,
    this.coda = false,
  });

  static const none = MeasureDecor();

  /// Whether anything is drawn for this slot at all.
  bool get isNone =>
      !repeatForward &&
      !repeatBackward &&
      voltaLabel == null &&
      !measureRepeat &&
      !segno &&
      !coda;
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

/// Song-time slack (ms) under which a rewind tap means the PREVIOUS measure:
/// within this of a measure's start the player is "at" that start, so going
/// back means the bar before; further in, the tap means "again from the top of
/// this bar" — the audio-player back-button convention.
const double kRewindEpsilonMs = 300.0;

/// The playhead position (ms) one transport measure-rewind lands on from
/// [elapsedMs]: the start of the measure containing the playhead when it is
/// more than [epsilonMs] past that start, else the start of the previous
/// measure — so repeated taps stack back one measure at a time. Clamped to no
/// earlier than [minMs] (the run's effective start: the active range's first
/// measure on a selective run, else the piece's trimmed start). Returns null
/// when [measureStartMs] is empty (the demo score carries no measure table).
/// Pure and host-testable.
double? rewindTargetMs({
  required double elapsedMs,
  required List<int> measureStartMs,
  required double minMs,
  double epsilonMs = kRewindEpsilonMs,
}) {
  if (measureStartMs.isEmpty) return null;
  var i = measureStartMs.length - 1;
  while (i > 0 && elapsedMs < measureStartMs[i]) {
    i--;
  }
  final start = measureStartMs[i].toDouble();
  final target = elapsedMs > start + epsilonMs
      ? start
      : (i > 0 ? measureStartMs[i - 1].toDouble() : start);
  return target < minMs ? minMs : target;
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

    /// Engraved tie continuations, sorted by start — a render-only channel like
    /// [rests]. Each is a `tie stop` note whose duration was merged into its
    /// chain's first note in [notes] (a tie is a single attack), kept here so
    /// the Staff painter still engraves the written note and its tie arc.
    /// Deliberately separate from [notes] so continuations are never awaited by
    /// the Wait-Mode gate nor scored.
    @Default(<TimedNote>[]) List<TimedNote> tieContinuations,

    /// End of the song (ms).
    @Default(0.0) double songEndMs,

    /// The last UNSPENT stroke per surface, stamped on the **playhead's**
    /// clock (not the wall clock [struckSurfacesMs] uses for its flash): the
    /// early-stroke tolerance is a musical window, so it has to be measured
    /// where the onset lives. An entry is removed the moment its stroke is
    /// credited to an onset, which is what stops one stroke from validating
    /// two. Percussion only; cleared with [struckSurfacesMs].
    ///
    /// Because the stamp is on the playhead, it is only meaningful **on the
    /// run that made it**: every transport reset that re-arms [gateSatisfied]
    /// drops these too, and a stamp that ends up ahead of the playhead anyway
    /// is discarded rather than read as an enormously early stroke.
    @Default(<int, double>{}) Map<int, double> strokeAtMs,

    /// Start time (ms) of each measure, in order (Partition cursor placement).
    /// Empty for the demo score; populated from a parsed MusicXML document.
    @Default(<int>[]) List<int> measureStartMs,

    /// Key signature (fifths) in force during each measure, aligned with
    /// [measureStartMs] — so the scrolling staff shows the armure at the playhead
    /// and a mid-piece modulation is reflected. Empty for the demo score.
    @Default(<int>[]) List<int> measureKeyFifths,

    /// The written measure each played slot performs, aligned with
    /// [measureStartMs] — a repeated written measure appears once per pass.
    /// Empty means identity (no repeats or linear practice run).
    @Default(<int>[]) List<int> writtenMeasureOf,

    /// Repeat notation to draw per played slot on the scrolling staff,
    /// aligned with [measureStartMs]. Render-only.
    @Default(<MeasureDecor>[]) List<MeasureDecor> measureDecors,

    /// Number of **written** measures of the loaded piece (0 for the demo).
    /// The practice range and Partition taps live in written measures; with
    /// repeats the played tables above are longer than this.
    @Default(0) int writtenMeasureCount,

    @Default(RenderMode.synthesia) RenderMode mode,
    @Default(true) bool waitMode,
    @Default(false) bool isPlaying,

    /// Remaining pre-start countdown in ms (0 = none). While > 0, playback is
    /// "armed" ([isPlaying] is true) but the playhead is frozen so the player has
    /// time to get ready; the screen shows a 3…2…1…GO countdown. Counts down on
    /// **real** frame time — never scaled by [speed], so a slow-tempo practice
    /// run does not stretch the wait — then playback proceeds normally.
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
    ///
    /// **Keyboard scores only** (change: add-practice-focus-controls). A drum
    /// part is written on one staff, so the staff mapping never applied to it;
    /// the hands/feet reading that stood in for it split a groove at the wrong
    /// grain — it classified the kick as a foot event, so "hands only" removed
    /// the one piece a drummer never stops playing. Percussion isolates through
    /// [mutedDrumPieces] instead, and leaves this at [Hand.both].
    @Default(Hand.both) Hand selectedHands,

    /// The kit pieces the session does **not** ask for (change:
    /// add-practice-focus-controls) — identified by [drumPieceIdOf], so muting
    /// the hi-hat takes its closed and open numbers together.
    ///
    /// Stored as the complement of the focus set ([focusedDrumPieces]) so that
    /// "everything in focus" is the empty default, and so a number the kit
    /// model never enumerated reads as in focus rather than being silently
    /// filtered out.
    ///
    /// Session-only, like the hand selection it replaces on percussion: it
    /// describes the passage being worked on, not a preference, and persisting
    /// it would hand a later score a kit with holes in it. Reset on load.
    @Default(<String>{}) Set<String> mutedDrumPieces,

    /// The loaded score is percussion (change: add-drum-kit-view): the player
    /// renders the drum cascade + pad strip and no keyboard-range apparatus.
    /// The notation modes are offered alongside the cascade (change:
    /// add-drum-notation-render), and Wait Mode + scoring since the matcher
    /// exists (change: add-drum-scoring).
    @Default(false) bool isPercussion,

    /// The ordered lane layout derived ONCE from the loaded percussion score
    /// (one lane per kit piece present, kick excluded by construction) —
    /// consumed by BOTH the cascade and the pad strip via
    /// [presentedDrumLanes], never derived twice.
    @Default(<DrumLane>[]) List<DrumLane> drumLanes,

    /// The inverted-kit setting (change: add-drum-kit-view): reverses the
    /// PRESENTED lane order and the pad strip together — never the notation,
    /// never how incoming notes are interpreted. Persisted; defaults to the
    /// standard layout and is never inferred.
    @Default(false) bool invertedKit,

    /// When each controller surface was last struck — wall-clock milliseconds,
    /// keyed by PRESENTED surface (a pad by its lane index, the kick pedal
    /// under [kPedalSurface]); change: add-drum-input-mapping.
    ///
    /// The one honest feedback state percussion has: **struck**. It claims
    /// nothing about correctness — there is no matcher until
    /// `add-drum-scoring` — and it decays on its own short duration rather
    /// than tracking the hold, because percussion releases arrive within
    /// milliseconds (a hold-driven highlight would be an invisible flicker).
    /// Wall clock, not the playhead: the flash must animate while playback is
    /// stopped, where no score time passes at all.
    ///
    /// Bounded by the surface count (one entry per pad, one for the pedal), so
    /// nothing accumulates; expired entries are simply painted at zero
    /// intensity. Cleared whenever the surfaces themselves change (another
    /// score, an inverted layout), since the keys are positions.
    @Default(<int, double>{}) Map<int, double> struckSurfacesMs,

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

    /// Whether this session's input source is the microphone (change:
    /// add-acoustic-piano-input), seeded and kept in sync from the effective
    /// input source. An acoustic instrument sounds itself by definition, so
    /// live-event synthesis is suppressed **inherently** — independent of the
    /// [instrumentSoundsItself] setting, which stays a MIDI-scoped choice.
    /// Everything else — the on-screen keyboard, score playback, metronome,
    /// scoring, Wait Mode — is untouched, exactly as with that setting.
    @Default(false) bool usesMicrophoneInput,

    /// Set when starting scored free-run play with the microphone was refused
    /// and the session was steered to Wait Mode instead (spec: Free-Run Gated
    /// On Measured Latency) — consumed by a listener widget that shows the
    /// localized explanation, then acknowledged. Never a silent degradation.
    @Default(false) bool micSteeredToWaitMode,

    /// Whether the app stops sounding the **written score** (change:
    /// add-practice-focus-controls), seeded from the persisted play
    /// preferences. The counterpart of [instrumentSoundsItself] on the other
    /// side of the exercise: that one silences what the player *plays*, this
    /// one what the app *asks for*.
    ///
    /// Presentation only, like every other audio rule here. The playhead still
    /// advances, the score is still drawn, the Wait Mode gate still holds and
    /// releases, the scorer still judges, and the metronome still clicks —
    /// [PlayerData] cannot tell the difference anywhere except in what reaches
    /// the synth.
    @Default(false) bool scoreAudioMuted,

    /// Output latency compensation in **wall-clock** milliseconds (change:
    /// add-audio-output-routing), seeded from the persisted play preferences.
    /// The audio the user hears at any instant is what the engine emitted this
    /// many real milliseconds ago, so [referenceMs] — the position they are
    /// actually hearing — is what the playhead is drawn at and what a free-run
    /// attack is judged against. 0 (the default) makes it a no-op, at every
    /// transport speed and in both modes. See [referenceMs] for the known
    /// wall-clock/score-clock unit defect at speeds other than 1x.
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
      !((instrumentSoundsItself || usesMicrophoneInput) &&
          source == NoteSource.midiDevice);

  /// The score position the player is **hearing** right now — the playhead
  /// shifted back by [outputOffsetMs] (change: add-audio-output-routing).
  ///
  /// [elapsedMs] is the emission clock: it is what decides when a note is handed
  /// to the audio engine. On a delayed route that sound only reaches the ear
  /// [outputOffsetMs] later, so this is the position the highlight must show —
  /// and, in free run, the reference an attack is judged against. With the
  /// default offset of 0 it *is* [elapsedMs].
  ///
  /// **Known defect, deliberately not fixed here** (see the
  /// `fix-output-offset-units` change): [outputOffsetMs] is a *wall-clock*
  /// latency while this is a *score* clock, so at a transport speed other than
  /// 1x the shift is off by a factor of `speed`. Rescaling it instantly
  /// (`outputOffsetMs * speed`) is NOT the fix — it makes this clock a function
  /// of a value the user can change mid-run, and a tempo tap then teleports it
  /// by `outputOffsetMs * Δspeed`, sweeping pending onsets into `missed` and
  /// truncating sustains. The lag has to *drain* like the audio it models, i.e.
  /// this has to become a delayed copy of the playhead rather than a rescale.
  double get referenceMs => elapsedMs - outputOffsetMs;

  /// Both score clocks for a playhead at [playheadMs]: the emission clock (the
  /// playhead itself) and the heard clock (shifted back by [outputOffsetMs] —
  /// [referenceMs] for the current playhead). The single place the pair is
  /// assembled; the scorer receives both and picks per call — and, for
  /// sustains, per note (see `judgmentClock` / `sustainClock` in
  /// `performance_scoring_core.dart`).
  ScoreClocks clocksAt(double playheadMs) =>
      (emission: playheadMs, heard: playheadMs - outputOffsetMs);

  /// [clocksAt] for the current playhead.
  ScoreClocks get clocks => clocksAt(elapsedMs);

  /// The score clock the scorer is driven on, for a playhead at [playheadMs].
  ///
  /// **Free run** judges an attack by its distance from the onset, and the
  /// player is reacting to what they *hear*, so the heard position is the fair
  /// reference — that is the whole point of the output offset.
  ///
  /// **Wait Mode** is different, and must use the emission clock. There the
  /// judgment is a reaction time measured on the wall clock; the score clock is
  /// used only to identify *which* onset is gating, by matching the frozen
  /// playhead to within a millisecond. Nothing has sounded yet during a freeze —
  /// the player is waiting to play the note, not to hear it — so shifting that
  /// clock buys nothing and makes the frozen playhead miss its own onset, which
  /// silently drops every press on the floor.
  ///
  /// Toggling Wait Mode mid-run switches this clock by [outputOffsetMs]. The
  /// verdict model changes with it anyway (reaction time vs timing offset) —
  /// but a *sustain* must never straddle the switch, which is why the scorer
  /// measures each note's hold on the clock that bound it, not on this one
  /// (see `sustainClock` in `performance_scoring_core.dart`).
  double judgmentClockAt(double playheadMs) =>
      judgmentClock(clocksAt(playheadMs), waitMode: waitMode);

  /// [judgmentClockAt] for the current playhead.
  double get judgmentClockMs => judgmentClockAt(elapsedMs);

  /// The emission-clock position a scored run finishes at: the first playhead
  /// whose *judgment* clock has reached [endMs]. In Wait Mode — and always at
  /// the default offset of 0 — that is [endMs] itself. In free run under an
  /// output offset the judgment clock trails the playhead by [outputOffsetMs],
  /// so the run keeps judging through that drain tail: finalizing at [endMs]
  /// would resolve every onset in the piece's last [outputOffsetMs] as
  /// `missed` before the player has even heard it, and truncate the final
  /// sustains by the same amount — the tail of the piece would be unreachable
  /// on a delayed route.
  double get scoredRunEndMs => waitMode ? endMs : endMs + outputOffsetMs;

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

  /// The lanes in PRESENTATION order: the derived layout, reversed when the
  /// inverted-kit setting is on. The one point the cascade and the pad strip
  /// both read, so the two surfaces can never disagree — and the only place
  /// the inversion applies: notation and note interpretation never see it.
  List<DrumLane> get presentedDrumLanes =>
      invertedKit ? drumLanes.reversed.toList() : drumLanes;

  /// The General MIDI number the **kick pedal** emits for the loaded score, or
  /// null when the score writes no kick — in which case the strip draws no
  /// pedal at all (change: add-drum-input-mapping). Read from [notes], never
  /// [visibleNotes]: the pedal is a controller surface, and hand selection
  /// filters what is *shown and judged*, never what the player may strike.
  int? get kickEmissionGm => emittedKickGm({
    for (final n in notes)
      if (kKickGmNumbers.contains(n.pitch)) n.pitch,
  });

  /// Whether the pad strip draws the kick pedal (the score's foot bar exists).
  bool get hasKickPedal => kickEmissionGm != null;

  /// The General MIDI number a tap on controller [surface] emits — a pad by
  /// its presented lane index, [kPedalSurface] for the pedal — or null when
  /// the surface does not exist on this score (change: add-drum-input-mapping).
  int? emissionGmForSurface(int surface) {
    if (surface == kPedalSurface) return kickEmissionGm;
    final lanes = presentedDrumLanes;
    if (surface < 0 || surface >= lanes.length) return null;
    return emittedGmOfLane(lanes[surface]);
  }

  /// The controller surface a struck [gm] flashes, or null for a number the
  /// strip does not present (free play — audible, nothing to flash).
  int? struckSurfaceFor(int gm) =>
      struckSurfaceOf(presentedDrumLanes, gm, hasPedal: hasKickPedal);

  /// Milliseconds per beat as the score writes it, or 0 when the tempo is
  /// unknown. The bpm is the QUARTER rate, so a 6/8 beat is not 60000/bpm —
  /// the time signature's lower number scales it, and a missing one reads as
  /// a quarter.
  ///
  /// Derived here rather than at each surface: the two drum surfaces and the
  /// staff all draw the same grid, and three copies of this arithmetic would
  /// drift.
  double get beatMs {
    if (bpm <= 0) return 0;
    final unit = beatType == 0 ? 4 : beatType;
    return (60000 / bpm) * (4 / unit);
  }

  /// The pieces of the loaded score's kit, in the order the pad strip draws
  /// them (change: add-practice-focus-controls) — the list the focus control
  /// lists, and what "every piece" means when a selection is cleared.
  ///
  /// Read from [presentedDrumLanes], so the inverted-kit layout reorders the
  /// control with the instrument: the two read left to right the same way.
  List<String> get kitPieceIds => isPercussion
      ? kitPieceIdsOf(presentedDrumLanes, hasKick: hasKickPedal)
      : const [];

  /// The pieces the session asks for — [kitPieceIds] minus [mutedDrumPieces].
  ///
  /// The spec's own vocabulary, derived rather than stored: holding the
  /// complement is what makes "everything" the default and makes a piece the
  /// kit model never enumerated read as in focus (see [mutedDrumPieces]).
  Set<String> get focusedDrumPieces => {
    for (final id in kitPieceIds)
      if (!mutedDrumPieces.contains(id)) id,
  };

  /// The controller surfaces the current focus selection asks for at all.
  ///
  /// Derived from the whole score rather than from [visibleNotes] on purpose:
  /// it answers "does this selection ever ask for this piece", not "in this
  /// passage", so a measure-range practice does not grey out most of the kit.
  /// Empty outside a percussion score, and empty when nothing is muted — both
  /// mean "everything is live", which is what the surfaces draw by default.
  Set<int> get playableDrumSurfaces {
    if (!isPercussion || mutedDrumPieces.isEmpty) return const {};
    final result = <int>{};
    for (final n in notes) {
      if (!isDrumPieceInFocus(n.pitch, mutedDrumPieces)) continue;
      final surface = struckSurfaceFor(n.pitch);
      if (surface != null) result.add(surface);
    }
    return result;
  }

  /// Whether there is more than one piece to choose between — the precondition
  /// for offering the focus control at all (a one-piece kit has nothing to
  /// isolate, and muting its only piece would restore it immediately anyway).
  bool get hasDrumPiecesToFocus => isPercussion && kitPieceIds.length > 1;

  /// Whether this run asks for less than the whole kit (change:
  /// add-practice-focus-controls, design D7).
  ///
  /// Such a run IS scored and IS shown to the player — it reaches the last bar
  /// and its verdict on what it asked for is honest — but it is not submitted:
  /// a clean groove with the crashes muted is not the same achievement as a
  /// clean groove, and the boards carry the same piece id either way. It still
  /// counts as a **practice** session, so isolating part of a groove never
  /// costs the player their streak.
  bool get isFocusRestrictedRun => isPercussion && mutedDrumPieces.isNotEmpty;

  /// Whether the current selection shows this note — the ONE predicate every
  /// render mode, the Wait Mode gate and the scorer reach through
  /// [visibleNotes], so what is drawn, what is awaited and what is judged can
  /// never disagree.
  ///
  /// A keyboard score splits by staff (right = staff 1); a percussion score
  /// splits by **kit piece** (change: add-practice-focus-controls), at the
  /// grain a drummer isolates a groove in.
  bool _showsNote(TimedNote n) => isPercussion
      ? isDrumPieceInFocus(n.pitch, mutedDrumPieces)
      : showsStaff(n.staff);

  /// Notes the current selection asks for — the input every render mode and
  /// the gate derive from, so display and Wait Mode stay consistent.
  ///
  /// During a **selective run** this is the passage's notes only. The run cannot
  /// reach the rest of the piece, so showing it is misleading: the player cannot
  /// see where the passage ends, and the Wait-Mode gate would otherwise hold at
  /// an onset outside the loop. Restricting at this single source keeps the
  /// display, the gate and the score audio in agreement by construction.
  List<TimedNote> get visibleNotes => notes
      .where((n) => _showsNote(n) && _withinRun(n.startMs.toDouble()))
      .toList();

  /// Whether the current selection shows this rest. Keyboard scores split by
  /// staff; a **percussion** score never hides a rest (change:
  /// add-practice-focus-controls): a rest carries no General MIDI number, so it
  /// belongs to no kit piece, and the groove's silence is the whole part's
  /// rather than any one piece's. The voice-keyed split that used to hide voice
  /// 2's rests went with the hands/feet reading it served.
  bool _showsRest(TimedRest r) => isPercussion || showsStaff(r.staff);

  /// Rests the current selection asks for — the render-only companion to
  /// [visibleNotes], so the Staff painter hides a muted hand's rests with its
  /// notes (and, in a selective run, everything outside the passage).
  List<TimedRest> get visibleRests => rests
      .where((r) => _showsRest(r) && _withinRun(r.startMs.toDouble()))
      .toList();

  /// Tie continuations the current selection asks for — the render-only
  /// companion to [visibleNotes] (same filter, per-piece on percussion), so a
  /// muted piece's tied notation hides with its notes (and, in a selective run,
  /// everything outside the passage).
  List<TimedNote> get visibleTieContinuations => tieContinuations
      .where((n) => _showsNote(n) && _withinRun(n.startMs.toDouble()))
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
  /// carries no measure table). With repeats unrolled this counts **played
  /// slots**; the range pickers use [practiceMeasureCount] instead.
  int get measureCount => measureStartMs.length;

  /// Number of **written** measures — the domain of the practice range and of
  /// Partition taps, independent of repeat unrolling. Falls back to the played
  /// table for pieces loaded before the written count existed (demo: 0).
  int get practiceMeasureCount =>
      writtenMeasureCount > 0 ? writtenMeasureCount : measureCount;

  /// The written measure performed at played slot [slot] (identity when the
  /// piece has no repeats or the run is linear).
  int writtenMeasureAt(int slot) =>
      (slot >= 0 && slot < writtenMeasureOf.length)
      ? writtenMeasureOf[slot]
      : slot;

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

  /// Whether an attack of [incoming] satisfies a note the gate requires as
  /// [required] — the gate's half of the **one stroke identity** (change:
  /// add-drum-scoring).
  ///
  /// Pitch equality for a keyboard score; the shared kit-piece equivalence of
  /// `drum_kit.dart` for a percussion one, which is the same predicate the
  /// scorer binds with. A stroke that releases the gate is therefore exactly a
  /// stroke the scorer binds — the drift two independent tables would allow is
  /// inexpressible.
  ///
  /// The hi-hat articulation is deliberately not consulted: it shades a bound
  /// stroke's verdict and never gates, so a kit with no hi-hat controller can
  /// still complete a run written with open hi-hats.
  bool strokeSatisfies(int required, int incoming) =>
      isPercussion ? samePiece(required, incoming) : required == incoming;

  /// The onset numbers at instant [t] that an attack of [gm] satisfies — the
  /// set to latch into `gateSatisfied`, empty when the attack answers nothing
  /// the onset asks for.
  ///
  /// Returns the **required** numbers, not the incoming one, so the gate's
  /// `containsAll(onset)` release check keeps reading the score's own
  /// vocabulary: a written 38 satisfied by an incoming 40 latches 38.
  Set<int> onsetPitchesSatisfiedBy(int gm, double t) {
    final result = <int>{};
    for (final required in onsetPitchesAt(t)) {
      if (strokeSatisfies(required, gm)) result.add(required);
    }
    return result;
  }

  /// The controller surfaces the pad strip should show as **expected** — the
  /// pads of the pieces the gate is waiting for, plus [kPedalSurface] when a
  /// kick is required (change: add-drum-scoring). Empty outside a percussion
  /// score, and empty when nothing is expected.
  ///
  /// Derived from [expectedKeys], so the strip, the gate and the judgment
  /// always name the same onset; resolved through [struckSurfaceFor], so an
  /// expected pad and a struck one are the same surface.
  Set<int> get expectedDrumSurfaces {
    if (!isPercussion) return const {};
    final result = <int>{};
    for (final gm in expectedKeys) {
      final surface = struckSurfaceFor(gm);
      if (surface != null) result.add(surface);
    }
    return result;
  }

  /// The selection as the session result records it: the keyboard's own
  /// `left` / `right` / `both`.
  ///
  /// A percussion run always reports `both` (change:
  /// add-practice-focus-controls): the hands/feet reading that used to give it
  /// its own tokens is gone, and a focus selection is not a hand selection —
  /// a restricted drum run is marked as such by not being submitted at all
  /// (see `isFocusRestrictedRun`), not by a label on a submitted result.
  String get handsSelectionName => selectedHands.name;

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
