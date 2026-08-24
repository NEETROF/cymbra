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

import 'package:flutter/material.dart';

import '../src/rust/api/musicxml.dart';
import '../state/drum_kit.dart' show isFootEvent;
import '../state/notation_playback.dart' show clefSignLetter;
import '../state/player_data.dart' show Hand;
import '../theme/cymbra_theme.dart';
import 'notation_palette.dart';
import 'smufl.dart';
import 'staff_hit_index.dart';

/// One engraved measure's tappable rectangle, in the scrollable **content**
/// coordinates of the Partition canvas (change: add-measure-range-practice, D5).
typedef MeasureHit = ({int measure, Rect rect});

/// The measure whose rectangle contains [position], or null when the tap fell
/// outside every measure (the header, a gutter, the gap between systems). Pure
/// and host-testable — the whole of the tap-on-score picker's geometry logic.
int? measureAtPosition(List<MeasureHit> rects, Offset position) {
  for (final hit in rects) {
    if (hit.rect.contains(position)) return hit.measure;
  }
  return null;
}

/// Draws engraved notation ("Partition" mode) from a parsed [ScoreDocument] and
/// its laid-out [System]s, using SMuFL/Bravura glyphs for note heads, clefs,
/// flags, accidentals, rests and dynamics. Stems, beams, staff and ledger lines
/// are stroked at Bravura's engraving thicknesses; stem attachment uses the
/// font's note-head anchors. During playback it also draws a playhead cursor and
/// highlights the notes at the playhead (see [elapsedMs]/[measureStartMs]).
class PartitionPainter extends CustomPainter {
  final ScoreDocument document;
  final List<System> systems;

  /// Playback playhead (ms) and per-measure start times, so the cursor and note
  /// highlighting track the current position. [activeNotes] are the held MIDI
  /// pitches (a highlighted note reads "correct" when held, else "expected").
  /// With repeats, [measureStartMs] holds **played slots**; [writtenMeasureOf]
  /// (aligned, empty = identity) maps each slot back to the written measure so
  /// the cursor lands on the engraved bar being performed — jumping backward
  /// when a repeat returns.
  final double elapsedMs;
  final List<int> measureStartMs;
  final List<int> writtenMeasureOf;
  final double songEndMs;
  final Set<int> activeNotes;

  /// Which hand(s) to engrave. When a single hand is selected the unselected
  /// staff is collapsed entirely (lines, clef, signatures and notes).
  final Hand selectedHands;

  /// Visible vertical window (content coordinates) for viewport culling: only
  /// systems intersecting `[viewTop, viewBottom]` (plus a small margin) are
  /// engraved, so a long score doesn't re-draw every off-screen system on each
  /// playback frame. When either is null (previews, single-system overlays,
  /// tests) the whole score is painted — the previous behaviour.
  final double? viewTop;
  final double? viewBottom;

  /// Staff space (distance between two staff lines), in pixels. Everything else
  /// is derived from it so the engraving scales as one unit; the score-size
  /// setting passes `12 × factor` here.
  final double staffSpace;

  /// Colour set (dark surface or paper) the engraving is drawn with.
  final NotationPalette palette;

  /// Optional side channel (change: add-notation-help, D1): when supplied, each
  /// engraved symbol records its on-screen rect + [SymbolDescriptor] into it so a
  /// long-press can be resolved to the symbol under the finger. Cleared/refilled
  /// every [paint]; a null index leaves rendering byte-identical.
  final StaffHitIndex? hitIndex;

  /// The active practice measure range, highlighted on the score (change:
  /// add-measure-range-practice, D5). Null for a full run — nothing is tinted.
  final ({int start, int end})? practiceRange;

  /// Sink filled during [paint] with each engraved measure's rectangle in
  /// **content** coordinates, so the view can hit-test a tap back to a measure
  /// (the tap-on-score range picker). Only measures actually painted are
  /// recorded — culled, off-screen systems can't be tapped anyway. Null (the
  /// default) disables the collection entirely.
  final List<MeasureHit>? hitRects;

  PartitionPainter({
    required this.document,
    required this.systems,
    this.elapsedMs = 0,
    this.measureStartMs = const [],
    this.writtenMeasureOf = const [],
    this.songEndMs = 0,
    this.activeNotes = const {},
    this.selectedHands = Hand.both,
    this.viewTop,
    this.viewBottom,
    this.textFontFamily,
    this.staffSpace = 12,
    this.palette = NotationPalette.dark,
    this.hitIndex,
    this.practiceRange,
    this.hitRects,
  });

  /// Font family for the engraved *words* (tempo marks, lyrics, fingerings) —
  /// the music glyphs come from [Smufl] regardless. Null keeps the platform
  /// default, which is what the app ships; tests name a face so their goldens
  /// show words instead of the framework's box glyphs.
  final String? textFontFamily;

  /// Records an engraved [descriptor] at [region] into the hit index, if one is
  /// attached. A no-op otherwise, so drawing is unaffected.
  void _record(Rect region, SymbolDescriptor descriptor) =>
      hitIndex?.add(region, descriptor);

  static const Map<String, int> _semitoneOfStep = {
    'C': 0,
    'D': 2,
    'E': 4,
    'F': 5,
    'G': 7,
    'A': 9,
    'B': 11,
  };

  int _midiOf(Pitch p) =>
      (p.octave + 1) * 12 + (_semitoneOfStep[p.step] ?? 0) + p.alter;

  /// The measure index containing the playhead and the fraction within it, or
  /// null when there is no timing (demo score) or the playhead is out of range.
  ({int index, double fraction})? get _cursor {
    final starts = measureStartMs;
    if (starts.isEmpty || elapsedMs < starts.first) return null;
    for (var i = 0; i < starts.length; i++) {
      final start = starts[i];
      final end = (i + 1 < starts.length ? starts[i + 1] : songEndMs)
          .toDouble();
      if (elapsedMs >= start && elapsedMs < end) {
        final span = end - start;
        final frac = span > 0
            ? ((elapsedMs - start) / span).clamp(0.0, 1.0)
            : 0.0;
        // Map the played slot to the written measure it performs, so the
        // cursor (and the current-measure wash) land on the engraved bar.
        final written = i < writtenMeasureOf.length ? writtenMeasureOf[i] : i;
        return (index: written, fraction: frac);
      }
    }
    return null;
  }

  /// Shorthand for [staffSpace] — every other metric derives from it.
  double get _s => staffSpace;
  double get _staffHeight => 4 * _s;
  double get _interStaff => 8 * _s; // treble bottom → bass top
  double get _topPad => 5 * _s; // words/dynamics above
  /// Room under the bass staff: the full lyrics lane when the piece carries
  /// lyrics, otherwise just ledger-line clearance — an instrumental piece
  /// paying for an empty lyrics lane on every system is what pushed the next
  /// line's bass staff off short viewports.
  double get _bottomPad => (_hasLyrics ? 4.5 : 3.0) * _s;
  double get _systemGap => 2.5 * _s;
  static const double _stemLen = 3.5; // staff spaces

  late final bool _hasLyrics = document.measures.any(
    (m) => m.notes.any((n) => n.lyric != null),
  );

  /// The clef a staff carries when the score declares none: a percussion part
  /// is always on the percussion clef, a second staff on F, the first on G.
  Clef _defaultClefFor(int staff) {
    if (_isPercussion) {
      return const Clef(staff: 1, sign: ClefSign.percussion, line: 2);
    }
    if (staff >= 2) return const Clef(staff: 2, sign: ClefSign.f, line: 4);
    return const Clef(staff: 1, sign: ClefSign.g, line: 2);
  }

