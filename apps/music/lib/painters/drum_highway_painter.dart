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

/// EXPERIMENT — a perspective "highway" reading of the drum cascade.
///
/// Not wired into the app: this exists to be looked at beside the flat
/// cascade. The flat one answers "when", precisely; this one answers "which
/// limb, on which drum" first, by putting the notes on rails that land on a
/// drawn kit. Everything else — lane derivation, the hands/feet colour
/// convention, the kick as a full-width event — is unchanged, so the two are
/// two readings of one model rather than two models.
library;

import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import '../state/drum_kit.dart';
import 'drum_kit_art.dart';
import '../state/player_data.dart';
import '../theme/cymbra_theme.dart';

class DrumHighwayPainter extends CustomPainter {
  const DrumHighwayPainter({
    required this.lanes,
    required this.notes,
    required this.multiVoice,
    required this.elapsedMs,
    this.lookAheadMs = 3000,
    this.struckMs = const <int, double>{},
    this.nowMs = 0,
    this.laneLabels = const [],
    this.kickLabel = '',
    this.measureStartMs = const [],
    this.beatMs = 0,
    this.expectedSurfaces = const {},
    this.waitPulse = 0,
    this.hasKick = true,
    this.writtenMeasureOf = const [],
    this.speed = 1,
    this.playableSurfaces = const {},
    this.labelStyle,
  });

  final List<DrumLane> lanes;
  final List<TimedNote> notes;
  final bool multiVoice;
  final double elapsedMs;
  final double lookAheadMs;

  /// Wall-clock stamps of the last strike per lane index (and [kPedalSurface]
  /// for the kick), so a hit lights its rail — the same struck table the pad
  /// strip reads.
  final Map<int, double> struckMs;
  final double nowMs;

  /// Resolved (localised) lane names, drawn at the head of each rail. Passed
  /// in because a painter has no l10n; empty draws none.
  final List<String> laneLabels;

  /// The kick's localised name, drawn under its drum.
  final String kickLabel;

  /// Start time of each played measure — the bar lines. From the score, never
  /// from a wall clock: a grid on round numbers of milliseconds lands on no
  /// beat of any real tempo, and then every note looks displaced against it.
  final List<int> measureStartMs;

  /// One beat of the piece's metre, in ms (60000/bpm scaled by the beat type).
  /// Zero draws beats not at all rather than wrongly.
  final double beatMs;

  /// Surfaces Wait Mode is waiting for — they pulse on the kit, which is now
  /// where the player looks and strikes (the pad strip is gone in this mode).
  final Set<int> expectedSurfaces;

  /// Breathing amplitude of the awaited pieces while Wait Mode holds.
  final double waitPulse;

  /// Whether the score uses the kick at all.
  final bool hasKick;

  /// The WRITTEN measure each played slot performs, aligned with
  /// [measureStartMs] — an unrolled repeat numbers its bars the way the paper
  /// does. Empty = identity.
  final List<int> writtenMeasureOf;

  /// Transport speed, only ever used to size the stroke tolerance window — the
  /// surface must light a stroke on exactly the window the gate accepts.
  final double speed;

  /// The surfaces the hands/feet selection can play; the rest are drawn faded.
  final Set<int> playableSurfaces;

  /// Style for [laneLabels]; the caller passes the app's own text style.
  final TextStyle? labelStyle;

  /// Where the horizon sits, as a fraction of the height. The notes are born
  /// here and grow as they approach.
  static const double _horizon = 0.12;

  /// Where the hit line sits — the kit is drawn below it.
  static const double _hitLine = 0.62;

  /// How wide the rails are at the horizon, relative to their spread at the
  /// hit line: the smaller, the deeper the perspective.
  static const double _vanish = 0.28;

  /// True 1/z projection: an object twice as far is half the size, which is
  /// also what spaces the notes correctly — wide apart as they arrive, packed
  /// as they recede. A power curve does the opposite near the hit line (it
  /// bunches the arrivals), which is the giveaway of a fake perspective.
  double _scaleAt(double d) {
    const k = 1 / _vanish - 1; // so scale(1) == _vanish
    // Clamped BELOW zero, not at it: a note just past the hit line is nearer
    // than the line, so it keeps growing and sliding out of frame. Clamping at
    // 0 is what used to park it on the line.
    return 1 / (1 + k * d.clamp(-0.12, 1.0));
  }

  /// Maps a time offset (0 = at the hit line, 1 = at the horizon) to a
  /// vertical position, derived from the same projection so size and position
  /// agree.
  double _depthToY(double d, double h) {
    final hit = h * _hitLine;
    final hor = h * _horizon;
    final t = (1 - _scaleAt(d)) / (1 - _vanish); // 0 at the hit line, 1 far
    return hit - (hit - hor) * t;
  }

