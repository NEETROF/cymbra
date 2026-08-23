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

import 'package:flutter/material.dart';

import '../src/rust/api/musicxml.dart' show BeamState;
import '../state/note_density_core.dart';
import '../state/player_data.dart';
import '../theme/cymbra_theme.dart';
import 'notation_palette.dart';
import 'smufl.dart';
import 'staff_hit_index.dart';

/// "Standard staff" rendering synchronized to time, like the waterfall.
///
/// The staff (5 lines) stays fixed; the notes and measure bars scroll
/// horizontally from right to left based on [elapsedMs]. A vertical playhead
/// line marks the current instant: only the note aligned on it is highlighted —
/// green if it's correctly played, teal if it's expected.
class StaffPainter extends CustomPainter {
  final List<TimedNote> notes;

  /// Rests to engrave on the staff (render-only; never played or scored). Placed
  /// by time like the notes, routed to their staff.
  final List<TimedRest> rests;

  /// Tie continuations to engrave on the staff (render-only; never played or
  /// scored): the `tie stop` notes whose durations were merged into their
  /// chain's first note in [notes]. Drawn exactly like notes — head, stem/beam,
  /// dots — plus the tie arc back to the engraved note each one prolongs
  /// ([TimedNote.tieFromMs]), so the notation stays as written instead of
  /// leaving the prolonged measures empty.
  final List<TimedNote> tieContinuations;

  final double elapsedMs;
  final Set<int> activeNotes;

  /// Tempo, to place the measure bars.
  final int bpm;

  /// End of the song (ms), to bound the measure bars.
  final double songEndMs;

  /// Key signature (fifths) and time signature of the loaded piece, drawn as the
  /// armature + meter at the head of the system. [keyFifths] is the fallback when
  /// no per-measure data is supplied; [measureKeyFifths] (aligned with
  /// [measureStartMs]) lets the head show the armure in force at the playhead, so
  /// a mid-piece modulation is reflected as the score scrolls past it.
  final int keyFifths;
  final List<int> measureKeyFifths;
  final int beats;
  final int beatType;

  /// Absolute start time (ms) of each measure, in order — the same table that
  /// places the notes, so the bar lines land exactly on the real measure
  /// boundaries (any time signature). Empty for the demo score, which falls back
  /// to a meter-derived spacing.
  final List<int> measureStartMs;

  /// Optional replay overlay: a ring colour per note **index** (into [notes]) to
  /// mark where the player made a mistake, drawn in place on the staff. Empty in
  /// normal playback; populated by the mistake replay.
  final Map<int, Color> mistakeColors;

  /// Visible time window (ms) to the right of the playhead. A larger window fits
  /// more of the score across the same width — smaller, denser notation (used by
  /// the in-card rating preview). Defaults to [_defaultLookAheadMs] (the player's
  /// original 4 s window), so the player is unaffected.
  final double lookAheadMs;

  /// The score's tightest characteristic note spacing (ms), from
  /// [onsetGapMs] — when supplied, [lookAheadMs] is **narrowed** until that
  /// spacing engraves at least [minOnsetSpaces] wide, so a dense piece is not
  /// crushed into the same window as a slow one (see
  /// [densityCappedLookAheadMs]). `null` disables the cap and keeps
  /// [lookAheadMs] verbatim, which is what the fixed-scale previews want.
  final double? onsetGapMs;

  /// The score's typical measure duration (ms), from [medianMeasureMs] — when
  /// supplied, [lookAheadMs] is also narrowed so no more than
  /// [kMaxVisibleMeasures] measures sit ahead of the playhead. Independent of
  /// [onsetGapMs]: spacing keeps glyphs apart, this bounds how much score the
  /// reader faces. `null` disables it, like [onsetGapMs].
  final double? measureMs;

  /// Multiplier on the staff size (and therefore the note glyphs, stems, clefs
  /// and armature — everything derives from the staff line gap). `1.0` is the
  /// player's size; the in-card preview uses a smaller value to shrink the
  /// notation without changing the horizontal note spacing (that is [lookAheadMs]).
  final double noteScale;

  const StaffPainter({
    required this.notes,
    required this.elapsedMs,
    required this.activeNotes,
    required this.bpm,
    required this.songEndMs,
    this.rests = const [],
    this.tieContinuations = const [],
    this.keyFifths = 0,
    this.measureKeyFifths = const [],
    this.beats = 4,
    this.beatType = 4,
    this.measureStartMs = const [],
    this.mistakeColors = const {},
    this.lookAheadMs = defaultLookAheadMs,
    this.onsetGapMs,
    this.measureMs,
    this.noteScale = 1.0,
    this.palette = NotationPalette.dark,
    this.hitIndex,
    this.practiceRange,
    this.measureHits,
    this.showPlayhead = true,
    this.timeOriginFraction = 0.25,
  });

  /// Where `elapsedMs` lands across the width, as a fraction. The player keeps
  /// the playhead at a quarter so past notes stay visible to its left; a static
  /// ribbon wants the music to start right after the clef instead, so it passes
  /// a small value — otherwise a wide canvas leaves a quarter of it blank before
  /// the first note.
  final double timeOriginFraction;

  /// The active practice measure range, tinted across the system (change:
  /// add-measure-range-practice). Null for a full run — nothing is tinted.
  final ({int start, int end})? practiceRange;

  /// Sink filled during [paint] with each measure's on-screen rectangle, so a
  /// horizontal range picker can hit-test a tap back to a bar. Requires
  /// [measureStartMs]; null (the default) disables the collection entirely.
  final List<({int measure, Rect rect})>? measureHits;

