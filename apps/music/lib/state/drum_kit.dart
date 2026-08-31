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

/// The drum-kit model behind the percussion cascade and the pad strip
/// (change: add-drum-kit-view): which General MIDI percussion numbers denote
/// which physical piece, how the pieces present in a score become an ordered
/// lane layout, and how events split into hands and feet.
///
/// Pure and host-testable. The layout is derived ONCE per loaded score and
/// consumed by both the cascade and the pad strip — never derived twice — so
/// the two surfaces cannot drift (the same contract `keyboard-display` makes
/// between the keyboard and the waterfall, restated over the lane order).
library;

import 'player_data.dart' show TimedNote;

/// The roles a lane can play in the sort rule. [other] is the terminal
/// bucket: a resolved number outside the named roles takes a lane of its own
/// rather than being dropped — an invisible note that is still scheduled is a
/// worse failure than an inelegant lane (no-silent-drop requirement).
enum KitPieceRole { hiHat, snare, tom, ride, crash, other }

/// How far from a written onset a stroke still counts as that stroke, in
/// milliseconds either side.
///
/// A kit is played by feel, not by aim: the hand leaves before the ear checks,
/// and a window narrower than the player's own jitter turns a groove into a
/// typing test. The keyboard has always had its equivalent — a key held into
/// its onset satisfies it with no fresh press — and this is the percussion
/// half of the same tolerance: it releases the Wait Mode gate for a stroke
/// played slightly EARLY, and it is the window in which a surface lights the
/// note it answered.
///
/// One number, shared by the gate and by both drawn surfaces, so what is
/// accepted and what is shown can never drift apart.
const double kStrokeToleranceMs = 150;

/// The window at transport [speed], in the PLAYHEAD's milliseconds.
///
/// The window is measured musically — that is where the onset lives, and it is
/// what makes it deterministic — but a player's jitter is a wall-clock
/// phenomenon: the hand does not steady itself because the transport is set to
/// 200 %. A flat musical window would therefore halve in real terms exactly
/// where the exercise is already hardest.
///
/// So the window has a **floor in real time**: it is never shorter, in
/// wall-clock terms, than it is at normal speed. Slowing down still widens it
/// (a musical window stretches with the music, which is what a learner wants);
/// speeding up leaves it alone.
double strokeToleranceMsAt(double speed) =>
    kStrokeToleranceMs * (speed > 1 ? speed : 1);

/// Whether a stroke stamped at [strokeMs] on the playhead still answers the
/// onset the playhead has reached at [nowMs], at transport [speed].
///
/// Bounded on both sides, and that is the point. The window is measured on the
/// playhead, so a stamp only means anything on the run that made it: the
/// playhead runs backwards across a restart, a loop wrap or a rewind, and a
/// stroke left over from before one of those lands *ahead* of it. Read as an
/// age that would be a huge negative number — "played 40 seconds early" — which
/// no upper bound alone rejects, so it would silently open the next run's very
/// first onset with no stroke played at all.
bool strokeAnswersOnset({
  required double strokeMs,
  required double nowMs,
  required double speed,
}) {
  final age = nowMs - strokeMs;
  return age >= 0 && age <= strokeToleranceMsAt(speed);
}

/// How long a struck surface stays lit, in milliseconds.
const int kStruckFlashMs = 180;

/// [kStruckFlashMs] as a [Duration], for the repaint clock a surface runs on
/// while playback is stopped.
const Duration kStruckFlash = Duration(milliseconds: kStruckFlashMs);

/// The flash intensity of a surface struck at [struckMs] as seen at [nowMs]:
/// 1 at the attack, decaying linearly to 0 over [kStruckFlashMs]. Pure, so
/// the decay is testable without a clock.
///
/// Time-based, never hold-based: a percussion release arrives within
/// milliseconds of its attack, so a highlight that lasted the hold would be an
/// invisible flicker. Nothing here depends on the note-off, on the playhead,
/// or on whether the stroke landed on an onset — one state, claiming nothing.
///
/// Lives with the kit model rather than with any one painter: every surface
/// that draws a struck piece reads the same decay.
double struckFlashIntensity({required double struckMs, required double nowMs}) {
  final age = nowMs - struckMs;
  if (age < 0 || age >= kStruckFlashMs) return 0;
  return 1 - age / kStruckFlashMs;
}