  /// Vertical nudge of a rest on a two-voice percussion staff: the feet's
  /// rests sit below the middle line, the hands' above, so the two voices stay
  /// readable apart. Zero everywhere else.
  double _percRestOffset(bool percTwoVoice, int voice) {
    if (!percTwoVoice) return 0;
    return voice >= 2 ? _s : -_s;
  }

  /// The head colour of a note the playhead sits on: the success colour while
  /// it is actually held, the hand's own colour lifted toward the emphasis
  /// tint otherwise.
  Color _playheadHeadColor(int soundId, Color handColor) =>
      activeNotes.contains(soundId)
      ? palette.correct
      : Color.lerp(handColor, palette.emphasisTint, 0.55)!;

  /// Stem direction: the score's own when it declares one; otherwise the voice
  /// on a percussion staff (hands up, feet down) and the head's side of the
  /// middle line anywhere else.
  bool _stemUp(NoteEvent note, double y, double midY) {
    if (note.stem != null) return note.stem == StemDir.up;
    return _isPercussion ? note.voice < 2 : y >= midY;
  }

  /// Whether the document is a percussion score — every non-rest note
  /// unpitched, and at least one (change: add-drum-notation-render). Mirrors
  /// the crate's classification, so the painter routes to the percussion
  /// engraving rules exactly when the schedule and the player do.
  late final bool _isPercussion =
      document.measures.any((m) => m.notes.any((n) => n.unpitched != null)) &&
      !document.measures.any((m) => m.notes.any((n) => n.pitch != null));

  /// Whether the percussion part spans more than one voice — the hands/feet
  /// split then keys on the voice; a single-voice export falls back to the
  /// General MIDI numbers (see `isFootEvent`).
  late final bool _percMultiVoice = () {
    int? seen;
    for (final m in document.measures) {
      for (final n in m.notes) {
        if (n.isRest) continue;
        seen ??= n.voice;
        if (n.voice != seen) return true;
      }
    }
    return false;
  }();

  static const Map<String, int> _stepOrder = {
    'C': 0,
    'D': 1,
    'E': 2,
    'F': 3,
    'G': 4,
    'A': 5,
    'B': 6,
  };

  Color get _ink => palette.ink;

  /// Whether [staff]'s glyphs are drawn for the current [selectedHands]
  /// (staff 1 = right hand, staff 2+ = left hand) — the visibility predicate.
  ///
  /// The staff-collapse rule is **scoped to keyboard scores**: a percussion
  /// part has exactly one staff, so its lines, clef and time signature are
  /// always drawn and the hand filter hides **events** instead (see
  /// [_showsPercussionEvent]) — collapsing "the unselected hand's staff"
  /// would erase the only staff there is.
  bool _showsStaff(int staff) =>
      _isPercussion ||
      switch (selectedHands) {
        Hand.both => true,
        Hand.right => staff == 1,
        Hand.left => staff >= 2,
      };

  /// Whether the hand filter shows this percussion event (change:
  /// add-drum-notation-render): notes split hands/feet by the voice
  /// convention with the single-voice GM fallback; a multi-voice part's rests
  /// follow their voice, while a single-voice part's rests belong to the
  /// shared groove and stay visible either way.
  bool _showsPercussionEvent(NoteEvent note) {
    if (selectedHands == Hand.both) return true;
    if (note.isRest) {
      return !_percMultiVoice ||
          (selectedHands == Hand.right ? note.voice < 2 : note.voice >= 2);
    }
    final foot = isFootEvent(
      voice: note.voice,
      gmNumber: note.unpitched?.gmNumber,
      multiVoice: _percMultiVoice,
    );
    return selectedHands == Hand.right ? !foot : foot;
  }

  /// A grand staff is drawn only when both hands are shown; selecting a single
  /// hand collapses the other staff and lays out one staff per system. A
  /// percussion score always lays out its single staff.
  bool get _twoStaff =>
      !_isPercussion && document.staves >= 2 && selectedHands == Hand.both;

  /// The kept staff when a single hand is shown — its clef/armature/time sit on
  /// the lone staff (bass for the left hand, treble otherwise; always the one
  /// percussion staff on a drum part).
  int get _soloStaff => !_isPercussion && selectedHands == Hand.left ? 2 : 1;

  double get _systemHeight =>
      _topPad +
      _staffHeight +
      (_twoStaff ? _interStaff + _staffHeight : 0) +
      _bottomPad;

  double heightFor(double width) =>
      systems.length * (_systemHeight + _systemGap) + _systemGap;

  /// Vertical distance between consecutive system tops (matches `paint`).
  double get systemStride => _systemHeight + _systemGap;

  /// Height of one engraved system without the inter-system gap — the span
  /// that must be on screen for a line to be fully readable.
  double get systemHeight => _systemHeight;

  /// Padding above a system's first staff line (words/dynamics lane) — the
  /// part the auto-scroll may sacrifice before cutting actual staves.
  double get systemTopPad => _topPad;

  /// Y of the top of system [index] in the scrollable content (matches `paint`).
  double systemTopY(int index) => _systemGap + index * systemStride;

  int _divisionsPerMeasure() {
    final a = document.attributes;
    final beatType = a.time.beatType == 0 ? 4 : a.time.beatType;
    final perMeasure = a.divisions * a.time.beats * 4 ~/ beatType;
    return perMeasure > 0 ? perMeasure : a.divisions * 4;
  }

  /// Opacity applied to systems the playhead has fully passed, so the current
  /// and upcoming lines stand out (`played-system dimming`).
  static const double _playedDim = 0.45;

  /// Index of the system containing measure [measureIndex], or null.
  int? _systemIndexOf(int measureIndex) {
    for (var i = 0; i < systems.length; i++) {
      if (systems[i].measures.contains(measureIndex)) return i;
    }
    return null;
  }

  @override
  void paint(Canvas canvas, Size size) {
    // Refill the hit index from scratch every frame (side channel; no effect on
    // what is drawn).
    hitIndex?.clear();
    if (systems.isEmpty) return;
    // Each frame re-derives the hit geometry from what is actually engraved, so
    // it can never go stale against a re-layout or a scroll-driven cull.
    hitRects?.clear();
    final divPerMeasure = _divisionsPerMeasure();
    final clefAt = _computeClefAt();
    final cursor = _cursor;
    // Systems before the one holding the playhead render dimmed; without a
    // playhead (stopped/untimed) every line keeps full opacity.
    final cursorSystem = cursor == null ? null : _systemIndexOf(cursor.index);
    // Cull to the visible window when one is supplied, keeping one extra system
    // of margin on each side so scrolling never reveals an unpainted line.
    final vt = viewTop, vb = viewBottom;
    final cull = vt != null && vb != null;
    final margin = systemStride;
    var y = _systemGap;
    for (var i = 0; i < systems.length; i++) {
      final onScreen =
          !cull || (y + _systemHeight >= vt - margin && y <= vb + margin);
      if (onScreen) {
        final dimmed = cursorSystem != null && i < cursorSystem;
        if (dimmed) {
          // The layer's paint alpha fades the whole engraved line at once.
          canvas.saveLayer(
            Rect.fromLTWH(0, y - _systemGap / 2, size.width, systemStride),
            Paint()
              ..color = const Color(0xFFFFFFFF).withValues(alpha: _playedDim),
          );
        }
        _paintSystem(
          canvas,
          systems[i],
          size.width,
          y,
          divPerMeasure,
          i == 0,
          clefAt,
          cursor,
        );
        if (dimmed) canvas.restore();
      }
      y += _systemHeight + _systemGap;
    }
  }