  /// Centre x of lane [i] at depth [d].
  ///
  /// At the hit line the rail is exactly where the DRAWN piece is — the kit
  /// owns the horizontal layout (it straddles the bass drum when the row is
  /// sparse), and a rail computed from anything else would drop its notes
  /// beside the drum they name. Further away the offset shrinks with the same
  /// 1/z scale, so the rails converge on the vanishing point.
  double _laneX(int i, double d, Size size) {
    final centre = size.width / 2;
    return centre + (kitArtFor(size).centreXOf(i) - centre) * _scaleAt(d);
  }

  @override
  void paint(Canvas canvas, Size size) {
    _paintBackdrop(canvas, size);
    _paintKit(canvas, size);
    _paintRails(canvas, size);
    _paintKickEvents(canvas, size);
    // After the kick lines, before the pucks: a downbeat almost always
    // carries a kick, and a grid line under it is a reference you cannot see.
    _paintMetre(canvas, size);
    _paintNotes(canvas, size);
  }

  // --- backdrop -------------------------------------------------------------

  void _paintBackdrop(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    canvas.drawRect(
      rect,
      Paint()
        ..shader = ui.Gradient.linear(
          Offset(size.width / 2, 0),
          Offset(size.width / 2, size.height),
          const [Color(0xFF120E24), Color(0xFF0A0817), Color(0xFF080611)],
          const [0.0, 0.55, 1.0],
        ),
    );

    // The floor: a trapezoid between the horizon and the hit line, faintly lit
    // so the rails read as lying ON something.
    final floor = Path()
      ..moveTo(_laneEdge(-1, 1.0, size), _depthToY(1.0, size.height))
      ..lineTo(_laneEdge(1, 1.0, size), _depthToY(1.0, size.height))
      ..lineTo(_laneEdge(1, 0.0, size), _depthToY(0.0, size.height))
      ..lineTo(_laneEdge(-1, 0.0, size), _depthToY(0.0, size.height))
      ..close();
    canvas.drawPath(
      floor,
      Paint()
        ..shader = ui.Gradient.linear(
          Offset(0, _depthToY(1.0, size.height)),
          Offset(0, _depthToY(0.0, size.height)),
          const [Color(0x14FFFFFF), Color(0x05FFFFFF)],
        ),
    );
  }

  /// The grid IS the metre: a bright line per bar (from the score's own
  /// measure table) and a faint one per beat inside it. Nothing here is
  /// derived from round millisecond values — that was the bug that made every
  /// note look off the beat.
  void _paintMetre(Canvas canvas, Size size) {
    // Bar and beat are told apart the way a staff tells them apart: the bar
    // line spans the full width of the highway and glows, the beat is a
    // shorter, dimmer stroke inside it. Both thin with distance, like every
    // other object on the floor.
    void rule(double t, {required bool bar, int? number}) {
      final d = (t - elapsedMs) / lookAheadMs;
      if (d < 0 || d > 1) return;
      final y = _depthToY(d, size.height);
      final fade = 1.0 - d * 0.55;
      final left = _laneEdge(-1, d, size);
      final right = _laneEdge(1, d, size);
      final near = 0.45 + 0.55 * _scaleAt(d);
      if (bar) {
        canvas.drawLine(
          Offset(left, y),
          Offset(right, y),
          Paint()
            ..color = Colors.white.withValues(alpha: 0.13 * fade)
            ..strokeWidth = 6 * near
            ..maskFilter = MaskFilter.blur(BlurStyle.normal, 3 * near),
        );
      }
      final inset = bar ? 0.0 : (right - left) * 0.14;
      canvas.drawLine(
        Offset(left + inset, y),
        Offset(right - inset, y),
        Paint()
          ..color = Colors.white.withValues(alpha: (bar ? 0.62 : 0.26) * fade)
          ..strokeWidth = (bar ? 2.6 : 1.3) * near,
      );
      // The bar's number rides the line INTO the distance with it, so a bar
      // line never reads as one more rule — and it says where in the piece
      // the run is.
      if (bar && number != null) {
        _measureNumber(canvas, number, left - 6, y, near, fade);
      }
    }

    final horizonMs = elapsedMs + lookAheadMs;
    for (var m = 0; m < measureStartMs.length; m++) {
      final start = measureStartMs[m].toDouble();
      if (start > horizonMs) break;
      final end = m + 1 < measureStartMs.length
          ? measureStartMs[m + 1].toDouble()
          : start + (beatMs > 0 ? beatMs * 4 : lookAheadMs);
      if (end < elapsedMs) continue;
      rule(start, bar: true, number: measureNumberAt(m));
      if (beatMs <= 0) continue;
      for (var t = start + beatMs; t < end - 1; t += beatMs) {
        rule(t, bar: false);
      }
    }
  }