/// End of the played measure at index [m] in [measureStartMs]: the next
/// measure's start, or — for the last one, which has no successor — a bar's
/// worth of beats past its own start ([fallbackMs] when the beat is unknown).
///
/// Shared by both drum surfaces: they draw the same grid from the same table,
/// and two copies of this arithmetic would drift the day one of them changed.
double measureEndOf(
  List<int> measureStartMs,
  int m, {
  required double beatMs,
  required double fallbackMs,
}) {
  if (m + 1 < measureStartMs.length) return measureStartMs[m + 1].toDouble();
  final start = measureStartMs[m].toDouble();
  return start + (beatMs > 0 ? beatMs * 4 : fallbackMs);
}

/// The kick's General MIDI numbers — 35 (acoustic) and 36 (bass drum 1) both
/// drive the full-width bar and never occupy a lane: a lane encodes *where to
/// aim*, and the foot does not aim.
const Set<int> kKickGmNumbers = {35, 36};

/// The kick's numbers in **emission** order (change: add-drum-input-mapping):
/// Bass Drum 1 (36) first — the number virtually every notation export and
/// e-kit default map uses for *the* kick — then the Acoustic Bass Drum (35),
/// which a file may use instead. Same membership as [kKickGmNumbers], stated
/// as a list because emission needs an order a set cannot promise.
const List<int> kKickEmissionOrder = [36, 35];

/// Foot-struck General MIDI numbers: the two kicks and the pedal hi-hat
/// "chick" (44 — the foot closing the cymbals, no hand involved). The
/// single-voice hands/feet fallback keys on this set.
const Set<int> kFootGmNumbers = {35, 36, 44};

/// One lane of the cascade (and one pad of the strip): a physical piece, the
/// General MIDI numbers that denote it, and its display label.
class DrumLane {
  final KitPieceRole role;

  /// The numbers this lane collapses (e.g. closed 42 + open 46 hi-hat, the
  /// acoustic 38 / electric 40 snares and the 37 side stick — struck ON the
  /// snare). A generic piece carries exactly one.
  final Set<int> gmNumbers;

  /// Localisation key for a named piece (`kitPieceHiHat`…), or null for a
  /// generic piece, whose label is its General MIDI instrument name
  /// ([gmName]) — the standard names are a notation-level vocabulary and are
  /// not translated.
  final String? labelKey;

  /// The General MIDI name for a generic piece ("Cowbell", "Tambourine"…).
  final String? gmName;

  const DrumLane({
    required this.role,
    required this.gmNumbers,
    this.labelKey,
    this.gmName,
  });

  @override
  bool operator ==(Object other) =>
      other is DrumLane &&
      other.role == role &&
      other.labelKey == labelKey &&
      other.gmName == gmName &&
      other.gmNumbers.length == gmNumbers.length &&
      other.gmNumbers.containsAll(gmNumbers);

  @override
  int get hashCode => Object.hash(role, labelKey, gmName, gmNumbers.length);

  @override
  String toString() => 'DrumLane($role, $gmNumbers, ${labelKey ?? gmName})';
}