  /// Whether the playback line is drawn. The range picker is not playing
  /// anything, so it turns the line off rather than implying a playhead.
  final bool showPlayhead;

  /// Colour set (dark surface or paper) the staff is drawn with.
  final NotationPalette palette;

  /// Optional side channel (change: add-notation-help, D1): when supplied, each
  /// drawn symbol records its on-screen rect + [SymbolDescriptor] into it, so a
  /// long-press can be resolved to the symbol under the finger. It is cleared and
  /// refilled every [paint] and **never** affects what is drawn — a null index
  /// leaves rendering byte-identical.
  final StaffHitIndex? hitIndex;

  /// Vertical placement of the grand staff: the treble/bass block is centred
  /// in [height] with the inter-staff gap **clamped between 2 and 8 line
  /// gaps** — enough air for the between-hands ledger notes, never the
  /// pinned-to-the-edges void a tall window used to produce. The clamp floor
  /// mirrors the fit budget of [staffLineGap], so on short heights this
  /// degrades exactly as before.
  static ({double trebleBottom, double bassBottom}) grandStaffLayout({
    required double height,
    required double lineGap,
    required double stemClearance,
  }) {
    final gap = (height - 2 * stemClearance - 8 * lineGap).clamp(
      2 * lineGap,
      8 * lineGap,
    );
    final blockHeight = 8 * lineGap + gap;
    final top = math.max((height - blockHeight) / 2, stemClearance);
    return (trebleBottom: top + 4 * lineGap, bassBottom: top + blockHeight);
  }

  /// Staff line gap for a render area [height]: proportional within the usual
  /// 8–12 px band (12 = the Partition's staff space, so both notation
  /// modes read at the same size), **capped so the engraving always fits** — a grand staff
  /// needs ~19.2 gaps of vertical budget (two 4-gap staves, stem/beam clearance
  /// above and below, and at least two gaps of air between the hands), a lone
  /// staff ~12. Without the cap a short window kept the 8 px floor and the
  /// two staves collided. The 3 px hard floor is the readability minimum the
  /// glyphs can shrink to. `noteScale` applies after the clamps so the in-card
  /// preview can deliberately go smaller.
  static double staffLineGap({
    required double height,
    required bool twoStaff,
    double noteScale = 1.0,
  }) {
    final proportional = (height * (twoStaff ? 0.055 : 0.10)).clamp(
      8.0,
      _partitionStaffSpace,
    );
    final fitCap = height / (twoStaff ? 19.2 : 12.0);
    return math.min(proportional, fitCap).clamp(3.0, _partitionStaffSpace) *
        noteScale;
  }

  /// The engraved Partition's staff space (its `_s` at the medium score size):
  /// the Portée settles at exactly this size on comfortable viewports, so the
  /// two notation modes engrave at the same scale — the score-size setting then
  /// multiplies both identically via [noteScale].
  static const double _partitionStaffSpace = 12.0;

  /// Default visible time window to the right of the playhead. The score-size
  /// setting divides it by its factor so bigger notes get matching spacing, and
  /// [onsetGapMs] narrows it further on dense scores — this is the *widest* the
  /// window ever gets, not the window itself.
  static const double defaultLookAheadMs = 4000;

  /// Gap (staff spaces) between an accidental's left edge and its notehead's,
  /// the offset the accidental is engraved at. Feeds [minOnsetSpaces].
  static const double _accidentalOffset = 1.5;

  /// Narrowest advance (staff spaces) between two consecutive onsets on one
  /// staff that this painter can still engrave legibly — the density budget
  /// [densityCappedLookAheadMs] sizes the look-ahead window against.
  ///
  /// Read off what the painter actually draws, for the worst ordinary case (a
  /// note whose neighbour carries an accidental): half of the left head, then
  /// the right note's accidental, which reaches [_accidentalOffset] plus half a
  /// head to the left of its own centre. ≈ 2.68 spaces.
  ///
  /// Deliberately wider than the engraved Partition's `MIN_COL` (22 px at its
  /// 12 px staff space ≈ 1.83 spaces, `crates/musicxml-core/src/lib.rs`): the
  /// Partition wraps to a handful of measures per line and is read standing
  /// still, while the Portée puts one unbroken line in front of a player
  /// sight-reading it as it moves. Same glyphs, less time to decode them.
  static const double minOnsetSpaces =
      Smufl.noteheadWidth / 2 + _accidentalOffset + Smufl.noteheadWidth / 2;