  /// The bar's number as the paper writes it (1-based).
  int measureNumberAt(int slot) =>
      (slot >= 0 && slot < writtenMeasureOf.length
          ? writtenMeasureOf[slot]
          : slot) +
      1;

  /// Right-aligned just outside the highway's left edge, shrinking and fading
  /// with the line it belongs to.
  void _measureNumber(
    Canvas canvas,
    int number,
    double right,
    double y,
    double near,
    double fade,
  ) {
    final tp = TextPainter(
      text: TextSpan(
        text: '$number',
        style: TextStyle(
          color: Colors.white.withValues(alpha: 0.6 * fade),
          fontSize: 13 * near,
          fontWeight: FontWeight.w600,
          fontFamily: labelStyle?.fontFamily,
          fontFeatures: const [ui.FontFeature.tabularFigures()],
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(canvas, Offset(right - tp.width, y - tp.height / 2));
  }

  /// x of the outer edge of the highway on side [side] (-1 left, 1 right).
  double _laneEdge(int side, double d, Size size) {
    final n = math.max(lanes.length, 1);
    final spread = size.width * 0.78;
    final step = spread / n;
    final centre = size.width / 2;
    return centre + side * (spread / 2 + step * 0.1) * _scaleAt(d);
  }

  // --- rails ----------------------------------------------------------------

  void _paintRails(Canvas canvas, Size size) {
    for (var i = 0; i < lanes.length; i++) {
      final hue = kLaneHues[i % kLaneHues.length];
      final top = Offset(_laneX(i, 1.0, size), _depthToY(1.0, size.height));
      final bottom = Offset(_laneX(i, 0.0, size), _depthToY(0.0, size.height));
      // Lit when the lane was struck recently.
      final lit = _struckIntensity(i);
      _neonLine(
        canvas,
        top,
        bottom,
        hue,
        width: 2.5 + 2 * lit,
        glow: 0.5 + lit,
      );
    }
    // The hit line itself: where the rails meet the kit.
    final y = _depthToY(0, size.height);
    _neonLine(
      canvas,
      Offset(_laneEdge(-1, 0, size), y),
      Offset(_laneEdge(1, 0, size), y),
      CymbraColors.primary,
      width: 2,
      glow: 0.6,
    );
  }

  double _struckIntensity(int surface) {
    final at = struckMs[surface];
    if (at == null) return 0;
    const flashMs = 180.0;
    final age = nowMs - at;
    if (age < 0 || age > flashMs) return 0;
    return 1 - age / flashMs;
  }

  /// A neon stroke: a wide soft pass under a bright thin core.
  void _neonLine(
    Canvas canvas,
    Offset a,
    Offset b,
    Color color, {
    required double width,
    required double glow,
  }) {
    canvas.drawLine(
      a,
      b,
      Paint()
        ..color = color.withValues(alpha: 0.28 * glow)
        ..strokeWidth = width * 4
        ..maskFilter = MaskFilter.blur(BlurStyle.normal, width * 2),
    );
    canvas.drawLine(
      a,
      b,
      Paint()
        ..color = color.withValues(alpha: 0.95)
        ..strokeWidth = width
        ..strokeCap = StrokeCap.round,
    );
  }

  // --- notes ----------------------------------------------------------------

  void _paintNotes(Canvas canvas, Size size) {
    // Past the hit line the puck KEEPS GOING — it slides under the kit and
    // fades over the tolerance window instead of parking on the line. A note
    // that stops where it landed reads as still owed.
    final tolerance = strokeToleranceMsAt(speed);
    final tail = lookAheadMs > 0 ? tolerance / lookAheadMs : 0.0;
    // Far first, so nearer (bigger) notes overlap them.
    final visible =
        notes
            .where((n) => !kKickGmNumbers.contains(n.pitch))
            .map((n) => (n, (n.startMs - elapsedMs) / lookAheadMs))
            .where((e) => e.$2 >= -tail && e.$2 <= 1.0)
            .toList()
          ..sort((a, b) => b.$2.compareTo(a.$2));

    for (final (note, d) in visible) {
      final lane = laneIndexOf(lanes, note.pitch);
      if (lane == null) continue;
      final hue = kLaneHues[lane % kLaneHues.length];
      final foot = isFootNote(note, multiVoice: multiVoice);
      // The ONLY thing that lights a puck is the player striking its piece
      // while the note is inside the tolerance window. Nothing lights on
      // arrival: a note that lights itself teaches the surface to be read
      // instead of the beat to be felt, and it makes a good hit and a lucky
      // one look the same.
      final struck =
          (note.startMs - elapsedMs).abs() <= tolerance &&
          _recentlyStruck(lane);
      _paintPuck(
        canvas,
        size,
        lane: lane,
        depth: d,
        color: foot ? kKickHue : hue,
        open: isOpenHiHat(note.pitch),
        struck: struck,
        fade: d < 0 && tail > 0 ? (1 + d / tail).clamp(0.0, 1.0) : 1.0,
      );
    }
  }

  bool _recentlyStruck(int surface) {
    final at = struckMs[surface];
    return at != null && struckFlashIntensity(struckMs: at, nowMs: nowMs) > 0;
  }

  /// One note as a 3D-ish puck: an elliptical top face and a short body, so a
  /// stack of them reads as objects sliding toward the player rather than
  /// rectangles scrolling.
  void _paintPuck(
    Canvas canvas,
    Size size, {
    required int lane,
    required double depth,
    required Color color,
    required bool open,
    bool struck = false,
    double fade = 1.0,
  }) {
    final s = _scaleAt(depth);
    final x = _laneX(lane, depth, size);
    final y = _depthToY(depth, size.height);
    final n = math.max(lanes.length, 1);
    final w = math.min((size.width * 0.78 / n) * 0.46, size.height * 0.16) * s;
    final h = w * 0.42;
    final body = h * 0.62;

    final top = Rect.fromCenter(
      center: Offset(x, y - body),
      width: w,
      height: h,
    );
    final bottom = Rect.fromCenter(center: Offset(x, y), width: w, height: h);

    // One quiet halo, always the same size: it seats the puck on the floor,
    // it never announces anything.
    canvas.drawOval(
      bottom.inflate(w * 0.18),
      Paint()
        ..color = color.withValues(alpha: 0.22 * fade)
        ..maskFilter = MaskFilter.blur(BlurStyle.normal, w * 0.22),
    );

    // Body: the side wall between the two faces.
    final side = Path()
      ..moveTo(top.left, top.center.dy)
      ..lineTo(bottom.left, bottom.center.dy)
      ..arcTo(bottom, math.pi, -math.pi, false)
      ..lineTo(top.right, top.center.dy)
      ..arcTo(top, 0, -math.pi, false)
      ..close();
    canvas.drawPath(
      side,
      Paint()
        ..shader = ui.Gradient.linear(
          Offset(top.left, 0),
          Offset(top.right, 0),
          [
            color.withValues(alpha: 0.55 * fade),
            color.withValues(alpha: 0.95 * fade),
            color.withValues(alpha: 0.55 * fade),
          ],
          const [0.0, 0.45, 1.0],
        ),
    );

    // Top face — hollow for an open hi-hat, exactly as the flat cascade marks
    // it, so the one musical distinction survives the restyle.
    canvas.drawOval(
      top,
      open
          ? (Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = math.max(1.4, w * 0.06)
              ..color = Colors.white.withValues(alpha: 0.92 * fade))
          : (Paint()
              ..color = Color.lerp(
                color,
                Colors.white,
                struck ? 0.85 : 0.45,
              )!.withValues(alpha: fade)),
    );
  }

  // --- the kick, as a full-width event -------------------------------------

  void _paintKickEvents(Canvas canvas, Size size) {
    for (final note in notes) {
      if (!kKickGmNumbers.contains(note.pitch)) continue;
      final d = (note.startMs - elapsedMs) / lookAheadMs;
      if (d < -0.05 || d > 1) continue;
      final y = _depthToY(d, size.height);
      final a = (1 - d).clamp(0.15, 1.0) * 0.9;
      _neonLine(
        canvas,
        Offset(_laneEdge(-1, d, size), y),
        Offset(_laneEdge(1, d, size), y),
        kKickHue,
        width: 2 + 3 * (1 - d),
        glow: a,
      );
    }
  }

  // --- the kit --------------------------------------------------------------

  /// A stylised neon kit under the hit line: one drawn piece per rail, so a
  /// falling note visibly lands ON the drum it names.
  void _paintKit(Canvas canvas, Size size) {
    kitArtFor(size).paint(
      canvas,
      struck: struckMs,
      nowMs: nowMs,
      expected: expectedSurfaces,
      waitPulse: waitPulse,
      playable: playableSurfaces,
      labels: laneLabels,
      kickLabel: kickLabel,
      labelStyle: labelStyle,
    );
  }

  /// The kit's geometry for this surface — the screen uses the SAME instance's
  /// hit areas, so what is struck is exactly what is drawn. Cheap enough to
  /// rebuild per call — it holds no state, only geometry — and the painter is
  /// const, so it cannot memoise it anyway.
  DrumKitArt kitArtFor(Size size) => DrumKitArt(
    lanes: lanes,
    size: size,
    top: _depthToY(0, size.height),
    hasKick: hasKick,
  );

  @override
  bool shouldRepaint(covariant DrumHighwayPainter old) =>
      old.elapsedMs != elapsedMs ||
      old.notes != notes ||
      old.lanes != lanes ||
      old.struckMs != struckMs ||
      old.nowMs != nowMs;
}