/// The named pieces: identity → (role, member GM numbers, l10n key). Built
/// against real exports (MuseScore/Finale drum maps), not the GM list alone:
/// numbers that denote one physical aim point collapse onto one lane.
///
/// The members are listed in **canonical emission order** (change:
/// add-drum-input-mapping): collapse-equal for the lane — one aim point, one
/// pad — but not for emission, where a tap must pick exactly one number. The
/// order is pinned here rather than left to set iteration so a pad's stroke is
/// reproducible; [emittedGmOfLane] applies it. Lane derivation is unchanged:
/// the same members, in the same lanes.
///
/// The toms are each their own piece — different drums, different aim — kept
/// here highest to lowest, the order the sort rule wants.
const List<({KitPieceRole role, List<int> gm, String key})> _namedPieces = [
  // Closed (42) and open (46) hi-hat are hand strokes on ONE instrument: a
  // variant of the note inside the lane, never a second lane. The pedal 44 is
  // a FOOT event and deliberately not part of it (it lands in the terminal
  // bucket until its bar encoding exists — see design.md). Closed first: it is
  // the stroke a groove is written with, and the strip offers no gesture that
  // distinguishes open from closed (open/closed is the FOOT on the instrument,
  // not a second aim point).
  (role: KitPieceRole.hiHat, gm: [42, 46], key: 'kitPieceHiHat'),
  // Snare: the acoustic 38 first, then the electric 40, then the side stick 37
  // — the sound a drummer means by "the snare", in decreasing ordinariness.
  (role: KitPieceRole.snare, gm: [38, 40, 37], key: 'kitPieceSnare'),
  (role: KitPieceRole.tom, gm: [50], key: 'kitPieceTomHigh'),
  (role: KitPieceRole.tom, gm: [48], key: 'kitPieceTomHiMid'),
  (role: KitPieceRole.tom, gm: [47], key: 'kitPieceTomLowMid'),
  (role: KitPieceRole.tom, gm: [45], key: 'kitPieceTomLow'),
  (role: KitPieceRole.tom, gm: [43], key: 'kitPieceTomFloorHigh'),
  (role: KitPieceRole.tom, gm: [41], key: 'kitPieceTomFloorLow'),
  // The ride family (ride 1/2 and the bell) is one time-keeping cymbal: the
  // ride 51 first, then the second ride 59, the bell 53 last (a bell stroke is
  // a deliberate accent, never the default aim).
  (role: KitPieceRole.ride, gm: [51, 59, 53], key: 'kitPieceRide'),
  // Accent cymbals are DIFFERENT physical cymbals (different aim): one lane
  // each, ordered stably among the cymbals by their lowest GM number.
  (role: KitPieceRole.crash, gm: [49], key: 'kitPieceCrash'),
  (role: KitPieceRole.crash, gm: [52], key: 'kitPieceChina'),
  (role: KitPieceRole.crash, gm: [55], key: 'kitPieceSplash'),
  (role: KitPieceRole.crash, gm: [57], key: 'kitPieceCrash2'),
];

/// The lowest and highest numbers the General MIDI percussion map names
/// (Acoustic Bass Drum 35 … Open Triangle 81). A stroke outside this window is
/// not part of the standard map at all — which a diagnostic surface says out
/// loud (change: add-drum-input-calibration), because it is the difference
/// between "the app read your stroke as the wrong piece" and "the app has no
/// name for what your kit sent", and only the second one explains silence.
const int kGmPercussionLowest = 35;
const int kGmPercussionHighest = 81;

/// The General MIDI percussion name of [gm], or null when the number is outside
/// the standard map.
///
/// The whole map, not just the terminal bucket: a diagnostic surface names what
/// arrived before it says anything about the loaded score's kit, and the named
/// pieces' *display* labels are localised l10n keys, which is the wrong
/// vocabulary here — General MIDI instrument names are a notation-level
/// standard and are not translated.
String? gmPercussionName(int gm) => _gmNames[gm];

/// General MIDI percussion names, 35–81 — the whole standard map. The named
/// pieces (which carry localised labels for display) are here too, because a
/// monitor reports what the *standard* calls a number, not what this app's kit
/// view chose to label it.
const Map<int, String> _gmNames = {
  35: 'Acoustic Bass Drum',
  36: 'Bass Drum 1',
  37: 'Side Stick',
  38: 'Acoustic Snare',
  40: 'Electric Snare',
  41: 'Low Floor Tom',
  42: 'Closed Hi Hat',
  43: 'High Floor Tom',
  44: 'Pedal Hi-Hat',
  45: 'Low Tom',
  46: 'Open Hi-Hat',
  47: 'Low-Mid Tom',
  48: 'Hi-Mid Tom',
  49: 'Crash Cymbal 1',
  50: 'High Tom',
  51: 'Ride Cymbal 1',
  52: 'Chinese Cymbal',
  53: 'Ride Bell',
  55: 'Splash Cymbal',
  57: 'Crash Cymbal 2',
  59: 'Ride Cymbal 2',
  39: 'Hand Clap',
  54: 'Tambourine',
  56: 'Cowbell',
  58: 'Vibraslap',
  60: 'Hi Bongo',
  61: 'Low Bongo',
  62: 'Mute Hi Conga',
  63: 'Open Hi Conga',
  64: 'Low Conga',
  65: 'High Timbale',
  66: 'Low Timbale',
  67: 'High Agogo',
  68: 'Low Agogo',
  69: 'Cabasa',
  70: 'Maracas',
  71: 'Short Whistle',
  72: 'Long Whistle',
  73: 'Short Guiro',
  74: 'Long Guiro',
  75: 'Claves',
  76: 'Hi Wood Block',
  77: 'Low Wood Block',
  78: 'Mute Cuica',
  79: 'Open Cuica',
  80: 'Mute Triangle',
  81: 'Open Triangle',
};