  @override
  void paint(Canvas canvas, Size size) {
    // Refill the hit index from scratch every frame so it always matches what is
    // currently on screen (side channel; no effect on drawing).
    final hits = hitIndex?..clear();
    void record(Rect r, SymbolDescriptor d) => hits?.add(r, d);

    if (notes.isEmpty && rests.isEmpty) return;

    const margin = 48.0;
    // A grand staff needs both hands present in the (already hand-filtered)
    // notes/rests; when a single hand is shown only its staff is drawn and
    // recentred.
    final hasTreble =
        notes.any((n) => n.staff == 1) || rests.any((r) => r.staff == 1);
    final hasBass =
        notes.any((n) => n.staff >= 2) || rests.any((r) => r.staff >= 2);
    final twoStaff = hasTreble && hasBass;
    // The kept staff when a single hand is shown: its clef/armature are drawn on
    // the lone staff (bass when only staff 2+ remains, else treble).
    final soloStaff = !twoStaff && hasBass ? 2 : 1;
    // The staff line gap sizes ALL notation (notes, stems, glyphs, armature).
    final lineGap = staffLineGap(
      height: size.height,
      twoStaff: twoStaff,
      noteScale: noteScale,
    );
    final stepGap = lineGap / 2;

    // Time origin: the player pins the playhead at the left quarter and time
    // advances toward the left; a static ribbon pushes it hard left so the whole
    // piece gets the width.
    final playLineX = size.width * timeOriginFraction;
    final trackPx = size.width - playLineX - margin;
    // The window the notes are spread over: the caller's, narrowed by the
    // score's own density and by its measure length, so a fast 3/8 does not
    // engrave five cramped measures where a slow 4/2 gets one. Both bounds are
    // constants of the piece, so the scroll speed stays linear.
    final window = densityCappedLookAheadMs(
      requestedMs: lookAheadMs,
      trackPx: trackPx,
      lineGap: lineGap,
      minSpaces: minOnsetSpaces,
      gapMs: onsetGapMs,
      measureMs: measureMs,
    );
    final pxPerMs = trackPx / window;
    double xForTime(double tMs) => playLineX + (tMs - elapsedMs) * pxPerMs;

    // Vertical placement of the staff/staves. Stems follow the notation, so a
    // note can carry its stem and beam either above or below its head: reserve
    // the same clearance on both sides of the system (~3.9·lineGap of stem/beam
    // plus a little ledger room). Otherwise notes near the top or bottom clip
    // against the render area — visible on short phone-landscape viewports,
    // where the proportional margin alone is too small.
    final stemClearance = lineGap * 4.6;
    final double trebleBottom;
    final double? bassBottom;
    if (twoStaff) {
      final layout = grandStaffLayout(
        height: size.height,
        lineGap: lineGap,
        stemClearance: stemClearance,
      );
      trebleBottom = layout.trebleBottom;
      bassBottom = layout.bassBottom;
    } else {
      // Centre the lone staff, but guarantee the stem clearance above its top
      // line and below its bottom line so extreme notes stay visible on short
      // viewports. The top constraint wins when both cannot be met.
      final lowest = stemClearance + 4 * lineGap; // clearance above the staff
      final highest = size.height - stemClearance; // clearance below the staff
      final centred = size.height / 2 + 2 * lineGap;
      trebleBottom = highest > lowest
          ? centred.clamp(lowest, highest)
          : math.max(centred, lowest);
      bassBottom = null;
    }

    final linePaint = Paint()
      ..color = palette.staffLine.withValues(alpha: 0.45)
      ..strokeWidth = 1.2;
    final barPaint = Paint()
      ..color = palette.ink.withValues(alpha: 0.5)
      ..strokeWidth = 1.4;

    // 1) Staff lines (treble, and bass for a grand staff).
    _drawStaffLines(
      canvas,
      trebleBottom,
      margin,
      size.width,
      lineGap,
      linePaint,
    );
    if (bassBottom != null) {
      _drawStaffLines(
        canvas,
        bassBottom,
        margin,
        size.width,
        lineGap,
        linePaint,
      );
    }

    final systemTop = trebleBottom - 4 * lineGap;
    final systemBottom = bassBottom ?? trebleBottom;

    // Clefs (SMuFL glyphs) at the head of the system — the clef in effect at
    // the playhead, so a mid-piece clef change is reflected as you scroll. On a
    // collapsed single staff the lone staff carries the kept hand's clef.
    final topStaff = twoStaff ? 1 : soloStaff;
    final trebleClef = _clefAtPlayhead(topStaff);
    Smufl.draw(
      canvas,
      Smufl.clef(trebleClef.$1),
      6,
      trebleBottom - (trebleClef.$2 - 1) * lineGap,
      lineGap,
      palette.staffLine,
    );
    record(
      Rect.fromLTWH(
        2,
        trebleBottom - 5.2 * lineGap,
        lineGap * 3.4,
        lineGap * 7,
      ),
      SymbolDescriptor.clef(sign: trebleClef.$1),
    );
    if (bassBottom != null) {
      final bassClef = _clefAtPlayhead(2);
      Smufl.draw(
        canvas,
        Smufl.clef(bassClef.$1),
        6,
        bassBottom - (bassClef.$2 - 1) * lineGap,
        lineGap,
        palette.staffLine,
      );
      record(
        Rect.fromLTWH(
          2,
          bassBottom - 4.6 * lineGap,
          lineGap * 3.4,
          lineGap * 5,
        ),
        SymbolDescriptor.clef(sign: bassClef.$1),
      );
    }

    // Key signature (armature) + time signature at the head of the system. The
    // armure reflects the key at the playhead, so a mid-piece modulation appears
    // as you scroll past it (like the clef above).
    final headColor = palette.staffLine;
    final headKey = _keyFifthsAtPlayhead();
    var hx = 6 + lineGap * 2.8;
    final keyW = Smufl.drawKeySignature(
      canvas,
      hx,
      trebleBottom,
      lineGap,
      headKey,
      topStaff >= 2, // bass-clef placement when the lone staff is the left hand
      headColor,
    );
    if (bassBottom != null) {
      Smufl.drawKeySignature(
        canvas,
        hx,
        bassBottom,
        lineGap,
        headKey,
        true,
        headColor,
      );
    }
    if (keyW > 0) {
      record(
        Rect.fromLTWH(
          hx,
          systemTop - lineGap,
          keyW,
          systemBottom - systemTop + 2 * lineGap,
        ),
        SymbolDescriptor.keySignature(fifths: headKey),
      );
    }
    hx += keyW;
    final timeW = Smufl.drawTimeSignature(
      canvas,
      hx,
      trebleBottom,
      lineGap,
      beats,
      beatType,
      headColor,
    );
    record(
      Rect.fromLTWH(
        hx,
        systemTop - lineGap,
        timeW,
        systemBottom - systemTop + 2 * lineGap,
      ),
      SymbolDescriptor.timeSignature(beats: beats, beatType: beatType),
    );
    if (bassBottom != null) {
      Smufl.drawTimeSignature(
        canvas,
        hx,
        bassBottom,
        lineGap,
        beats,
        beatType,
        headColor,
      );
    }
    // Right edge of the fixed head (clef + armature + metre). The scrolling
    // glyphs (bar lines, notes, rests, beams) are kept to the right of this so
    // they slide UNDER the head as they travel left, instead of drawing on top of
    // the clef/armature.
    final headEnd = hx + timeW + lineGap * 0.4;

    // 2) Scrolling measure bars (span the whole system). Drawn at the real
    // measure boundaries from [measureStartMs] (plus the final bar at songEnd) so
    // they match the notes for any time signature. The demo score carries no
    // table, so fall back to a meter-derived spacing (beats × beat-unit), not a
    // hardcoded 4/4.
    //
    // A bar line sits *just left* of the measure's first note — in the gap before
    // the downbeat — so the downbeat (also at the measure-start time) doesn't land
    // on top of its bar. There is no opening bar before the very first measure;
    // the closing bar at songEnd has no downbeat after it, so it is not shifted.
    final barGap = lineGap * 1.25;
    void drawBar(double t, {bool beforeDownbeat = true}) {
      if (t <= 0) return; // no bar before the first measure
      final x = xForTime(t) - (beforeDownbeat ? barGap : 0);
      // Clip to the right of the head so a bar line never crosses the clef/armature.
      if (x < headEnd || x > size.width - margin) return;
      canvas.drawLine(Offset(x, systemTop), Offset(x, systemBottom), barPaint);
      record(
        Rect.fromLTRB(
          x - lineGap * 0.4,
          systemTop,
          x + lineGap * 0.4,
          systemBottom,
        ),
        const SymbolDescriptor.barLine(),
      );
    }

    // Measure geometry for the horizontal range picker (change: add-measure-
    // range-practice): each bar's on-screen span, derived from the SAME
    // time→x mapping as the notes, so a tap can never disagree with what is
    // drawn. Also paints the active range's tint under the notation.
    measureHits?.clear();
    if (measureStartMs.isNotEmpty &&
        (measureHits != null || practiceRange != null)) {
      // The same tint as the engraved Partition picker, so both surfaces read
      // as one feature.
      final tint = Paint()
        ..color = CymbraColors.tertiary.withValues(alpha: 0.14);
      for (var i = 0; i < measureStartMs.length; i++) {
        final startX = xForTime(measureStartMs[i].toDouble());
        final endT = i + 1 < measureStartMs.length
            ? measureStartMs[i + 1].toDouble()
            : songEndMs;
        final endX = xForTime(endT);
        // Clip to the visible track so the head never absorbs a bar's rect.
        final l = math.max(startX, headEnd);
        final r = math.min(endX, size.width - margin);
        if (r <= l) continue;
        final bounds = Rect.fromLTRB(l, systemTop, r, systemBottom);
        measureHits?.add((measure: i, rect: bounds));
        final range = practiceRange;
        if (range != null && i >= range.start && i <= range.end) {
          canvas.drawRect(bounds, tint);
        }
      }
    }

    if (measureStartMs.isNotEmpty) {
      for (final t in measureStartMs) {
        drawBar(t.toDouble());
      }
      drawBar(songEndMs, beforeDownbeat: false); // closing bar line
    } else {
      final bt = beatType == 0 ? 4 : beatType;
      final measureMs = (60000.0 / bpm) * beats * 4 / bt;
      if (measureMs > 0) {
        for (var t = measureMs; t <= songEndMs + measureMs; t += measureMs) {
          drawBar(t);
        }
      }
    }

    // 3) Playhead (playback line) across the whole system. Suppressed by the
    // range picker, which shows the music as a static ribbon and has no
    // playhead to point at.
    if (showPlayhead) {
      final playPaint = Paint()
        ..color = palette.accent.withValues(alpha: 0.9)
        ..strokeWidth = 2;
      canvas.drawLine(
        Offset(playLineX, systemTop - lineGap * 1.5),
        Offset(playLineX, systemBottom + lineGap * 1.5),
        playPaint,
      );
    }

    // Vertical position of a note on its staff.
    double noteY(TimedNote n) {
      final isBass = bassBottom != null && n.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
      // Position by the clef in effect for this note (not its staff index), and
      // by the note's *written* staff step when known (so an A♭ sits on the A
      // line like the engraved Partition), falling back to the MIDI number for
      // MIDI-only sources (demo/replay).
      final bottom = _clefBottomDiatonic(n.clefSign, n.clefLine);
      final dia = n.diatonic ?? _diatonic(n.pitch);
      return base - (dia - bottom) * stepGap;
    }

    // Stem direction for a note: the one the notation engraves when it carries
    // it, else the engraving default (heads below the middle line stem up).
    // Same rule as the Partition view, so both render a run of eighths alike.
    bool stemUpOf(TimedNote n) {
      final carried = n.stemUp;
      if (carried != null) return carried;
      final isBass = bassBottom != null && n.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
      return noteY(n) >= base - 2 * lineGap;
    }

    final quarterMs = bpm > 0 ? 60000.0 / bpm : 500.0;
    // Flags from the engraved note type when known — a duration ratio lies for
    // notes whose played duration differs from their figure (a merged tie
    // chain's eighth lasting a whole, a grace note's nominal sliver) — with the
    // ratio as the MIDI-only fallback.
    int flagsOf(TimedNote n) {
      switch (n.noteType) {
        case 'whole' || 'half' || 'quarter':
          return 0;
        case 'eighth':
          return 1;
        case '16th' || '32nd' || '64th':
          return 2; // two-flag glyph is the smallest we draw
      }
      final ratio = n.durationMs / quarterMs;
      return ratio <= 0.32 ? 2 : (ratio <= 0.62 ? 1 : 0);
    }

    // Everything engraved as a note: the playable notes plus the render-only
    // tie continuations, merged by onset (both lists arrive start-sorted) so
    // beam groups keep their engraved member order — a continuation can open a
    // beam group whose remaining members are playable (e.g. an eighth tied from
    // the previous measure beamed with the sixteenths that follow it).
    final drawNotes = _mergedByStart(notes, tieContinuations);

    // Beam groups carried from the notation (per staff). Members get a beam
    // instead of individual flags.
    final beamGroups = <List<TimedNote>>[];
    final openGroups = <int, List<TimedNote>>{};
    for (final n in drawNotes) {
      if (n.beams.isEmpty) continue;
      final g = openGroups.putIfAbsent(n.staff, () => <TimedNote>[]);
      g.add(n);
      if (n.beams.contains(BeamState.end)) {
        beamGroups.add(g);
        openGroups.remove(n.staff);
      }
    }
    beamGroups.addAll(openGroups.values);
    final beamed = beamGroups.expand((g) => g).toSet();

    bool visible(double x) =>
        x >= margin - lineGap && x <= size.width - margin + lineGap;

    // Notes are coloured by hand (right = blue, left = amber): brighter at the
    // playhead ("play now"), success green once correctly held.
    Color colorFor(TimedNote n) {
      final handColor = n.staff >= 2 ? palette.handLeft : palette.handRight;
      final atPlayhead =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      if (atPlayhead && activeNotes.contains(n.pitch)) {
        return palette.correct; // well played
      } else if (atPlayhead) {
        return Color.lerp(handColor, const Color(0xFFFFFFFF), 0.4)!; // play now
      }
      return handColor; // upcoming, by hand
    }

    // Keep every scrolling glyph to the right of the fixed head, so notes/beams
    // slide under the clef/armature instead of painting over it.
    canvas.save();
    canvas.clipRect(Rect.fromLTRB(headEnd, 0, size.width, size.height));

    // 4) Scrolling notes, routed to their staff. One body for both channels:
    // playable notes (which can carry a replay mistake ring) and the render-only
    // tie continuations (which cannot — the mistake overlay indexes [notes]).
    void drawNoteGlyphs(TimedNote n, {Color? mistake}) {
      final x = xForTime(n.startMs.toDouble());
      if (!visible(x)) return;
      final y = noteY(n);
      final atPlayhead =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      final color = colorFor(n);

      // Replay: ring the note in its mistake colour, in place on the staff.
      if (mistake != null) {
        canvas.drawCircle(
          Offset(x, y),
          lineGap * 1.6,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2.5
            ..color = mistake,
        );
      }

      // Grace notes engrave smaller (head, stem and flag), like the Partition.
      final glyphGap = n.isGrace ? lineGap * 0.7 : lineGap;
      final head = _headGlyph(n, quarterMs);
      _drawHead(canvas, Offset(x, y), glyphGap, atPlayhead, color, head);
      record(
        Rect.fromCenter(
          center: Offset(x, y),
          width: Smufl.noteheadWidth * lineGap * 1.5,
          height: lineGap * 1.7,
        ),
        SymbolDescriptor.note(
          pitch: n.pitch,
          diatonic: n.diatonic,
          clefSign: n.clefSign,
          staff: n.staff,
          noteType: n.noteType,
          dots: n.dots,
          isGrace: n.isGrace,
        ),
      );
      // Accidental engraved on this note (sharp/flat/natural…), left of the
      // head like the Partition view — without it a D♯ reads as a plain D.
      final token = n.accidental;
      if (token != null) {
        final glyph = Smufl.accidental(token);
        if (glyph != null) {
          final ax =
              x -
              Smufl.noteheadWidth * lineGap / 2 -
              lineGap * _accidentalOffset;
          Smufl.draw(canvas, glyph, ax, y, lineGap, color);
          record(
            Rect.fromLTWH(ax, y - lineGap * 1.4, lineGap * 1.3, lineGap * 2.6),
            SymbolDescriptor.accidental(token: token),
          );
        }
      }
      // Whole notes carry no stem; others do. Beamed notes get their stem/beam
      // from the group pass, and a chord member shares its principal's stem
      // (drawing its own put a spurious flag next to the principal's beam), so
      // only unbeamed, non-whole, non-chord notes stem here — the same rule the
      // engraved Partition applies.
      if (!beamed.contains(n) && head != Smufl.noteheadWhole && !n.isChord) {
        _drawStemFlag(
          canvas,
          Offset(x, y),
          glyphGap,
          color,
          flagsOf(n),
          stemUpOf(n),
        );
      }
      if (n.dots > 0) {
        _drawDots(canvas, Offset(x, y), lineGap, n.dots, color);
        record(
          Rect.fromLTWH(
            x + Smufl.noteheadWidth * lineGap / 2,
            y - lineGap * 0.6,
            lineGap * (0.4 + n.dots * 0.5),
            lineGap * 1.2,
          ),
          const SymbolDescriptor.augmentationDot(),
        );
      }
      final isBass = bassBottom != null && n.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
      _drawLedgerLines(
        canvas,
        x,
        y,
        base,
        base - 4 * lineGap,
        lineGap,
        linePaint,
      );
      // A note sitting above the top line or below the bottom line carries ledger
      // lines; record the gap between the staff edge and the head so a press on
      // one of those short lines is explained (not swallowed as empty).
      if (y > base + lineGap * 0.5 || y < base - 4 * lineGap - lineGap * 0.5) {
        final edge = y > base ? base : base - 4 * lineGap;
        record(
          Rect.fromLTRB(
            x - lineGap * 1.1,
            math.min(edge, y),
            x + lineGap * 1.1,
            math.max(edge, y),
          ),
          const SymbolDescriptor.ledgerLine(),
        );
      }
    }

    for (var i = 0; i < notes.length; i++) {
      drawNoteGlyphs(notes[i], mistake: mistakeColors[i]);
    }
    // 4a) The engraved tie continuations, drawn identically (their beamed
    // members share the group pass below like everyone else's).
    for (final n in tieContinuations) {
      drawNoteGlyphs(n);
    }

    // 4b) Scrolling rests, routed to their staff and centred on its middle line
    // (two spaces above the bottom line), like the engraved Partition view.
    final restColor = palette.staffLine;
    for (final r in rests) {
      final x = xForTime(r.startMs.toDouble());
      if (!visible(x)) continue;
      final isBass = bassBottom != null && r.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
      final y = base - 2 * lineGap;
      Smufl.draw(
        canvas,
        _restGlyph(r, quarterMs),
        x,
        y,
        lineGap,
        restColor,
        centerX: true,
      );
      record(
        Rect.fromCenter(
          center: Offset(x, y - lineGap * 0.5),
          width: lineGap * 1.6,
          height: lineGap * 2.4,
        ),
        SymbolDescriptor.rest(noteType: r.noteType),
      );
      if (r.dots > 0) {
        _drawDots(canvas, Offset(x, y), lineGap, r.dots, restColor);
        record(
          Rect.fromLTWH(
            x + lineGap * 0.6,
            y - lineGap * 0.6,
            lineGap * (0.4 + r.dots * 0.5),
            lineGap * 1.2,
          ),
          const SymbolDescriptor.augmentationDot(),
        );
      }
    }

    // 5) Beams over (or under) their groups — one straight beam each, on the
    // side the group's stems point to.
    for (final group in beamGroups) {
      if (group.length < 2) {
        if (group.length == 1 &&
            visible(xForTime(group.first.startMs.toDouble()))) {
          final n = group.first;
          _drawStemFlag(
            canvas,
            Offset(xForTime(n.startMs.toDouble()), noteY(n)),
            lineGap,
            colorFor(n),
            flagsOf(n),
            stemUpOf(n),
          );
        }
        continue;
      }
      final pts = group
          .map((n) => Offset(xForTime(n.startMs.toDouble()), noteY(n)))
          .toList();
      if (pts.every((p) => !visible(p.dx))) continue;
      final up = stemUpOf(group.first);
      _drawBeam(canvas, pts, group, lineGap, up);
      // The beam bar itself sits a stem length off the extreme head, on the stem
      // side — record a thin band there so a press on the beam is explained.
      final beamY = up
          ? pts.map((p) => p.dy).reduce(math.min) - lineGap * _stemLen
          : pts.map((p) => p.dy).reduce(math.max) + lineGap * _stemLen;
      record(
        Rect.fromLTRB(
          pts.first.dx,
          beamY - lineGap * 0.6,
          pts.last.dx,
          beamY + lineGap * 0.6,
        ),
        const SymbolDescriptor.beam(),
      );
    }

    // 5b) Tie arcs: each continuation is joined to the engraved note it
    // prolongs, on the head side (away from the stem), like the Partition view.
    final headHalf = Smufl.noteheadWidth * lineGap / 2;
    for (final n in tieContinuations) {
      final from = n.tieFromMs;
      if (from == null) continue;
      final x1 = xForTime(from.toDouble()) + headHalf;
      final x2 = xForTime(n.startMs.toDouble()) - headHalf;
      // Cull only when the whole span is off-screen: a chain crossing the edge
      // still shows its arc up to the clip.
      if (x2 < margin - lineGap || x1 > size.width - margin + lineGap) continue;
      final y = noteY(n);
      final bowUp = !stemUpOf(n);
      _drawTieArc(canvas, x1, x2, y, lineGap, bowUp);
      record(
        Rect.fromLTRB(
          math.min(x1, x2),
          y - lineGap * (bowUp ? 1.6 : 0.4),
          math.max(x1, x2),
          y + lineGap * (bowUp ? 0.4 : 1.6),
        ),
        const SymbolDescriptor.tie(),
      );
    }

    canvas.restore(); // end the scrolling-glyph clip
  }

