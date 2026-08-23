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

import 'dart:math' as math;

import 'package:flutter/painting.dart' show Offset, Rect;
import 'package:freezed_annotation/freezed_annotation.dart';

part 'staff_hit_index.freezed.dart';

/// The kind of a staff symbol — one entry per glyph family the staff renderers
/// (`StaffPainter`, `PartitionPainter`) can draw. It is the stable key the help
/// content lookup and the totality test are written against, so **every glyph a
/// painter emits must map to one of these** (change: add-notation-help).
enum SymbolKind {
  note,
  rest,
  accidental,
  clef,
  keySignature,
  timeSignature,
  augmentationDot,
  stem,
  flag,
  beam,
  ledgerLine,
  barLine,
  tie,
  slur,
  tuplet,
  brace,
  dynamics,
}

/// A single staff symbol resolved under a long-press: its [kind] plus the
/// specifics the help lookup needs to pick the right explanation (for a note its
/// pitch/spelling and clef; for an accidental which one; for a clef which sign;
/// and so on). It carries no geometry — the [StaffHitIndex] pairs it with an
/// on-screen [Rect].
@freezed
sealed class SymbolDescriptor with _$SymbolDescriptor {
  const SymbolDescriptor._();

  /// A note head. [pitch] is MIDI; [diatonic] is the spelled staff step
  /// (`octave*7 + step`) when the source carries it, else null (demo/replay),
  /// and [clefSign] positions the register. These let the help name the pitch.
  /// [isGrace] marks a grace note (petite note), whose help explains the
  /// ornament instead of quoting a misleading hold duration.
  const factory SymbolDescriptor.note({
    required int pitch,
    int? diatonic,
    @Default('G') String clefSign,
    @Default(1) int staff,
    String? noteType,
    @Default(0) int dots,
    @Default(false) bool isGrace,
  }) = NoteSymbol;

  /// A rest, with its note-value type when known (`whole`/`half`/`quarter`/
  /// `eighth`/`16th`) so the help can name it and give its duration.
  const factory SymbolDescriptor.rest({String? noteType}) = RestSymbol;

  /// An accidental engraved on a note, by MusicXML token (`sharp`, `flat`,
  /// `natural`, `double-sharp`/`sharp-sharp`, `flat-flat`).
  const factory SymbolDescriptor.accidental({required String token}) =
      AccidentalSymbol;

  /// A clef, by MusicXML sign (`G`/`F`/`C`).
  const factory SymbolDescriptor.clef({required String sign}) = ClefSymbol;

  /// The key signature (armature) as a whole, by fifths (sharps > 0, flats < 0).
  const factory SymbolDescriptor.keySignature({required int fifths}) =
      KeySignatureSymbol;

  /// The time signature.
  const factory SymbolDescriptor.timeSignature({
    required int beats,
    required int beatType,
  }) = TimeSignatureSymbol;

  /// An augmentation dot next to a note or rest.
  const factory SymbolDescriptor.augmentationDot() = AugmentationDotSymbol;

  /// A note stem.
  const factory SymbolDescriptor.stem() = StemSymbol;

  /// A flag on an unbeamed eighth/sixteenth note.
  const factory SymbolDescriptor.flag() = FlagSymbol;

  /// A beam joining a run of eighth/sixteenth notes.
  const factory SymbolDescriptor.beam() = BeamSymbol;

  /// A ledger line above/below the staff.
  const factory SymbolDescriptor.ledgerLine() = LedgerLineSymbol;

  /// A bar (measure) line.
  const factory SymbolDescriptor.barLine() = BarLineSymbol;

  /// A tie joining two notes of the same pitch.
  const factory SymbolDescriptor.tie() = TieSymbol;

  /// A slur over a phrase.
  const factory SymbolDescriptor.slur() = SlurSymbol;

  /// A tuplet bracket/number (e.g. a triplet's "3").
  const factory SymbolDescriptor.tuplet({required int actual}) = TupletSymbol;

  /// The brace/bracket joining the grand staff.
  const factory SymbolDescriptor.brace() = BraceSymbol;

  /// A dynamics marking (e.g. `p`, `mf`).
  const factory SymbolDescriptor.dynamics({required String token}) =
      DynamicsSymbol;