/// Derive the ordered lane layout from the notes actually present in the
/// score — never a fixed kit. The kick is excluded by construction (it is the
/// bar). The order is a RULE applied to the pieces present, protecting one
/// invariant: position 1 is whatever is struck continuously (the hi-hat, or
/// the ride when there is none) and position 2 the snare; further pieces
/// append to the right, so a player moving between sparse and dense scores
/// never relearns where to look. A snare-less score closes ranks leftward —
/// empty buckets are skipped, never reserved.
List<DrumLane> deriveDrumLanes(Iterable<TimedNote> notes) {
  final present = <int>{
    for (final n in notes)
      if (!kKickGmNumbers.contains(n.pitch)) n.pitch,
  };
  if (present.isEmpty) return const [];

  final lanes = <DrumLane>[];
  DrumLane? named(({KitPieceRole role, List<int> gm, String key}) piece) {
    // Filter-and-consume in one pass, walking the piece's canonical order so
    // the lane's member set carries it too (membership is identical to the
    // former set intersection — only the iteration order is now pinned).
    final members = <int>{
      for (final gm in piece.gm)
        if (present.remove(gm)) gm,
    };
    if (members.isEmpty) return null;
    return DrumLane(role: piece.role, gmNumbers: members, labelKey: piece.key);
  }

  final hiHat = named(_namedPieces[0]);
  final snare = named(_namedPieces[1]);
  final toms = [
    for (final p in _namedPieces.where((p) => p.role == KitPieceRole.tom))
      ?named(p),
  ];
  final ride = named(
    _namedPieces.firstWhere((p) => p.role == KitPieceRole.ride),
  );
  final crashes = [
    for (final p in _namedPieces.where((p) => p.role == KitPieceRole.crash))
      ?named(p),
  ];

  // 1. The time-keeper: the hi-hat, or the ride when there is none. The rule
  //    is keyed to the piece's FUNCTION (keeping time), not its name.
  final rideKeepsTime = hiHat == null && ride != null;
  if (hiHat != null) lanes.add(hiHat);
  if (rideKeepsTime) lanes.add(ride);
  // 2. The snare — adjacent to the time-keeper: together they carry the
  //    overwhelming majority of a groove's notes and must sit inside one eye
  //    fixation.
  if (snare != null) lanes.add(snare);
  // 3. The toms, highest to lowest (the list above is already in that order).
  lanes.addAll(toms);
  // 4. The remaining cymbals: the ride (when the hi-hat took position 1),
  //    then the accent cymbals in stable ascending GM order.
  if (!rideKeepsTime && ride != null) lanes.add(ride);
  lanes.addAll(crashes);
  // 5. The terminal bucket: any other resolved piece, one generic lane each,
  //    in stable ascending GM order — visible and aim-able beats
  //    invisible-but-scheduled (GM 44 lands here until its bar encoding
  //    exists).
  final generic = present.toList()..sort();
  lanes.addAll([
    for (final gm in generic)
      DrumLane(
        role: KitPieceRole.other,
        gmNumbers: {gm},
        gmName: _gmNames[gm] ?? 'GM $gm',
      ),
  ]);
  return lanes;
}

/// Index of the lane a General MIDI number falls in, or null for the kick
/// (which has no lane — it is the full-width bar) and for a number absent
/// from the layout.
int? laneIndexOf(List<DrumLane> lanes, int gm) {
  for (var i = 0; i < lanes.length; i++) {
    if (lanes[i].gmNumbers.contains(gm)) return i;
  }
  return null;
}