  /// Clef in effect per staff for each measure index, honouring mid-piece clef
  /// changes (a measure's `clefs` override the running clef from its start).
  List<Map<int, Clef>> _computeClefAt() {
    final running = <int, Clef>{};
    for (final c in document.attributes.clefs) {
      running[c.staff] = c;
    }
    final out = <Map<int, Clef>>[];
    for (final m in document.measures) {
      for (final c in m.clefs) {
        running[c.staff] = c;
      }
      out.add(Map<int, Clef>.from(running));
    }
    return out;
  }

  Clef _clefFor(Map<int, Clef> clefs, int staff) =>
      clefs[staff] ??
      // A clef-less percussion export still gets the percussion clef — the
      // treble default below would silently draw a G clef on a drum staff.
      _defaultClefFor(staff);

  void _paintSystem(
    Canvas canvas,
    System system,
    double width,
    double yTop,
    int divPerMeasure,
    bool isFirst,
    List<Map<int, Clef>> clefAt,
    ({int index, double fraction})? cursor,
  ) {
    final trebleBottom = yTop + _topPad + _staffHeight;
    final bassBottom = trebleBottom + _interStaff + _staffHeight;
    final systemBottom = _twoStaff ? bassBottom : trebleBottom;
    final headerClefs = clefAt[system.measures.first];
    final words = _TextLanes(_s * 1.3);
    final arcs = _Arcs();

    final linePaint = Paint()
      ..color = palette.staffLine.withValues(alpha: 0.7)
      ..strokeWidth = Smufl.staffLineThickness * _s;
    final barPaint = Paint()
      ..color = _ink.withValues(alpha: 0.7)
      ..strokeWidth = Smufl.thinBarlineThickness * _s;

    _drawStaffLines(canvas, trebleBottom, width, linePaint);
    if (_twoStaff) _drawStaffLines(canvas, bassBottom, width, linePaint);

    final systemTop = trebleBottom - _staffHeight;
    // Left system bracket connecting the grand staff (a single brace glyph does
    // not stretch cleanly, so a thick rounded bar is used instead).
    canvas.drawLine(
      Offset(1.2, systemTop),
      Offset(1.2, systemBottom),
      Paint()
        ..color = _ink.withValues(alpha: 0.8)
        ..strokeWidth = _s * 0.35
        ..strokeCap = StrokeCap.round,
    );
    if (_twoStaff) {
      _record(
        Rect.fromLTRB(0, systemTop, _s * 0.8, systemBottom),
        const SymbolDescriptor.brace(),
      );
    }

    // --- Header: clef (in effect here), key signature, time signature. On a
    // collapsed single staff the lone staff carries the kept hand's clef. ---
    final topStaff = _twoStaff ? 1 : _soloStaff;
    _drawClef(
      canvas,
      _clefFor(headerClefs, topStaff),
      _s * 0.4,
      trebleBottom,
      _s,
    );
    if (_twoStaff) {
      _drawClef(canvas, _clefFor(headerClefs, 2), _s * 0.4, bassBottom, _s);
    }
    var hx = _s * 3.0; // after the clef

    // Armure of THIS system: the key in force at its first measure, not one
    // document-wide value — so a piece that modulates shows the right signature
    // on every line. When the key changes at this system's boundary, the header
    // shows the change (cancelling naturals + the new signature).
    final firstIdx = system.measures.first;
    final systemKey = document.measures[firstIdx].keyFifths;
    final prevKey = firstIdx > 0
        ? document.measures[firstIdx - 1].keyFifths
        : null;
    final headerKeyChanged = prevKey != null && prevKey != systemKey;
    // No armature on a percussion staff — nor key-change naturals (change:
    // add-drum-notation-render): a declared `fifths` is an exporter leftover,
    // and engraving sharps or flats on a drum staff is wrong music.
    double drawHeaderKey(double staffBottom, bool bass) => _isPercussion
        ? 0.0
        : headerKeyChanged
        ? Smufl.drawKeyChange(
            canvas,
            hx,
            staffBottom,
            _s,
            prevKey,
            systemKey,
            bass,
            _ink,
          )
        : Smufl.drawKeySignature(
            canvas,
            hx,
            staffBottom,
            _s,
            systemKey,
            bass,
            _ink,
          );
    final keyWidth = drawHeaderKey(trebleBottom, topStaff >= 2);
    if (_twoStaff) {
      drawHeaderKey(bassBottom, true);
    }
    if (keyWidth > 0) {
      _record(
        Rect.fromLTRB(hx, systemTop, hx + keyWidth, systemBottom),
        SymbolDescriptor.keySignature(fifths: systemKey),
      );
    }
    hx += keyWidth;

    if (isFirst) {
      final time = document.attributes.time;
      final timeWidth = Smufl.drawTimeSignature(
        canvas,
        hx,
        trebleBottom,
        _s,
        time.beats,
        time.beatType,
        _ink,
      );
      if (_twoStaff) {
        Smufl.drawTimeSignature(
          canvas,
          hx,
          bassBottom,
          _s,
          time.beats,
          time.beatType,
          _ink,
        );
      }
      _record(
        Rect.fromLTRB(hx, systemTop, hx + timeWidth, systemBottom),
        SymbolDescriptor.timeSignature(
          beats: time.beats,
          beatType: time.beatType,
        ),
      );
      hx += timeWidth;
    }
    final headerX = hx + _s * 0.6;

    // Measures justified to fill the line width after the header.
    final indices = system.measures;
    var totalMin = 0.0;
    for (final idx in indices) {
      totalMin += document.measures[idx].minWidth;
    }
    final usable = width - headerX;
    final scale = totalMin > 0 ? usable / totalMin : 1.0;

    var x = headerX;
    double? cursorX; // set when the active measure is in this system
    for (var k = 0; k < indices.length; k++) {
      final idx = indices[k];
      final measure = document.measures[idx];
      final mWidth = measure.minWidth * scale;
      final bounds = Rect.fromLTRB(x, systemTop, x + mWidth, systemBottom);
      hitRects?.add((measure: idx, rect: bounds));
      // Tint the measures of the active practice range, so the selection is
      // visible on the score itself.
      final range = practiceRange;
      if (range != null && idx >= range.start && idx <= range.end) {
        canvas.drawRect(
          bounds,
          Paint()..color = CymbraColors.tertiary.withValues(alpha: 0.14),
        );
      }
      canvas.drawLine(
        Offset(x + mWidth, systemTop),
        Offset(x + mWidth, systemBottom),
        barPaint,
      );
      _record(
        Rect.fromLTRB(
          x + mWidth - _s * 0.4,
          systemTop,
          x + mWidth + _s * 0.4,
          systemBottom,
        ),
        const SymbolDescriptor.barLine(),
      );
      _drawMeasureRepeats(
        canvas,
        measure,
        x,
        mWidth,
        systemTop,
        systemBottom,
        trebleBottom,
        bassBottom,
      );
      final isCursorMeasure = cursor != null && cursor.index == idx;
      // Subtle wash behind the active measure (current-measure highlight): over
      // the staff lines but under the glyphs, and only while a playhead exists.
      if (isCursorMeasure) {
        canvas.drawRect(
          Rect.fromLTRB(x, systemTop - _s, x + mWidth, systemBottom + _s),
          Paint()..color = palette.accent.withValues(alpha: 0.08),
        );
      }
      // The cursor x comes back from _paintMeasure, which is the only place
      // that knows the measure's note columns — a clef/key change shifts them,
      // and they stop short of the closing bar. Placing the cursor by the raw
      // measure fraction instead put it beside the note it points at.
      final cx = _paintMeasure(
        canvas,
        measure,
        x,
        mWidth,
        divPerMeasure,
        trebleBottom,
        bassBottom,
        clefAt[idx],
        k == 0, // first measure of the system → clef already in the header
        k > 0 ? document.measures[indices[k - 1]].keyFifths : null,
        words,
        arcs,
        isCursorMeasure ? cursor.fraction * divPerMeasure : null,
      );
      if (cx != null) cursorX = cx;
      x += mWidth;
    }

    // Playhead cursor, drawn over the system's staves.
    if (cursorX != null) {
      canvas.drawLine(
        Offset(cursorX, systemTop),
        Offset(cursorX, systemBottom),
        Paint()
          ..color = palette.accent
          ..strokeWidth = _s * 0.18
          ..strokeCap = StrokeCap.round,
      );
    }
  }