  /// Playable notes and tie continuations interleaved by onset — a stable merge
  /// of two start-sorted lists, so beam groups see the engraved member order.
  static List<TimedNote> _mergedByStart(List<TimedNote> a, List<TimedNote> b) {
    if (b.isEmpty) return a;
    if (a.isEmpty) return b;
    final merged = <TimedNote>[];
    var i = 0, j = 0;
    while (i < a.length && j < b.length) {
      merged.add(a[i].startMs <= b[j].startMs ? a[i++] : b[j++]);
    }
    merged
      ..addAll(a.sublist(i))
      ..addAll(b.sublist(j));
    return merged;
  }

  /// A tie between two same-pitch heads: a shallow arc from the previous head's
  /// right edge to the continuation's left edge, bowed on the head side.
  void _drawTieArc(
    Canvas canvas,
    double x1,
    double x2,
    double y,
    double lineGap,
    bool bowUp,
  ) {
    final dir = bowUp ? -1.0 : 1.0;
    final yEdge = y + dir * lineGap * 0.55;
    canvas.drawPath(
      Path()
        ..moveTo(x1, yEdge)
        ..quadraticBezierTo((x1 + x2) / 2, yEdge + dir * lineGap, x2, yEdge),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = Smufl.stemThickness * lineGap
        ..color = palette.staffLine.withValues(alpha: 0.75),
    );
  }