/// The piece identity standing for the kick pedal — the one named piece with
/// no [_namedPieces] row, because it has no lane (it is the full-width bar).
const String kKickPieceId = 'kick';

/// The **kit piece** a General MIDI percussion number denotes (change:
/// add-drum-scoring) — the grain at which strokes are matched.
///
/// Read from the STATIC named-piece table above, never from the score-derived
/// [DrumLane] layout: the layout intersects each group with the numbers the
/// score actually writes, so a number the file never uses has no lane — yet it
/// still denotes the same physical piece and must still resolve to it. The
/// table is the identity; the layout is a view of it.
///
/// The kick's two numbers share [kKickPieceId] (one pedal); every named group
/// shares its l10n key (hi-hat {42, 46}, snare {37, 38, 40}, ride {51, 53, 59},
/// each tom and each accent cymbal alone); anything else — the terminal
/// bucket — is a piece of its own, keyed by its number, so it matches only
/// itself.
String drumPieceIdOf(int gm) {
  if (kKickGmNumbers.contains(gm)) return kKickPieceId;
  for (final piece in _namedPieces) {
    if (piece.gm.contains(gm)) return piece.key;
  }
  return gmPieceId(gm);
}

/// Whether an incoming stroke [incomingGm] is a stroke of the piece a written
/// note [writtenGm] asks for — **the one stroke-identity function** every
/// consumer routes through (change: add-drum-scoring): the Wait Mode gate's
/// required set, the scorer's binding decision, extra-stroke detection and the
/// pad feedback.
///
/// The governing principle is *never demand a distinction the cascade does not
/// draw*: numbers the display collapses into one lane with one look — which
/// zone of the snare, which flavour of kick, which ride surface — are
/// hardware-mapping artefacts the player can neither see nor control, so they
/// match; pieces with lanes of their own are separate aim points, so hitting
/// the wrong one is exactly the error scoring exists to catch.
bool samePiece(int writtenGm, int incomingGm) =>
    drumPieceIdOf(writtenGm) == drumPieceIdOf(incomingGm);

/// Whether [incomingGm] carries the **articulation** the written [writtenGm]
/// asks for — the open/closed hi-hat, the one distinction the cascade draws
/// *inside* a lane (the hollow variant).
///
/// Never a binding or gating question ([samePiece] answers those): the two
/// numbers are one piece, because producing the open stroke needs a hi-hat
/// controller many electronic kits lack and a practice gate must never block on
/// a stroke the hardware cannot emit. It only *shades* the verdict — see
/// `capBelowPerfect` in `performance_scoring_core.dart`. Trivially true for
/// every other piece, which draws no in-lane variant.
bool sameStrokeArticulation(int writtenGm, int incomingGm) =>
    isOpenHiHat(writtenGm) == isOpenHiHat(incomingGm);

/// Index of the lane whose **piece** [gm] belongs to, or null for the kick and
/// for a piece the layout does not present.
///
/// The piece-grained counterpart of [laneIndexOf] (change: add-drum-scoring):
/// a stroke of 40 lights the snare lane of a score written entirely in 38s,
/// because it is a stroke of that lane's piece. [laneIndexOf] stays the exact
/// lookup the painters place *written* notes with — a written note is always a
/// member of its own derived lane, so the two agree there by construction.
int? pieceLaneIndexOf(List<DrumLane> lanes, int gm) {
  final id = drumPieceIdOf(gm);
  for (var i = 0; i < lanes.length; i++) {
    for (final member in lanes[i].gmNumbers) {
      if (drumPieceIdOf(member) == id) return i;
    }
  }
  return null;
}

/// The [struckSurfaceOf] index standing for the **kick pedal** — the one
/// controller surface that is not a lane (change: add-drum-input-mapping). The
/// pads take the lane indices `0..n-1`, so a negative sentinel cannot collide
/// with one, and pads and pedal share a single struck-flash table.
const int kPedalSurface = -1;