  /// Jump instructions engraved as words (D.C./D.S./Fine/To Coda), matched
  /// case-insensitively so the long-press help can explain them.
  static final RegExp _jumpWords = RegExp(
    r'\b(d\.?\s?c\.?|d\.?\s?s\.?|da capo|dal segno|fine|to coda|al coda)\b',
    caseSensitive: false,
  );

  /// Engraves the repeat notation of one measure (change: add-repeat-unrolling):
  /// repeat barlines (thick line + dots on the repeated side), the volta
  /// bracket + number, the measure-repeat `%` sign, and segno/coda signs —
  /// each recorded for the long-press help.
  void _drawMeasureRepeats(
    Canvas canvas,
    NotationMeasure measure,
    double x,
    double mWidth,
    double systemTop,
    double systemBottom,
    double trebleBottom,
    double bassBottom,
  ) {
    final r = measure.repeats;
    final thick = Paint()
      ..color = _ink
      ..strokeWidth = Smufl.thickBarlineThickness * _s;
    final thin = Paint()
      ..color = _ink.withValues(alpha: 0.85)
      ..strokeWidth = Smufl.thinBarlineThickness * _s * 1.3;

    void dots(double dx) {
      for (final base in [trebleBottom, if (_twoStaff) bassBottom]) {
        for (final dy in const [1.5, 2.5]) {
          canvas.drawCircle(
            Offset(dx, base - dy * _s),
            _s * 0.22,
            Paint()..color = _ink,
          );
        }
      }
    }

    if (r.forward) {
      // ‖: at the measure's left edge.
      canvas.drawLine(
        Offset(x + _s * 0.18, systemTop),
        Offset(x + _s * 0.18, systemBottom),
        thick,
      );
      dots(x + _s * 0.95);
      _record(
        Rect.fromLTRB(x - _s * 0.4, systemTop, x + _s * 1.4, systemBottom),
        const SymbolDescriptor.repeatBarline(forward: true),
      );
    }
    if (r.backwardTimes > 0) {
      // :‖ at the measure's right edge.
      final bx = x + mWidth;
      canvas.drawLine(
        Offset(bx - _s * 0.18, systemTop),
        Offset(bx - _s * 0.18, systemBottom),
        thick,
      );
      dots(bx - _s * 0.95);
      _record(
        Rect.fromLTRB(bx - _s * 1.4, systemTop, bx + _s * 0.4, systemBottom),
        const SymbolDescriptor.repeatBarline(forward: false),
      );
    }
    if (r.endingStart.isNotEmpty || r.endingStop || r.endingDiscontinue) {
      // Volta bracket segment over this measure; label on the start measure.
      final y = systemTop - _s * 1.6;
      canvas.drawLine(Offset(x, y), Offset(x + mWidth, y), thin);
      if (r.endingStart.isNotEmpty) {
        canvas.drawLine(Offset(x, y), Offset(x, y + _s * 1.1), thin);
        final label = '${r.endingStart.join('.')}.';
        _text(canvas, label, x + _s * 0.4, y + _s * 0.2, color: _ink);
        _record(
          Rect.fromLTRB(x, y - _s, x + mWidth, y + _s * 1.4),
          SymbolDescriptor.volta(label: label),
        );
      }
      if (r.endingStop) {
        canvas.drawLine(
          Offset(x + mWidth, y),
          Offset(x + mWidth, y + _s * 1.1),
          thin,
        );
      }
    }
    if (r.measureRepeatOf != null) {
      final cx = x + mWidth / 2;
      final cy = trebleBottom - 2 * _s;
      Smufl.draw(
        canvas,
        r.measureRepeatSlashes >= 2 ? Smufl.repeat2Bars : Smufl.repeat1Bar,
        cx,
        cy,
        _s,
        _ink,
        centerX: true,
      );
      _record(
        Rect.fromCenter(
          center: Offset(cx, cy),
          width: _s * 2.6,
          height: _s * 2.6,
        ),
        const SymbolDescriptor.measureRepeat(),
      );
    }
    if (r.segno || r.coda) {
      final gx = x + _s * 1.2;
      final gy = systemTop - _s * 1.8;
      Smufl.draw(
        canvas,
        r.segno ? Smufl.segno : Smufl.coda,
        gx,
        gy,
        _s * 1.1,
        _ink,
        centerX: true,
      );
      _record(
        Rect.fromCenter(
          center: Offset(gx, gy - _s * 0.7),
          width: _s * 2.2,
          height: _s * 2.6,
        ),
        r.segno
            ? const SymbolDescriptor.segno()
            : const SymbolDescriptor.coda(),
      );
    }
  }

  /// Draws a clef glyph for [clef] on the staff whose bottom line is at
  /// [staffBottom] (the clef sign sits on its `line`).
  void _drawClef(
    Canvas canvas,
    Clef clef,
    double x,
    double staffBottom,
    double size,
  ) {
    // G/F/C clefs sit on their declared `line`; the percussion clef is centred
    // on the middle line whatever line the file declares (its SMuFL origin is
    // the glyph's vertical centre).
    final baselineY = clef.sign == ClefSign.percussion
        ? staffBottom - 2 * size
        : staffBottom - (clef.line - 1) * size;
    Smufl.draw(
      canvas,
      Smufl.clef(clefSignLetter(clef.sign)),
      x,
      baselineY,
      size,
      _ink,
    );
    _record(
      Rect.fromLTWH(x - 2, staffBottom - 4 * size - size, size * 3.2, size * 6),
      SymbolDescriptor.clef(sign: clefSignLetter(clef.sign)),
    );
  }

  void _drawStaffLines(
    Canvas canvas,
    double bottom,
    double width,
    Paint paint,
  ) {
    for (var i = 0; i < 5; i++) {
      final y = bottom - i * _s;
      canvas.drawLine(Offset(0, y), Offset(width, y), paint);
    }
  }