  void _drawStaffLines(
    Canvas canvas,
    double bottomLineY,
    double margin,
    double width,
    double lineGap,
    Paint linePaint,
  ) {
    for (var i = 0; i < 5; i++) {
      final y = bottomLineY - i * lineGap;
      canvas.drawLine(Offset(margin, y), Offset(width - margin, y), linePaint);
    }
  }

  /// Diatonic value of the bottom staff line for a clef (sign on its `line`).
  /// Uses MIDI reference pitches through [_diatonic], so it shares the same
  /// written-diatonic scale as both [_diatonic] and [TimedNote.diatonic].
  int _clefBottomDiatonic(String sign, int line) {
    final refMidi = switch (sign) {
      'F' => 53, // F3
      'C' => 60, // C4
      _ => 67, // G4
    };
    return _diatonic(refMidi) - (line - 1) * 2;
  }

  /// Key signature (fifths) in force at the playhead: the armure of the measure
  /// containing [elapsedMs]. Falls back to the fixed [keyFifths] when no
  /// per-measure data is supplied (e.g. the demo score).
  int _keyFifthsAtPlayhead() {
    if (measureKeyFifths.isEmpty || measureStartMs.isEmpty) return keyFifths;
    var idx = 0;
    for (var m = 0; m < measureStartMs.length; m++) {
      if (measureStartMs[m] <= elapsedMs) {
        idx = m;
      } else {
        break;
      }
    }
    return idx < measureKeyFifths.length ? measureKeyFifths[idx] : keyFifths;
  }