/// The General MIDI number a tap on [lane]'s pad emits: the **first member of
/// the piece's canonical order that the loaded score actually uses** (change:
/// add-drum-input-mapping). A lane holds exactly the members the score writes
/// (they are derived from the notes present), so "first canonical member of
/// the lane" *is* "first the score uses": a score writing its snare only as
/// the electric 40 emits 40, never the absent canonical 38, and an open-only
/// hi-hat emits 46.
///
/// Emitting inside the score's own vocabulary keeps the stroke audible with
/// the score's own piece and spares the future matcher (`add-drum-scoring`)
/// avoidable same-piece mismatches. It also keeps the struck-flash lookup
/// trivially correct: the emitted number is always a member of the lane it
/// came from.
///
/// The canonical order is read from the piece table rather than from the
/// lane's set iteration order: a [DrumLane] built by hand elsewhere must emit
/// the same number as a derived one, so emission must not depend on how the
/// set happened to be constructed. A generic (terminal-bucket) lane has no
/// table entry and exactly one number — it emits that one.
int emittedGmOfLane(DrumLane lane) {
  final key = lane.labelKey;
  if (key != null) {
    for (final piece in _namedPieces) {
      if (piece.key != key) continue;
      for (final gm in piece.gm) {
        if (lane.gmNumbers.contains(gm)) return gm;
      }
      break;
    }
  }
  // Generic piece (one number), or a hand-built lane holding none of its
  // piece's canonical members: the lowest number, so the answer is still
  // deterministic rather than set-iteration-dependent.
  return lane.gmNumbers.reduce((a, b) => a < b ? a : b);
}

/// The General MIDI number the **kick pedal** emits for a score whose kick is
/// written with [present] (change: add-drum-input-mapping): Bass Drum 1 (36)
/// whenever the file uses it, 35 for a score writing its kick only as the
/// Acoustic Bass Drum — the same present-member rule the pads follow, so the
/// pedal's stroke also stays inside the score's vocabulary.
///
/// Null when the score has no kick at all: there is no pedal on the strip to
/// tap, and nothing to emit.
int? emittedKickGm(Set<int> present) {
  for (final gm in kKickEmissionOrder) {
    if (present.contains(gm)) return gm;
  }
  return null;
}

/// The controller surface a struck General MIDI number lights up: the index of
/// the pad whose lane collapses [gm] — in the order [lanes] is given, so the
/// caller passes PRESENTED lanes for the strip — or [kPedalSurface] for a kick
/// when the strip draws a pedal ([hasPedal]).
///
/// Null when the number belongs to no surface the controller presents: a piece
/// the score does not use, or a kick on a kickless score. That stroke is free
/// play — it sounds and simply has nothing to flash, which is not an error
/// (change: add-drum-input-mapping).
///
/// Resolved at the **piece** grain (change: add-drum-scoring), so the flash
/// reflects the same piece resolution the scorer judged the stroke with: an
/// incoming 40 lights the snare pad of a score written only in 38s, exactly as
/// it binds to that score's snare onsets. Two independent resolutions here
/// would let the strip contradict the verdict.
int? struckSurfaceOf(List<DrumLane> lanes, int gm, {required bool hasPedal}) {
  if (kKickGmNumbers.contains(gm)) return hasPedal ? kPedalSurface : null;
  return pieceLaneIndexOf(lanes, gm);
}

/// Whether a percussion note is FOOT-struck, per the convention stated in
/// `hand-color-coding`: voice 1 (stems up) is the hands and voice 2 (stems
/// down) the feet; a single-voice file (common in real exports) falls back to
/// the General MIDI number — the kicks and the pedal hi-hat are feet,
/// everything else hands. [multiVoice] is whether the loaded score's
/// percussion notes span more than one voice (precomputed once).
bool isFootNote(TimedNote n, {required bool multiVoice}) =>
    isFootEvent(voice: n.voice, gmNumber: n.pitch, multiVoice: multiVoice);

/// [isFootNote] over raw values, for consumers that hold the parsed document
/// rather than [TimedNote]s (the engraved Partition painter, change:
/// add-drum-notation-render). The single implementation of the convention —
/// the surfaces cannot drift. A null [gmNumber] (unresolved) counts as a hand
/// under the single-voice fallback: only the named foot numbers are feet.
bool isFootEvent({
  required int voice,
  required int? gmNumber,
  required bool multiVoice,
}) => multiVoice ? voice >= 2 : kFootGmNumbers.contains(gmNumber);

