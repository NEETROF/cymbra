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

/// General MIDI percussion names for the generic (terminal-bucket) pieces.
const Map<int, String> _gmNames = {
  39: 'Hand Clap',
  44: 'Pedal Hi-Hat',
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
int? struckSurfaceOf(List<DrumLane> lanes, int gm, {required bool hasPedal}) {
  if (kKickGmNumbers.contains(gm)) return hasPedal ? kPedalSurface : null;
  return laneIndexOf(lanes, gm);
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