  /// Engraves one measure and returns the playhead's x inside it when
  /// [cursorDiv] falls in this measure (null otherwise) — mapped through the
  /// same note-column arithmetic as the heads, so the cursor lands *on* the
  /// note it points at rather than beside it.
  double? _paintMeasure(
    Canvas canvas,
    NotationMeasure measure,
    double measureX,
    double measureWidth,
    int divPerMeasure,
    double trebleBottom,
    double bassBottom,
    Map<int, Clef> clefs,
    bool isSystemFirst,
    int? previousKeyFifths,
    _TextLanes words,
    _Arcs arcs,
    double? cursorDiv,
  ) {
    // A mid-system clef change is drawn at the measure start and reserves space
    // — only for staves that are shown (a collapsed staff draws nothing).
    final showClefChange =
        !isSystemFirst && measure.clefs.any((c) => _showsStaff(c.staff));
    final clefLead = showClefChange ? _s * 2.6 : 0.0;
    if (showClefChange) {
      for (final c in measure.clefs) {
        if (!_showsStaff(c.staff)) continue;
        final sb = c.staff >= 2 && _twoStaff ? bassBottom : trebleBottom;
        _drawClef(canvas, c, measureX + _s * 0.3, sb, _s * 0.9);
      }
    }

    // A mid-system key change (modulation): cancelling naturals + the new
    // signature, after any clef change, reserving space before the first note.
    // Never on a percussion staff — no armature means no key-change naturals
    // either, even when the file declares a `fifths` change.
    final keyChanged =
        !_isPercussion &&
        !isSystemFirst &&
        previousKeyFifths != null &&
        measure.keyFifths != previousKeyFifths;
    var keyLead = 0.0;
    if (keyChanged) {
      final kx = measureX + _s * 0.3 + clefLead;
      final soloBass = !_twoStaff && _soloStaff >= 2;
      keyLead = Smufl.drawKeyChange(
        canvas,
        kx,
        trebleBottom,
        _s,
        previousKeyFifths,
        measure.keyFifths,
        _twoStaff ? false : soloBass,
        _ink,
      );
      if (_twoStaff) {
        Smufl.drawKeyChange(
          canvas,
          kx,
          bassBottom,
          _s,
          previousKeyFifths,
          measure.keyFifths,
          true,
          _ink,
        );
      }
    }
    final lead = clefLead + keyLead;

    double xForPosition(num position) {
      final frac = divPerMeasure > 0
          ? (position / divPerMeasure).clamp(0.0, 1.0)
          : 0.0;
      final left = measureX + _s + lead;
      return left + frac * (measureWidth - lead - 2.4 * _s);
    }

    // The playhead rides the same column mapping as the note heads.
    final cursorX = cursorDiv == null ? null : xForPosition(cursorDiv);

    for (final dir in measure.directions) {
      final x = xForPosition(dir.positionDivisions);
      switch (dir.kind) {
        case DirectionKind_Words(:final field0):
          // Stack overlapping words onto separate rows above the staff.
          final w = _textWidth(field0, _s * 1.05, italic: true);
          final baseY = trebleBottom - _staffHeight - _s * 1.8;
          final y = words.yFor(x, w + _s * 0.6, baseY);
          _text(canvas, field0, x, y, italic: true, color: palette.staffLine);
          // Jump instructions (D.C./D.S./Fine/To Coda) get a help entry: a
          // beginner meeting "D.C. al Fine" deserves an explanation.
          if (_jumpWords.hasMatch(field0)) {
            _record(
              Rect.fromLTWH(x, y - _s * 1.2, w + _s * 0.6, _s * 2.0),
              SymbolDescriptor.jump(words: field0),
            );
          }
        case DirectionKind_Dynamics(:final field0):
          // Dynamics sit a little below note-head size (≈ 0.78 staff spaces).
          Smufl.draw(
            canvas,
            Smufl.dynamics(field0),
            x,
            trebleBottom + _s * 2.2,
            _s * 0.78,
            palette.accent,
          );
          _record(
            Rect.fromLTWH(
              x - _s * 0.3,
              trebleBottom + _s * 1.3,
              _s * 2.2,
              _s * 1.6,
            ),
            SymbolDescriptor.dynamics(token: field0),
          );
        case DirectionKind_Wedge():
        case DirectionKind_Metronome():
          break;
      }
    }

    final beamGroups = <String, List<_Note>>{};
    final openTuplets = <String, _TupletAcc>{};

    // Percussion (change: add-drum-notation-render): whether this measure
    // engraves both voices — the per-voice rest displacement applies only
    // then — and the head columns already engraved, for the shared-onset
    // offsetting rule.
    final percTwoVoice =
        _isPercussion && {for (final n in measure.notes) n.voice}.length > 1;
    final drawnHeads = <(num, int)>{};

    for (final note in measure.notes) {
      // Collapse the unselected hand: skip every glyph on a hidden staff.
      if (!_showsStaff(note.staff)) continue;
      // A percussion score never collapses its lone staff — the hand filter
      // hides events by the hands/feet voice convention instead.
      if (_isPercussion && !_showsPercussionEvent(note)) continue;
      final isBass = note.staff >= 2 && _twoStaff;
      final staffBottom = isBass ? bassBottom : trebleBottom;
      // A grace note shares its principal's position (it occupies no musical
      // time): engrave it smaller, offset left of the principal's column,
      // instead of drawing both full-size on the same spot.
      final glyphScale = note.isGrace ? 0.7 : 1.0;
      var x = xForPosition(note.positionDivisions);
      if (note.isGrace) x -= Smufl.noteheadWidth * _s * 1.4;

      if (note.isRest) {
        // In a two-voice percussion measure a rest is displaced by voice —
        // voice 1 above the middle line, voice 2 below — clear of the other
        // voice's material; single-voice measures keep the midline.
        final restY =
            staffBottom - 2 * _s + _percRestOffset(percTwoVoice, note.voice);
        Smufl.draw(canvas, _restGlyph(note), x, restY, _s, _ink, centerX: true);
        _record(
          Rect.fromCenter(
            center: Offset(x, restY - 0.5 * _s),
            width: _s * 1.6,
            height: _s * 2.4,
          ),
          SymbolDescriptor.rest(noteType: note.noteType),
        );
        continue;
      }
      final pitch = note.pitch;
      final unpitched = note.unpitched;
      if (pitch == null && unpitched == null) continue;

      final clef = _clefFor(clefs, note.staff);
      // An unpitched note is placed by its WRITTEN position — display step and
      // octave under the treble mapping — never by the General MIDI number
      // (a sound identity, not a position); a note whose number is unresolved
      // still engraves at its written position.
      final diatonic = pitch != null
          ? pitch.octave * 7 + (_stepOrder[pitch.step] ?? 0)
          : unpitched!.displayOctave * 7 +
                (_stepOrder[unpitched.displayStep] ?? 0);
      final y = staffBottom - (diatonic - _clefBottomDiatonic(clef)) * (_s / 2);
      // Shared onsets: when two voices strike the same instant on the same
      // staff position, the second head steps right so neither is hidden.
      if (_isPercussion &&
          !drawnHeads.add((note.positionDivisions, diatonic))) {
        x += Smufl.noteheadWidth * _s * 1.1;
      }
      _drawLedgerLines(canvas, x, y, staffBottom);
      // Ledger lines are drawn when the head sits off the staff; record the gap
      // so a press on one is explained rather than swallowed.
      if (y > staffBottom + _s * 0.5 || y < staffBottom - 4 * _s - _s * 0.5) {
        final edge = y > staffBottom ? staffBottom : staffBottom - 4 * _s;
        _record(
          Rect.fromLTRB(
            x - _s * 1.1,
            y < edge ? y : edge,
            x + _s * 1.1,
            y < edge ? edge : y,
          ),
          const SymbolDescriptor.ledgerLine(),
        );
      }

      // Note heads are coloured by hand (right = blue, left = amber); on a
      // percussion score by hands/feet — split by the voice convention with
      // the single-voice GM fallback (`hand-color-coding`) — so a kick at F4
      // carries the same amber its bar carries in the cascade. The note at
      // the playhead is emphasised: green once its pitch is held ("correct"),
      // otherwise a brighter tint of its hand colour.
      final isLeft = _isPercussion
          ? isFootEvent(
              voice: note.voice,
              gmNumber: unpitched?.gmNumber,
              multiVoice: _percMultiVoice,
            )
          : note.staff >= 2;
      final handColor = isLeft ? palette.handLeft : palette.handRight;
      final soundId = pitch != null
          ? _midiOf(pitch)
          : (unpitched!.gmNumber ?? -1);
      final isAtPlayhead =
          cursorDiv != null &&
          note.positionDivisions <= cursorDiv &&
          cursorDiv < note.positionDivisions + note.durationDivisions;
      final headColor = isAtPlayhead
          ? _playheadHeadColor(soundId, handColor)
          : handColor;

      // Note head, centred on x.
      final headLeft = x - Smufl.noteheadWidth * _s * glyphScale / 2;
      Smufl.draw(
        canvas,
        _headGlyph(note),
        headLeft,
        y,
        _s * glyphScale,
        headColor,
      );
      // The conventional open mark on an open hi-hat stroke (GM 46, the
      // bridged xOpen class): a small circle above the x head, keeping the
      // open/closed distinction readable as it is in the cascade.
      if (unpitched?.headClass == HeadClass.xOpen) {
        canvas.drawCircle(
          Offset(x, y - _s * 1.6),
          _s * 0.32,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = Smufl.stemThickness * _s * 1.4
            ..color = headColor,
        );
      }
      _record(
        Rect.fromCenter(
          center: Offset(x, y),
          width: Smufl.noteheadWidth * _s * 1.5,
          height: _s * 1.7,
        ),
        SymbolDescriptor.note(
          pitch: soundId,
          diatonic: diatonic,
          clefSign: clefSignLetter(clef.sign),
          staff: note.staff,
          noteType: note.noteType,
          dots: note.dots,
          isGrace: note.isGrace,
        ),
      );

      if (note.accidental != null) {
        final glyph = Smufl.accidental(note.accidental!);
        if (glyph != null) {
          Smufl.draw(canvas, glyph, headLeft - _s * 1.5, y, _s, _ink);
          _record(
            Rect.fromLTWH(
              headLeft - _s * 1.7,
              y - _s * 1.4,
              _s * 1.4,
              _s * 2.6,
            ),
            SymbolDescriptor.accidental(token: note.accidental!),
          );
        }
      }
      _drawDots(canvas, x, y, note.dots);
      if (note.dots > 0) {
        _record(
          Rect.fromLTWH(
            x + Smufl.noteheadWidth * _s / 2,
            y - _s * 0.6,
            _s * (0.4 + note.dots * 0.5),
            _s * 1.2,
          ),
          const SymbolDescriptor.augmentationDot(),
        );
      }

      // Ties (same-pitch) and slurs (phrase), connecting to a stored start.
      // An unpitched tie keys on the resolved sound + written position — the
      // distinct key space keeps it from ever pairing with a pitched chain.
      final headR = Smufl.noteheadWidth * _s / 2;
      final tieKey = pitch != null
          ? '${note.staff}_${note.voice}_${pitch.step}'
                '${pitch.octave}_${pitch.alter}'
          : '${note.staff}_${note.voice}_u${unpitched!.gmNumber}_$diatonic';
      if (note.tieStop) {
        final start = arcs.takeTie(tieKey);
        if (start != null) _drawTie(canvas, start, Offset(x - headR, y));
      }
      if (note.tieStart) arcs.putTie(tieKey, Offset(x + headR, y));

      final slurKey = '${note.staff}_${note.voice}';
      arcs.observeSlur(slurKey, y); // track the phrase's highest note
      if (note.slurStop) {
        final s = arcs.popSlur(slurKey);
        if (s != null) _drawSlur(canvas, s.start, Offset(x, y), s.minY);
      }
      if (note.slurStart) arcs.pushSlur(slurKey, Offset(x, y));

      // Stem + beam grouping (chord members share the principal note's stem).
      // Whole-figure heads (oval or x form) carry no stem. Stems follow the
      // file's explicit <stem> first; a bare percussion note defaults to the
      // voice convention — voice 1 (hands) up, voice 2 (feet) down.
      final headGlyph = _headGlyph(note);
      if (headGlyph != Smufl.noteheadWhole &&
          headGlyph != Smufl.noteheadXWhole &&
          !note.isChord) {
        final midY = staffBottom - 2 * _s;
        final up = _stemUp(note, y, midY);
        final n = _Note(x, y, up, note, glyphScale);
        if (note.beams.isEmpty) {
          _drawStemAndFlag(canvas, n);
        } else {
          final key = '${note.staff}_${note.voice}';
          beamGroups.putIfAbsent(key, () => <_Note>[]).add(n);
          if (note.beams.contains(BeamState.end)) {
            _drawBeamGroup(canvas, beamGroups.remove(key)!);
          }
        }

        // Tuplet grouping: collect consecutive same-voice notes that carry a
        // tuplet ratio; draw the number once the group is complete.
        final t = note.tuplet;
        final key = '${note.staff}_${note.voice}';
        if (t != null) {
          final acc = openTuplets.putIfAbsent(
            key,
            () => _TupletAcc(t.actual, up),
          );
          acc.add(x, y, note.beams.isNotEmpty);
          if (acc.count >= t.actual) {
            _drawTuplet(canvas, openTuplets.remove(key)!);
          }
        } else if (openTuplets.containsKey(key)) {
          _drawTuplet(canvas, openTuplets.remove(key)!);
        }
      }

      final lyric = note.lyric;
      if (lyric != null) {
        _text(
          canvas,
          lyric.text,
          x - _s,
          staffBottom + _s * 1.6,
          size: _s * 1.05,
        );
      }
    }
    for (final group in beamGroups.values) {
      _drawBeamGroup(canvas, group);
    }
    for (final acc in openTuplets.values) {
      _drawTuplet(canvas, acc);
    }
    return cursorX;
  }