  /// The clef (sign, line) in effect on [staff] at the current playhead — the
  /// latest note at/before [elapsedMs], else the first note on that staff.
  (String, int) _clefAtPlayhead(int staff) {
    TimedNote? chosen;
    for (final n in notes) {
      if (n.staff != staff) continue;
      chosen ??= n; // fallback: first note on the staff
      if (n.startMs <= elapsedMs) chosen = n; // latest before the playhead
    }
    if (chosen == null) return (staff >= 2 ? 'F' : 'G', staff >= 2 ? 4 : 2);
    return (chosen.clefSign, chosen.clefLine);
  }

  /// Written-diatonic staff step for a MIDI pitch (fallback when a note carries
  /// no spelled [TimedNote.diatonic]). Uses the musical octave (`pitch~/12 - 1`,
  /// so MIDI 60 = C4) to match [TimedNote.diatonic]'s `octave*7 + step` scale.
  /// Enharmonics collapse (A♭→G) — acceptable for the MIDI-only demo/replay.
  int _diatonic(int pitch) {
    const whiteInOctave = {
      0: 0,
      1: 0,
      2: 1,
      3: 1,
      4: 2,
      5: 3,
      6: 3,
      7: 4,
      8: 4,
      9: 5,
      10: 5,
      11: 6,
    };
    final octave = pitch ~/ 12 - 1;
    final semitone = pitch % 12;
    return octave * 7 + (whiteInOctave[semitone] ?? 0);
  }

