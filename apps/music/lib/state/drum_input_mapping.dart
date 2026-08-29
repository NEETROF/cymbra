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

/// The per-device drum input mapping (change: add-drum-input-calibration):
/// what the numbers an instrument sends *mean*, learned by playing it.
///
/// The app reasons about percussion entirely in General MIDI numbers — the kit
/// model derives lanes from them, the scorer matches at the piece grain, the
/// engine echo sounds them. That is one vocabulary end to end, and it works
/// exactly as long as the instrument speaks it. An e-kit module often does not:
/// every pad is reassignable, and rim/bell/edge zones have no standard numbers
/// to be assigned to in the first place.
///
/// This file holds the whole model, pure and host-testable: a device's table,
/// the translation it performs, and the conflict rule the calibration pass
/// needs. Nothing here knows about storage, streams or screens.
library;

import 'dart:convert';

import 'drum_kit.dart';

/// One device's learned table: **piece identity → the number that device sends
/// for it**.
///
/// Stored in that direction because that is the direction it is learned in
/// ("hit your snare" records a number *for the snare*), and because it is the
/// direction a review table reads. Translation walks it the other way, through
/// an index built once per instance.
class DrumInputMapping {
  DrumInputMapping(Map<String, int> byPiece)
    : byPiece = Map.unmodifiable(byPiece),
      _toCanonical = _indexOf(byPiece);

  /// The empty mapping — an uncalibrated device. [translate] is the identity.
  static final DrumInputMapping empty = DrumInputMapping(const {});

  /// Piece identity ([drumPieceIdOf]) → the number this device sends.
  final Map<String, int> byPiece;

  /// The reverse index translation runs on: incoming number → canonical
  /// General MIDI number. Built once, so [translate] is a lookup rather than a
  /// scan, and so its result cannot depend on map iteration order.
  final Map<int, int> _toCanonical;

  bool get isEmpty => byPiece.isEmpty;
  bool get isNotEmpty => byPiece.isNotEmpty;

  static Map<int, int> _indexOf(Map<String, int> byPiece) {
    final index = <int, int>{};
    for (final entry in byPiece.entries) {
      final canonical = canonicalGmOfPiece(entry.key);
      // A piece identity this build does not know (a table written by a later
      // one) is skipped rather than throwing: an unreadable *part* of a mapping
      // degrades to uncalibrated for that piece only.
      if (canonical == null) continue;
      index[entry.value] = canonical;
    }
    return index;
  }

  /// The General MIDI number [incoming] means on this device.
  ///
  /// **Total and order-independent**: every number has an answer, an unmapped
  /// one is itself, and the answer never depends on how the table was built.
  /// The identity fallback is what keeps an uncalibrated device — and every
  /// number a calibrated device sends that nobody recorded — behaving exactly
  /// as it did before any mapping existed.
  int translate(int incoming) => _toCanonical[incoming] ?? incoming;

  /// The wire form the engine is pushed (incoming → canonical), which is the
  /// direction the MIDI callback needs and the only one it should know about.
  Map<int, int> get translationTable => Map.unmodifiable(_toCanonical);

  /// The piece [number] is already recorded for, or null when it is free.
  ///
  /// Re-recording the **same** piece is not a conflict with itself: a player
  /// striking the snare twice during the pass is correcting, not colliding.
  String? conflictFor(int number, {required String forPiece}) {
    for (final entry in byPiece.entries) {
      if (entry.value == number && entry.key != forPiece) return entry.key;
    }
    return null;
  }

  /// This mapping with [pieceId] recorded as [number].
  DrumInputMapping withPiece(String pieceId, int number) =>
      DrumInputMapping({...byPiece, pieceId: number});

  /// This mapping without [pieceId].
  DrumInputMapping withoutPiece(String pieceId) =>
      DrumInputMapping({...byPiece}..remove(pieceId));

  @override
  bool operator ==(Object other) =>
      other is DrumInputMapping &&
      other.byPiece.length == byPiece.length &&
      other.byPiece.entries.every((e) => byPiece[e.key] == e.value);

  @override
  int get hashCode => Object.hashAllUnordered([
    for (final e in byPiece.entries) Object.hash(e.key, e.value),
  ]);

  @override
  String toString() => 'DrumInputMapping($byPiece)';
}

/// Every device's mapping, keyed by MIDI **port name** (design D3): the handle
/// the app already identifies a device by (`selectPort`, the port dropdown, the
/// persisted `midiPort`), so a kit at home and a practice pad elsewhere keep
/// their own tables.
typedef DrumInputMappings = Map<String, DrumInputMapping>;

/// Serialise every device's mapping for the local preferences store.
String encodeDrumInputMappings(DrumInputMappings mappings) => jsonEncode({
  for (final entry in mappings.entries)
    if (entry.value.isNotEmpty) entry.key: entry.value.byPiece,
});

/// Read back what [encodeDrumInputMappings] wrote.
///
/// **Never throws and never guesses.** A missing, unparseable or wrongly-shaped
/// value yields no mappings at all — the uncalibrated behaviour — and a single
/// malformed device entry is dropped without taking its neighbours with it,
/// because one corrupt table must not cost a player the kit they calibrated.
DrumInputMappings decodeDrumInputMappings(String? raw) {
  if (raw == null || raw.isEmpty) return const {};
  try {
    final decoded = jsonDecode(raw);
    if (decoded is! Map) return const {};
    final result = <String, DrumInputMapping>{};
    for (final entry in decoded.entries) {
      final port = entry.key;
      final table = entry.value;
      if (port is! String || table is! Map) continue;
      final byPiece = <String, int>{};
      for (final e in table.entries) {
        final piece = e.key;
        final number = e.value;
        // A number outside the 7-bit MIDI range cannot have been sent by any
        // instrument, so it is a corrupt entry rather than an exotic one.
        if (piece is! String || number is! int || number < 0 || number > 127) {
          continue;
        }
        byPiece[piece] = number;
      }
      if (byPiece.isNotEmpty) result[port] = DrumInputMapping(byPiece);
    }
    return result;
  } catch (_) {
    return const {};
  }
}