  /// Draws a tuplet's number (e.g. "3") centred over/under its note group, with
  /// a bracket when the group is not beamed (the beam already implies grouping).
  void _drawTuplet(Canvas canvas, _TupletAcc acc) {
    if (acc.xs.isEmpty) return;
    final cx = (acc.xs.first + acc.xs.last) / 2;
    final double y;
    if (acc.up) {
      var top = acc.ys.first;
      for (final v in acc.ys) {
        if (v < top) top = v;
      }
      y = top - (_stemLen + 1.6) * _s;
    } else {
      var bot = acc.ys.first;
      for (final v in acc.ys) {
        if (v > bot) bot = v;
      }
      y = bot + (_stemLen + 0.8) * _s;
    }
    // Tuplet numbers are drawn smaller than note heads (≈ 0.6 staff spaces).
    Smufl.draw(
      canvas,
      Smufl.tupletNumber(acc.actual),
      cx,
      y,
      _s * 0.6,
      _ink,
      centerX: true,
    );
    _record(
      Rect.fromLTRB(acc.xs.first, y - _s, acc.xs.last, y + _s),
      SymbolDescriptor.tuplet(actual: acc.actual),
    );

    if (!acc.allBeamed && acc.xs.length >= 2) {
      final x0 = acc.xs.first;
      final x1 = acc.xs.last;
      final gap = _s * 0.9;
      final hook = acc.up ? _s * 0.5 : -_s * 0.5;
      final paint = Paint()
        ..color = _ink
        ..strokeWidth = Smufl.stemThickness * _s;
      canvas.drawLine(Offset(x0, y), Offset(cx - gap, y), paint);
      canvas.drawLine(Offset(cx + gap, y), Offset(x1, y), paint);
      canvas.drawLine(Offset(x0, y), Offset(x0, y + hook), paint);
      canvas.drawLine(Offset(x1, y), Offset(x1, y + hook), paint);
    }
  }