/// Whether [notes] span more than one voice — the precondition for the
/// voice-keyed hands/feet split (single-voice files use the GM fallback).
bool spansMultipleVoices(Iterable<TimedNote> notes) {
  int? seen;
  for (final n in notes) {
    seen ??= n.voice;
    if (n.voice != seen) return true;
  }
  return false;
}

/// The open hi-hat (GM 46): drawn as a visual variant of the note inside the
/// hi-hat lane — never a bar, never a lane of its own (open-versus-closed is
/// a different number on the HAND stroke; no foot note exists in the file).
bool isOpenHiHat(int gm) => gm == 46;

// --- Per-piece practice focus (change: add-practice-focus-controls) ---------

/// The piece identity of [lane] — the same identity [drumPieceIdOf] gives its
/// members, read through the lane's emitted number.
///
/// Deliberately NOT a second notion of "one piece": a lane already holds only
/// numbers that collapse onto one aim point, so any member answers the same,
/// and routing through [emittedGmOfLane] picks the one member the lane is
/// guaranteed to have.
String drumPieceIdOfLane(DrumLane lane) => drumPieceIdOf(emittedGmOfLane(lane));

/// The pieces of a loaded kit, in the order the pad strip draws them: the
/// [lanes] as presented, then the kick pedal when the score writes one — pads
/// band first, pedal band beneath, exactly as
/// the pad-strip and cascade painters lay the strip out.
///
/// This is the list the focus control lists and the list "every piece" means
/// when a selection is cleared. It is derived from the score's own kit, so a
/// piece the file never writes is never offered.
List<String> kitPieceIdsOf(List<DrumLane> lanes, {required bool hasKick}) => [
  for (final lane in lanes) drumPieceIdOfLane(lane),
  if (hasKick) kKickPieceId,
];

/// Whether the piece a General MIDI number denotes is **in focus** — i.e. not
/// among [mutedPieceIds] (change: add-practice-focus-controls).
///
/// Keyed on the piece, never on the number, so the distinctions the kit model
/// already collapsed stay collapsed: muting the hi-hat mutes the closed 42 and
/// the open 46 together, because they are one pad on the instrument, and
/// muting the snare takes the acoustic 38, the electric 40 and the side stick
/// 37 with it.
///
/// The **complement** is stored (what is muted) rather than the focus set, so
/// an unknown piece — a number the score never writes, reached by a stroke on
/// free-play territory — reads as in focus. The other direction would silently
/// filter out anything the kit model had not enumerated.
bool isDrumPieceInFocus(int gm, Set<String> mutedPieceIds) =>
    mutedPieceIds.isEmpty || !mutedPieceIds.contains(drumPieceIdOf(gm));

/// The muted set after muting [pieceId] out of [all], honouring the
/// empty-focus rule (design D2): muting the last piece in focus restores every
/// piece rather than leaving a session that draws nothing, gates on nothing and
/// judges nothing — a state indistinguishable from a broken score.
Set<String> mutedAfterMuting(
  Set<String> muted,
  String pieceId,
  List<String> all,
) {
  final next = {...muted, pieceId};
  // Every piece of the kit muted ⇒ nothing in focus ⇒ restore everything.
  return all.every(next.contains) ? const {} : next;
}

/// The muted set after **soloing** [pieceId] out of [all] (design D2): solo is
/// the same one set, edited from the other side.
///
/// From "everything in focus" it isolates [pieceId]; from an existing
/// selection it **adds** — soloing the snare after the hi-hat asks for both,
/// which is what a drummer building up a groove means by isolating a second
/// piece.
Set<String> mutedAfterSoloing(
  Set<String> muted,
  String pieceId,
  List<String> all,
) =>
    muted.isEmpty
          ? {
              for (final id in all)
                if (id != pieceId) id,
            }
          : {...muted}
      ..remove(pieceId);

// --- Per-device input mapping (change: add-drum-input-calibration) ----------