  /// SMuFL note head (plus a soft glow when under the playhead). [glyph] selects
  /// the head shape (filled/half/whole) for the note's duration.
  void _drawHead(
    Canvas canvas,
    Offset center,
    double lineGap,
    bool emphasized,
    Color color,
    String glyph,
  ) {
    if (emphasized) {
      canvas.drawCircle(
        center,
        lineGap * 1.1,
        Paint()
          ..color = color.withValues(alpha: 0.45)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
    }
    final headLeft = center.dx - Smufl.noteheadWidth * lineGap / 2;
    Smufl.draw(canvas, glyph, headLeft, center.dy, lineGap, color);
  }

  /// Notehead glyph for a note: open for half/whole, filled otherwise. Uses the
  /// parsed [TimedNote.noteType] when present, else infers from the duration
  /// (in [quarterMs] units) — matching the engraved Partition view.
  String _headGlyph(TimedNote n, double quarterMs) {
    switch (n.noteType) {
      case 'whole':
        return Smufl.noteheadWhole;
      case 'half':
        return Smufl.noteheadHalf;
      case null:
        final ratio = quarterMs > 0 ? n.durationMs / quarterMs : 1.0;
        if (ratio >= 3.5) return Smufl.noteheadWhole; // ~whole (4 beats)
        if (ratio >= 1.5) return Smufl.noteheadHalf; // ~half / dotted half
        return Smufl.noteheadBlack;
      default:
        return Smufl.noteheadBlack;
    }
  }

  /// Rest glyph for a rest: from its [TimedRest.noteType] when present, else
  /// inferred from the duration (in [quarterMs] units).
  String _restGlyph(TimedRest r, double quarterMs) {
    switch (r.noteType) {
      case 'whole':
        return Smufl.restWhole;
      case 'half':
        return Smufl.restHalf;
      case 'quarter':
        return Smufl.restQuarter;
      case 'eighth':
        return Smufl.rest8th;
      case '16th':
        return Smufl.rest16th;
      case null:
        final ratio = quarterMs > 0 ? r.durationMs / quarterMs : 1.0;
        if (ratio >= 3.5) return Smufl.restWhole;
        if (ratio >= 1.5) return Smufl.restHalf;
        if (ratio <= 0.32) return Smufl.rest16th;
        if (ratio <= 0.62) return Smufl.rest8th;
        return Smufl.restQuarter;
      default:
        return Smufl.restQuarter;
    }
  }

  /// Augmentation dots to the right of a note head or rest.
  void _drawDots(
    Canvas canvas,
    Offset center,
    double lineGap,
    int dots,
    Color color,
  ) {
    for (var i = 0; i < dots; i++) {
      Smufl.draw(
        canvas,
        Smufl.augmentationDot,
        center.dx +
            Smufl.noteheadWidth * lineGap / 2 +
            lineGap * (0.3 + i * 0.5),
        center.dy,
        lineGap,
        color,
      );
    }
  }

  /// Stem length beyond the head anchor, in staff spaces (a standard octave).
  static const double _stemLen = 3.2;

  /// Stem and (for unbeamed notes) flags, attached at the head anchor on the
  /// side the stem points to.
  void _drawStemFlag(
    Canvas canvas,
    Offset center,
    double lineGap,
    Color color,
    int flags,
    bool up,
  ) {
    final anchor = _stemAnchor(center, lineGap, up);
    final tipY = up
        ? anchor.dy - lineGap * _stemLen
        : anchor.dy + lineGap * _stemLen;
    canvas.drawLine(
      anchor,
      Offset(anchor.dx, tipY),
      Paint()
        ..color = color
        ..strokeWidth = Smufl.stemThickness * lineGap
        ..strokeCap = StrokeCap.round,
    );
    if (flags > 0) {
      final glyph = flags >= 2
          ? (up ? Smufl.flag16thUp : Smufl.flag16thDown)
          : (up ? Smufl.flag8thUp : Smufl.flag8thDown);
      Smufl.draw(canvas, glyph, anchor.dx, tipY, lineGap, color);
    }
  }

  /// Stem-attachment point on the note head for the given direction: the head's
  /// right edge going up, its left edge going down.
  Offset _stemAnchor(Offset center, double lineGap, bool up) {
    final headLeft = center.dx - Smufl.noteheadWidth * lineGap / 2;
    return up
        ? Offset(
            headLeft + Smufl.stemUpAnchorX * lineGap,
            center.dy - Smufl.stemUpAnchorY * lineGap,
          )
        : Offset(
            headLeft + Smufl.stemDownAnchorX * lineGap,
            center.dy - Smufl.stemDownAnchorY * lineGap,
          );
  }

  /// One straight beam across a group of note heads, on the side [up] points
  /// to, with stems of varying length reaching it (matching the Partition
  /// engraving), plus a secondary beam for consecutive sixteenths.
  void _drawBeam(
    Canvas canvas,
    List<Offset> pts,
    List<TimedNote> group,
    double lineGap,
    bool up,
  ) {
    final color = palette.staffLine.withValues(alpha: 0.75);
    final quarterMs = bpm > 0 ? 60000.0 / bpm : 500.0;
    final stemPaint = Paint()
      ..color = color
      ..strokeWidth = Smufl.stemThickness * lineGap
      ..strokeCap = StrokeCap.round;

    final anchors = pts.map((p) => _stemAnchor(p, lineGap, up)).toList();
    // Flat beam clearing the extreme head of the group by the stem length.
    final beamY = up
        ? anchors.map((a) => a.dy).reduce(math.min) - lineGap * _stemLen
        : anchors.map((a) => a.dy).reduce(math.max) + lineGap * _stemLen;
    for (final a in anchors) {
      canvas.drawLine(a, Offset(a.dx, beamY), stemPaint);
    }
    canvas.drawLine(
      Offset(anchors.first.dx, beamY),
      Offset(anchors.last.dx, beamY),
      Paint()
        ..color = color
        ..strokeWidth = Smufl.beamThickness * lineGap,
    );
    // Secondary beam between consecutive sixteenths (duration < a third beat),
    // stacked on the staff side of the primary beam.
    bool isSixteenth(TimedNote n) => n.durationMs / quarterMs <= 0.32;
    final off = (up ? 1.0 : -1.0) * (Smufl.beamThickness + 0.2) * lineGap;
    for (var i = 0; i < group.length - 1; i++) {
      if (isSixteenth(group[i]) && isSixteenth(group[i + 1])) {
        canvas.drawLine(
          Offset(anchors[i].dx, beamY + off),
          Offset(anchors[i + 1].dx, beamY + off),
          Paint()
            ..color = color
            ..strokeWidth = Smufl.beamThickness * lineGap,
        );
      }
    }
  }

  void _drawLedgerLines(
    Canvas canvas,
    double x,
    double y,
    double bottomLineY,
    double topLineY,
    double lineGap,
    Paint linePaint,
  ) {
    for (var ly = bottomLineY + lineGap; ly <= y + 0.5; ly += lineGap) {
      canvas.drawLine(
        Offset(x - lineGap, ly),
        Offset(x + lineGap, ly),
        linePaint,
      );
    }
    for (var ly = topLineY - lineGap; ly >= y - 0.5; ly -= lineGap) {
      canvas.drawLine(
        Offset(x - lineGap, ly),
        Offset(x + lineGap, ly),
        linePaint,
      );
    }
  }

  @override
  bool shouldRepaint(StaffPainter old) =>
      old.elapsedMs != elapsedMs ||
      old.activeNotes != activeNotes ||
      old.notes != notes ||
      old.rests != rests ||
      old.tieContinuations != tieContinuations ||
      old.measureStartMs != measureStartMs ||
      old.measureKeyFifths != measureKeyFifths ||
      old.mistakeColors != mistakeColors ||
      old.lookAheadMs != lookAheadMs ||
      old.onsetGapMs != onsetGapMs ||
      old.measureMs != measureMs ||
      old.noteScale != noteScale ||
      old.palette != palette ||
      old.practiceRange != practiceRange ||
      old.showPlayhead != showPlayhead ||
      old.timeOriginFraction != timeOriginFraction;
}