  Paint get _arcPaint => Paint()
    ..color = _ink
    ..style = PaintingStyle.stroke
    ..strokeWidth = _s * 0.13
    ..strokeCap = StrokeCap.round;

  /// A short tie between two same-pitch heads, curving below (belly down).
  void _drawTie(Canvas canvas, Offset a, Offset b) {
    final ctrl = Offset((a.dx + b.dx) / 2, (a.dy + b.dy) / 2 + _s * 1.0);
    canvas.drawPath(
      Path()
        ..moveTo(a.dx, a.dy)
        ..quadraticBezierTo(ctrl.dx, ctrl.dy, b.dx, b.dy),
      _arcPaint,
    );
    _record(
      Rect.fromLTRB(a.dx, a.dy - _s * 0.4, b.dx, ctrl.dy + _s * 0.4),
      const SymbolDescriptor.tie(),
    );
  }

  /// A phrase slur from the first to the last note, arcing above the whole
  /// phrase: the control point is placed so the curve clears the highest note
  /// ([minY]) seen while the slur was open.
  void _drawSlur(Canvas canvas, Offset a, Offset b, double minY) {
    final cx = (a.dx + b.dx) / 2;
    // Longer phrases bulge a little more so the arc stays clear of the notes.
    final clearance = _s * 1.4 + (b.dx - a.dx).abs() * 0.05;
    // Quadratic midpoint y = 0.25*(a.y+b.y) + 0.5*ctrlY; solve so it sits at
    // minY - clearance (above the highest note head).
    final ctrlY = 2 * (minY - clearance) - 0.5 * (a.dy + b.dy);
    canvas.drawPath(
      Path()
        ..moveTo(a.dx, a.dy)
        ..quadraticBezierTo(cx, ctrlY, b.dx, b.dy),
      _arcPaint,
    );
    // The quadratic's apex is halfway to the control point; band the arc there.
    final apexY = 0.25 * (a.dy + b.dy) + 0.5 * ctrlY;
    _record(
      Rect.fromLTRB(
        a.dx,
        apexY - _s * 0.5,
        b.dx,
        (a.dy > b.dy ? a.dy : b.dy) + _s * 0.3,
      ),
      const SymbolDescriptor.slur(),
    );
  }

  /// Diatonic value of a clef's bottom staff line. The clef sign sits on its
  /// `line` (G→G4, F→F3, C→C4); each staff line is two diatonic steps apart.
  /// The percussion staff maps written positions exactly as a treble
  /// (G, line 2) staff — the MusicXML convention for unpitched display
  /// placement — regardless of the file's declared clef line.
  int _clefBottomDiatonic(Clef clef) {
    if (clef.sign == ClefSign.percussion) return 4 * 7 + 2; // E4, as treble
    final refOnLine = switch (clef.sign) {
      ClefSign.f => 3 * 7 + 3, // F3
      ClefSign.c => 4 * 7 + 0, // C4
      _ => 4 * 7 + 4, // G4
    };
    return refOnLine - (clef.line - 1) * 2;
  }

  Paint get _stemPaint => Paint()
    ..color = _ink
    ..strokeWidth = Smufl.stemThickness * _s
    ..strokeCap = StrokeCap.round;

  /// Stem-attachment point on the note head for the given direction.
  Offset _stemAnchor(_Note n) {
    final s = _s * n.scale;
    return n.up
        ? Offset(
            n.x - Smufl.noteheadWidth * s / 2 + Smufl.stemUpAnchorX * s,
            n.y - Smufl.stemUpAnchorY * s,
          )
        : Offset(
            n.x - Smufl.noteheadWidth * s / 2 + Smufl.stemDownAnchorX * s,
            n.y - Smufl.stemDownAnchorY * s,
          );
  }

  void _drawStemAndFlag(Canvas canvas, _Note n) {
    final s = _s * n.scale;
    final anchor = _stemAnchor(n);
    final tipY = n.up ? anchor.dy - _stemLen * s : anchor.dy + _stemLen * s;
    canvas.drawLine(anchor, Offset(anchor.dx, tipY), _stemPaint);
    final flag = _flagGlyph(n.note, n.up);
    if (flag != null) {
      Smufl.draw(canvas, flag, anchor.dx, tipY, s, _ink);
    }
  }

  /// Draws a beamed group: one straight beam, stems of varying length reaching
  /// it, plus secondary beams for 16th-or-shorter runs.
  void _drawBeamGroup(Canvas canvas, List<_Note> group) {
    if (group.isEmpty) return;
    if (group.length == 1) {
      _drawStemAndFlag(canvas, group.first);
      return;
    }
    final up = group.first.up;
    final anchors = group.map(_stemAnchor).toList();
    // Flat beam clearing the extreme note by the stem length.
    double beamY;
    if (up) {
      beamY =
          anchors.map((a) => a.dy).reduce((a, b) => a < b ? a : b) -
          _stemLen * _s;
    } else {
      beamY =
          anchors.map((a) => a.dy).reduce((a, b) => a > b ? a : b) +
          _stemLen * _s;
    }
    for (final a in anchors) {
      canvas.drawLine(a, Offset(a.dx, beamY), _stemPaint);
    }
    final beamPaint = Paint()
      ..color = _ink
      ..strokeWidth = Smufl.beamThickness * _s
      ..strokeCap = StrokeCap.butt;
    canvas.drawLine(
      Offset(anchors.first.dx, beamY),
      Offset(anchors.last.dx, beamY),
      beamPaint,
    );
    _record(
      Rect.fromLTRB(
        anchors.first.dx,
        beamY - _s * 0.6,
        anchors.last.dx,
        beamY + _s * 0.6,
      ),
      const SymbolDescriptor.beam(),
    );
    // Secondary beam for consecutive 16th-or-shorter notes.
    final dir = up ? 1.0 : -1.0;
    final off = dir * (Smufl.beamThickness + 0.2) * _s;
    final thin = Paint()
      ..color = _ink
      ..strokeWidth = Smufl.beamThickness * _s;
    for (var i = 0; i < group.length - 1; i++) {
      if (_flagCount(group[i].note) >= 2 &&
          _flagCount(group[i + 1].note) >= 2) {
        canvas.drawLine(
          Offset(anchors[i].dx, beamY + off),
          Offset(anchors[i + 1].dx, beamY + off),
          thin,
        );
      }
    }
  }