/// The **canonical** General MIDI number for a piece identity — the number the
/// app reasons in once an incoming stroke has been translated.
///
/// The first member of the piece's canonical order (36 for the kick, 38 for the
/// snare, 42 for the hi-hat…), which is the number a notation export and a
/// factory drum map overwhelmingly use. Null for an identity the table does not
/// know, so a mapping stored by a later build cannot break this one.
int? canonicalGmOfPiece(String pieceId) {
  if (pieceId == kKickPieceId) return kKickEmissionOrder.first;
  for (final piece in _namedPieces) {
    if (piece.key == pieceId) return piece.gm.first;
  }
  // A generic (terminal-bucket) piece is keyed by its own number.
  if (pieceId.startsWith('gm:')) return int.tryParse(pieceId.substring(3));
  return null;
}

/// A calibration target keyed by the General MIDI number it translates to —
/// the form [drumPieceIdOf] gives the terminal bucket, and the form a **zone**
/// of a named piece takes when the pass asks for it on its own.
///
/// A lane collapses the numbers that share one aim point, because that is what
/// the eye needs; a module does not, because each zone is a separate trigger on
/// the wire. So the pass asks at the grain the *hardware* has (design D9) while
/// everything downstream keeps reasoning at the grain the *player* has: a rim
/// zone learned here translates to 37, which [drumPieceIdOf] still calls the
/// snare, so the flash, the gate and the scorer are untouched by the extra
/// question.
String gmPieceId(int gm) => 'gm:$gm';

/// The snare's other zone: the side stick / cross-stick, played on the same
/// drum ([drumPieceIdOf] answers `kitPieceSnare` for it).
const String kCrossStickPieceId = 'gm:37';

/// The hi-hat's foot "chick" — a FOOT event, in the terminal bucket until its
/// bar encoding exists, and the one control on a kit that no hand strikes.
const String kPedalHiHatPieceId = 'gm:44';

/// The open hi-hat: the same instrument as the closed 42 (one lane, one pad),
/// a different trigger on nearly every module.
const String kOpenHiHatPieceId = 'gm:46';

/// The ride's bell — the ride lane's accent surface.
const String kRideBellPieceId = 'gm:53';

/// The standard kit a calibration pass walks, in the order it asks — round the
/// kit as a drummer sits at it: the foot and the two hands first (the pieces
/// every groove has), each with the zones that sit on the same piece of
/// hardware, then the toms high to low, then the cymbals.
///
/// The **fixed standard kit**, deliberately not the loaded score's
/// (change: add-drum-input-calibration, design D7): a mapping describes a piece
/// of hardware, not a piece of music, and calibrating from a groove that has no
/// toms would leave a kit unable to map its toms at all. Every step is
/// skippable, so a kit that lacks a piece costs one tap.
const List<String> kCalibrationKitPieceOrder = [
  kKickPieceId,
  'kitPieceSnare',
  kCrossStickPieceId,
  'kitPieceHiHat',
  kOpenHiHatPieceId,
  kPedalHiHatPieceId,
  'kitPieceTomHigh',
  'kitPieceTomHiMid',
  'kitPieceTomLowMid',
  'kitPieceTomLow',
  'kitPieceTomFloorHigh',
  'kitPieceTomFloorLow',
  'kitPieceRide',
  kRideBellPieceId,
  'kitPieceCrash',
  'kitPieceCrash2',
  'kitPieceSplash',
  'kitPieceChina',
];

/// The auxiliary pads a module offers beside the kit itself (design D9). Last,
/// and after the pass can already be finished, because they are the part of the
/// list most kits answer "none" to — a player whose kit ends at the china
/// finishes there rather than tapping through them.
const List<String> kCalibrationAuxPieceOrder = [
  'gm:56', // Cowbell
  'gm:54', // Tambourine
  'gm:39', // Hand Clap
  'gm:75', // Claves
  'gm:76', // Hi Wood Block
];

/// Every step of a calibration pass: the kit, then the auxiliary pads.
const List<String> kCalibrationPieceOrder = [
  ...kCalibrationKitPieceOrder,
  ...kCalibrationAuxPieceOrder,
];