  /// Stable kind key, for content lookup and the totality test.
  SymbolKind get kind => switch (this) {
    NoteSymbol() => SymbolKind.note,
    RestSymbol() => SymbolKind.rest,
    AccidentalSymbol() => SymbolKind.accidental,
    ClefSymbol() => SymbolKind.clef,
    KeySignatureSymbol() => SymbolKind.keySignature,
    TimeSignatureSymbol() => SymbolKind.timeSignature,
    AugmentationDotSymbol() => SymbolKind.augmentationDot,
    StemSymbol() => SymbolKind.stem,
    FlagSymbol() => SymbolKind.flag,
    BeamSymbol() => SymbolKind.beam,
    LedgerLineSymbol() => SymbolKind.ledgerLine,
    BarLineSymbol() => SymbolKind.barLine,
    TieSymbol() => SymbolKind.tie,
    SlurSymbol() => SymbolKind.slur,
    TupletSymbol() => SymbolKind.tuplet,
    BraceSymbol() => SymbolKind.brace,
    DynamicsSymbol() => SymbolKind.dynamics,
  };
}

/// One recorded glyph: its on-screen [region] and the [descriptor] describing
/// what it is.
class StaffHitEntry {
  const StaffHitEntry(this.region, this.descriptor);

  final Rect region;
  final SymbolDescriptor descriptor;
}

/// A per-frame index of every glyph a staff renderer drew, paired with its
/// on-screen rect, so a long-press location can be resolved to the symbol under
/// the finger (change: add-notation-help, D1).
///
/// The painter is handed one of these and appends to it while drawing — a
/// **side channel** that does not affect what is painted. The gesture layer then
/// [hitTest]s the finger position against it. Because glyphs are small and dense
/// and the audience is beginners, resolution is **forgiving**: a press inside a
/// glyph resolves to it; otherwise the nearest glyph within a small tolerance;
/// only genuinely empty staff area resolves to null.
class StaffHitIndex {
  final List<StaffHitEntry> _entries = <StaffHitEntry>[];

  /// The recorded entries, in draw order (earlier glyphs first).
  List<StaffHitEntry> get entries => List.unmodifiable(_entries);

  bool get isEmpty => _entries.isEmpty;
  bool get isNotEmpty => _entries.isNotEmpty;
  int get length => _entries.length;

  /// Records a glyph drawn at [region] describing [descriptor]. Empty rects are
  /// ignored (nothing to press).
  void add(Rect region, SymbolDescriptor descriptor) {
    if (region.isEmpty) return;
    _entries.add(StaffHitEntry(region, descriptor));
  }

  /// Drops all entries — called by the painter at the start of each frame so the
  /// index always reflects what is currently on screen.
  void clear() => _entries.clear();

  /// Resolves [point] to the symbol under (or nearest) it, or null when the
  /// press is in empty staff area.
  ///
  /// A press contained by one or more glyph rects picks the one whose centre is
  /// closest (so a note head wins over the wide beam it sits under). Otherwise
  /// the nearest rect whose edge distance is within [tolerance] logical pixels
  /// wins; beyond that, null.
  SymbolDescriptor? hitTest(Offset point, {double tolerance = 14}) {
    StaffHitEntry? bestContaining;
    double bestContainingD2 = double.infinity;
    StaffHitEntry? bestNear;
    double bestNearDist = double.infinity;

    for (final e in _entries) {
      if (e.region.contains(point)) {
        final d2 = (e.region.center - point).distanceSquared;
        if (d2 < bestContainingD2) {
          bestContainingD2 = d2;
          bestContaining = e;
        }
      } else {
        final dist = _edgeDistance(e.region, point);
        if (dist < bestNearDist) {
          bestNearDist = dist;
          bestNear = e;
        }
      }
    }

    if (bestContaining != null) return bestContaining.descriptor;
    if (bestNear != null && bestNearDist <= tolerance) {
      return bestNear.descriptor;
    }
    return null;
  }

  /// Shortest distance from [p] to the border/interior of [r] (0 when inside).
  static double _edgeDistance(Rect r, Offset p) {
    final dx = (p.dx < r.left)
        ? r.left - p.dx
        : (p.dx > r.right ? p.dx - r.right : 0.0);
    final dy = (p.dy < r.top)
        ? r.top - p.dy
        : (p.dy > r.bottom ? p.dy - r.bottom : 0.0);
    return math.sqrt(dx * dx + dy * dy);
  }
}