  void _drawDots(Canvas canvas, double x, double y, int dots) {
    for (var i = 0; i < dots; i++) {
      Smufl.draw(
        canvas,
        Smufl.augmentationDot,
        x + Smufl.noteheadWidth * _s / 2 + _s * (0.3 + i * 0.5),
        y,
        _s,
        _ink,
      );
    }
  }

  void _drawLedgerLines(Canvas canvas, double x, double y, double bottomLineY) {
    final topLineY = bottomLineY - _staffHeight;
    final ext = Smufl.legerLineExtension * _s;
    final half = Smufl.noteheadWidth * _s / 2 + ext;
    final paint = Paint()
      ..color = palette.staffLine.withValues(alpha: 0.8)
      ..strokeWidth = Smufl.legerLineThickness * _s;
    for (var ly = bottomLineY + _s; ly <= y + 0.5; ly += _s) {
      canvas.drawLine(Offset(x - half, ly), Offset(x + half, ly), paint);
    }
    for (var ly = topLineY - _s; ly >= y - 0.5; ly -= _s) {
      canvas.drawLine(Offset(x - half, ly), Offset(x + half, ly), paint);
    }
  }

  String _headGlyph(NoteEvent note) {
    final div = document.attributes.divisions;
    // A cymbal (the bridged x / xOpen head class, derived once in the shared
    // crate — never re-derived from GM ranges here) takes the x form
    // following the same duration class as ordinary heads: filled x for
    // quarter and shorter, the open x forms for half and whole.
    final headClass = note.unpitched?.headClass;
    if (headClass == HeadClass.x || headClass == HeadClass.xOpen) {
      if (note.noteType == 'whole' ||
          (note.noteType == null && note.durationDivisions >= 4 * div)) {
        return Smufl.noteheadXWhole;
      }
      if (note.noteType == 'half' ||
          (note.noteType == null && note.durationDivisions >= 2 * div)) {
        return Smufl.noteheadXHalf;
      }
      return Smufl.noteheadXBlack;
    }
    switch (note.noteType) {
      case 'whole':
        return Smufl.noteheadWhole;
      case 'half':
        return Smufl.noteheadHalf;
      case null:
        if (note.durationDivisions >= 4 * div) return Smufl.noteheadWhole;
        if (note.durationDivisions >= 2 * div) return Smufl.noteheadHalf;
        return Smufl.noteheadBlack;
      default:
        return Smufl.noteheadBlack;
    }
  }

  String? _flagGlyph(NoteEvent note, bool up) => switch (note.noteType) {
    'eighth' => up ? Smufl.flag8thUp : Smufl.flag8thDown,
    '16th' => up ? Smufl.flag16thUp : Smufl.flag16thDown,
    '32nd' => up ? Smufl.flag32ndUp : Smufl.flag32ndDown,
    _ => null,
  };

  int _flagCount(NoteEvent note) => switch (note.noteType) {
    'eighth' => 1,
    '16th' => 2,
    '32nd' => 3,
    '64th' => 4,
    _ => 0,
  };

  String _restGlyph(NoteEvent note) => switch (note.noteType) {
    'whole' => Smufl.restWhole,
    'half' => Smufl.restHalf,
    'eighth' => Smufl.rest8th,
    '16th' => Smufl.rest16th,
    _ => Smufl.restQuarter,
  };

  void _text(
    Canvas canvas,
    String text,
    double x,
    double y, {
    double size = 13,
    Color color = CymbraColors.onSurface,
    bool italic = false,
  }) {
    if (text.isEmpty) return;
    TextPainter(
        text: TextSpan(
          text: text,
          style: TextStyle(
            fontFamily: textFontFamily,
            color: color,
            fontSize: size,
            fontStyle: italic ? FontStyle.italic : FontStyle.normal,
          ),
        ),
        textDirection: TextDirection.ltr,
      )
      ..layout()
      ..paint(canvas, Offset(x, y));
  }

  double _textWidth(String text, double size, {bool italic = false}) {
    final tp = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          fontFamily: textFontFamily,
          fontSize: size,
          fontStyle: italic ? FontStyle.italic : FontStyle.normal,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    return tp.width;
  }

  @override
  bool shouldRepaint(PartitionPainter old) =>
      old.document != document ||
      old.systems != systems ||
      old.elapsedMs != elapsedMs ||
      old.activeNotes != activeNotes ||
      old.selectedHands != selectedHands ||
      old.viewTop != viewTop ||
      old.viewBottom != viewBottom ||
      old.textFontFamily != textFontFamily ||
      old.staffSpace != staffSpace ||
      old.palette != palette ||
      old.practiceRange != practiceRange;
}

/// A note's drawn geometry (head centre + stem direction), used for beaming.
class _Note {
  final double x;
  final double y;
  final bool up;
  final NoteEvent note;

  /// Glyph scale — 1.0 for ordinary notes, smaller for grace notes.
  final double scale;
  const _Note(this.x, this.y, this.up, this.note, [this.scale = 1.0]);
}

/// Tracks open tie/slur starts within a system so the arc can be drawn when the
/// matching stop note is reached. Ties key on pitch (same note resumed); slurs
/// key on staff+voice and nest as a stack.
class _Arcs {
  final Map<String, Offset> _ties = {};
  final Map<String, List<_SlurOpen>> _slurs = {};

  void putTie(String key, Offset start) => _ties[key] = start;
  Offset? takeTie(String key) => _ties.remove(key);

  void pushSlur(String key, Offset start) =>
      _slurs.putIfAbsent(key, () => <_SlurOpen>[]).add(_SlurOpen(start));

  /// Updates the open slur's highest note (smallest y) as the phrase is drawn.
  void observeSlur(String key, double y) {
    final stack = _slurs[key];
    if (stack != null && stack.isNotEmpty && y < stack.last.minY) {
      stack.last.minY = y;
    }
  }

  _SlurOpen? popSlur(String key) {
    final stack = _slurs[key];
    if (stack == null || stack.isEmpty) return null;
    return stack.removeLast();
  }
}

/// An open slur: its start point and the highest note (smallest y) seen so far.
class _SlurOpen {
  final Offset start;
  double minY;
  _SlurOpen(this.start) : minY = start.dy;
}

/// Accumulates a tuplet's notes so its number/bracket can be drawn once the
/// group (`actual` notes) is complete.
class _TupletAcc {
  final int actual;
  final bool up;
  final List<double> xs = [];
  final List<double> ys = [];
  bool allBeamed = true;
  _TupletAcc(this.actual, this.up);

  void add(double x, double y, bool beamed) {
    xs.add(x);
    ys.add(y);
    if (!beamed) allBeamed = false;
  }

  int get count => xs.length;
}

/// Assigns text (e.g. expression words) to stacked rows so overlapping items at
/// nearby x-positions don't collide: each item takes the lowest row whose last
/// occupied x has cleared, else a new row above.
class _TextLanes {
  final double rowGap;
  final List<double> _rowEndX = [];
  _TextLanes(this.rowGap);

  /// Baseline Y for an item of [width] starting at [x]; rows stack upward.
  double yFor(double x, double width, double baseY) {
    for (var r = 0; r < _rowEndX.length; r++) {
      if (_rowEndX[r] <= x) {
        _rowEndX[r] = x + width;
        return baseY - r * rowGap;
      }
    }
    _rowEndX.add(x + width);
    return baseY - (_rowEndX.length - 1) * rowGap;
  }
}
