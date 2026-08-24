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
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import '../state/drum_kit.dart';
import 'drum_kit_art.dart';
import '../state/player_data.dart';
import '../theme/cymbra_theme.dart';

/// The percussion cascade (change: add-drum-kit-view): one lane per kit piece
/// the score actually uses, notes falling toward the hit line — and the kick
/// as a **full-width bar**, never a lane. A lane encodes *where to aim*; the
/// foot does not aim, so a bar answers the only question that matters — does
/// the foot land with the hand or between? — by intersection, in peripheral
/// vision.
///
/// Draw order is load-bearing, not cosmetic: the foot bars paint FIRST
/// (beneath the note layer), thinner than a note and attenuated. Painted
/// above, a bar hides the hand note exactly on a coincidence — the very
/// information it exists to convey — and the defect is invisible on any score
/// where feet and hands never align, which is why the paint order is pinned
/// by a test on a coinciding onset.
class DrumCascadePainter extends CustomPainter {
  /// The lanes in presentation order ([PlayerData.presentedDrumLanes]) — the
  /// same list the pad strip renders, so the two surfaces are one mapping.
  final List<DrumLane> lanes;

  /// The visible notes (kick notes included — they become the bars).
  final List<TimedNote> notes;

  /// Whether the score spans more than one voice (precomputed once): the
  /// hands/feet split keys on voice, with the GM fallback for single-voice
  /// files.
  final bool multiVoice;

  /// Playhead in milliseconds.
  final double elapsedMs;

  /// Visible time window (how many ms span the height).
  final double lookAheadMs;

  const DrumCascadePainter({
    required this.lanes,
    required this.notes,
    required this.multiVoice,
    required this.elapsedMs,
    this.lookAheadMs = 3000,
    this.measureStartMs = const [],
    this.beatMs = 0,
    this.struckMs = const <int, double>{},
    this.nowMs = 0,
    this.expectedSurfaces = const {},
    this.waitPulse = 0,
    this.hasKick = true,
    this.writtenMeasureOf = const [],
    this.speed = 1,
    this.playableSurfaces = const {},
    this.laneLabels = const [],
    this.kickLabel = '',
    this.labelStyle,
  });

  /// EXPERIMENT (drum-highway): the metre, the drawn kit and its labels moved
  /// INTO the cascade — the strip of labelled rectangles under it said the
  /// same thing twice, and put what you strike away from what you read.

  /// Start of each played measure — the bar rules.
  final List<int> measureStartMs;

  /// One beat of the piece's metre, in ms; zero draws beats not at all.
  final double beatMs;

  /// Wall-clock stamps of the last strike per surface, for the kit's flash.
  final Map<int, double> struckMs;
  final double nowMs;

  /// Surfaces Wait Mode is waiting for (they pulse on the kit).
  final Set<int> expectedSurfaces;

  /// Breathing amplitude of the awaited pieces while Wait Mode holds.
  final double waitPulse;
  final bool hasKick;

  /// The WRITTEN measure each played slot performs, aligned with
  /// [measureStartMs] — so an unrolled repeat numbers its bars the way the
  /// paper does (bar 5 played twice is bar 5 both times). Empty = identity.
  final List<int> writtenMeasureOf;

  /// Transport speed, only ever used to size the stroke tolerance window — the
  /// surface must light a stroke on exactly the window the gate accepts.
  final double speed;

  /// The surfaces the hands/feet selection can play; the rest are drawn faded.
  final Set<int> playableSurfaces;
  final List<String> laneLabels;
  final String kickLabel;
  final TextStyle? labelStyle;

  /// Bar thickness (px) of a kick bar — deliberately thinner than any note
  /// body, so the layering reads as layering rather than occlusion.
  static const double kickBarHeight = 7.0;

  /// Attenuation of the foot bars relative to a note (~60% per the settled
  /// mockup). The fade alone would not suffice — a bar faint enough to reveal
  /// a note behind it disappears where it has nothing behind it — which is
  /// why thickness and paint order carry the rest.
  static const double kickBarAlpha = 0.62;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = CymbraColors.background,
    );

    // The kit takes the bottom band; the falling surface ends on its heads.
    final art = kitArtFor(size);
    final hitLineY = this.hitLineY(size);
    final pxPerMs = hitLineY / lookAheadMs;

    // The hit line, made unmissable: it is the whole answer to "when do I
    // strike", and a hairline the same weight as the beat rules did not say
    // it. A wide glow under a bright bar, plus a lit segment per lane so the
    // rail visibly ENDS somewhere rather than fading out.
    canvas.drawRect(
      Rect.fromLTRB(0, hitLineY - 14, size.width, hitLineY + 2),
      Paint()
        ..shader = ui.Gradient.linear(
          Offset(0, hitLineY - 14),
          Offset(0, hitLineY + 2),
          [
            CymbraColors.primary.withValues(alpha: 0.0),
            CymbraColors.primary.withValues(alpha: 0.28),
          ],
        ),
    );
    canvas.drawLine(
      Offset(0, hitLineY),
      Offset(size.width, hitLineY),
      Paint()
        ..color = CymbraColors.primary.withValues(alpha: 0.35)
        ..strokeWidth = 9
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
    );
    canvas.drawLine(
      Offset(0, hitLineY),
      Offset(size.width, hitLineY),
      Paint()
        ..color = Colors.white.withValues(alpha: 0.85)
        ..strokeWidth = 2.5,
    );
    for (var i = 0; i < lanes.length; i++) {
      final x = art.centreXOf(i);
      final hue = kLaneHues[i % kLaneHues.length];
      final half = (size.width / math.max(lanes.length, 1)) * 0.22;
      canvas.drawLine(
        Offset(x - half, hitLineY),
        Offset(x + half, hitLineY),
        Paint()
          ..color = hue
          ..strokeWidth = 5
          ..strokeCap = StrokeCap.round,
      );
    }

    // The old subtle rule, kept beneath as the surface's own edge.
    canvas.drawLine(
      Offset(0, hitLineY - 1),
      Offset(size.width, hitLineY - 1),
      Paint()
        ..color = CymbraColors.outlineVariant.withValues(alpha: 0.6)
        ..strokeWidth = 1,
    );

    // The metre, from the score itself: a bright rule per bar, faint ones on
    // the beats. Round millisecond spacing lands on no real tempo, so a
    // correctly-placed note would read as displaced against it.
    // A bar line crosses the WHOLE surface and carries a soft glow; a beat is
    // a shorter, dimmer stroke inside the lanes. That is the hierarchy an
    // engraved staff already uses — length first, brightness second — so the
    // two never blur into one grey grid.
    void rule(double t, {required bool bar, int? number}) {
      final y = hitLineY - (t - elapsedMs) * pxPerMs;
      if (y < 0 || y > hitLineY) return;
      if (bar) {
        canvas.drawLine(
          Offset(0, y),
          Offset(size.width, y),
          Paint()
            ..color = Colors.white.withValues(alpha: 0.10)
            ..strokeWidth = 6
            ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3),
        );
      }
      final inset = bar ? 0.0 : size.width * 0.12;
      canvas.drawLine(
        Offset(inset, y),
        Offset(size.width - inset, y),
        Paint()
          ..color = Colors.white.withValues(alpha: bar ? 0.55 : 0.24)
          ..strokeWidth = bar ? 2.2 : 1.2,
      );
      if (bar && number != null) _measureNumber(canvas, number, 12, y - 20);
    }

    final horizonMs = elapsedMs + lookAheadMs;
    // Drawn AFTER the kick bars (see below), not here: a downbeat almost
    // always carries a kick, and a grid line under a full-width bar is a
    // reference you cannot see. The strokes still land on top of both.
    void paintMetre() {
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

    // A rail from each drawn piece up through the falling surface: the lane
    // is no longer an anonymous column, it belongs to the drum it lands on.
    for (var i = 0; i < lanes.length; i++) {
      final x = art.centreXOf(i);
      final hue = kLaneHues[i % kLaneHues.length];
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, hitLineY),
        Paint()
          ..color = hue.withValues(alpha: 0.30)
          ..strokeWidth = 1.5,
      );
    }

    double bottomYOf(TimedNote n) =>
        hitLineY - (n.startMs - elapsedMs) * pxPerMs;

    // --- Pass 1: the foot bars, BENEATH the note layer -------------------
    // The kick (GM 35/36) always; any other foot event with no lane would be
    // a silent drop, so it can only be the pedal hi-hat 44 — which HAS a lane
    // (terminal bucket) and is drawn there as a note by pass 2 instead.
    // The feet's amber — the SAME one the drawn bass drum and its name wear,
    // so the bar reads as that drum rather than as a generic accent.
    const footColor = kKickHue;
    for (final n in notes) {
      if (!kKickGmNumbers.contains(n.pitch)) continue;
      final y = bottomYOf(n);
      if (y < -kickBarHeight || y > hitLineY + kickBarHeight) continue;
      final inHitZone =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      final base = inHitZone
          ? Color.lerp(footColor, const Color(0xFFFFFFFF), 0.35)!
          : footColor;
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(0, y - kickBarHeight, size.width, kickBarHeight),
          const Radius.circular(3),
        ),
        Paint()..color = base.withValues(alpha: kickBarAlpha),
      );
    }

    paintMetre();

    // --- Pass 2: the hand strokes, per lane, on top ----------------------
    // A stroke is an ATTACK, not a held note: a drum's sound decays on its
    // own, so a body whose height is the written duration draws a length the
    // instrument does not have. They are discs on their lane, like the pucks
    // the stage mode slides down the same rails.
    if (lanes.isNotEmpty) {
      final radius = strokeRadius(size);
      for (final n in notes) {
        if (kKickGmNumbers.contains(n.pitch)) continue;
        final lane = laneIndexOf(lanes, n.pitch);
        if (lane == null) continue; // absent from the layout (defensive)
        final y = bottomYOf(n);
        if (y < -radius || y > hitLineY + radius) continue;
        final x = art.centreXOf(lane);
        final foot = isFootNote(n, multiVoice: multiVoice);
        // A stroke wears its LANE's hue — the same colour as the rail it
        // falls down, the piece it lands on, and that piece's name. One
        // colour per drum is what lets the eye aim without reading. A foot
        // stroke inside a lane (a pedalled hi-hat) is that hue pulled toward
        // the feet's amber, so which limb plays it still reads.
        final hue = kLaneHues[lane % kLaneHues.length];
        final limb = foot ? Color.lerp(hue, kKickHue, 0.55)! : hue;
        // A stroke lights ONLY when the player struck its piece inside the
        // tolerance window — never on arrival. A note that lights itself
        // teaches the surface instead of the beat, and makes a good hit look
        // like a lucky one.
        final struck =
            (elapsedMs - n.startMs).abs() <= strokeToleranceMsAt(speed) &&
            _recentlyStruck(lane);
        final base = struck
            ? Color.lerp(limb, const Color(0xFFFFFFFF), 0.75)!
            : limb;
        // CENTRED on its onset line, exactly like the stage's puck: the
        // instant to strike is when the line runs through the middle of the
        // disc. A disc resting ON the line reads as "already passed" — and
        // the two modes have to agree, or the same score teaches two
        // different moments.
        final rect = Rect.fromCenter(
          center: Offset(x, y),
          width: radius * 2.2,
          height: radius * 1.5,
        );
        canvas.drawOval(
          rect.inflate(radius * 0.5),
          Paint()
            ..color = base.withValues(alpha: 0.18)
            ..maskFilter = MaskFilter.blur(BlurStyle.normal, radius * 0.5),
        );
        if (isOpenHiHat(n.pitch)) {
          // The open hi-hat stays a VARIANT inside its lane — hollow where the
          // closed stroke is filled.
          canvas.drawOval(
            rect,
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2.2
              ..color = base,
          );
        } else {
          canvas.drawOval(rect, Paint()..color = base);
        }
      }
    }

    // The kit itself, under the falling surface.
    art.paint(
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

  /// Whether [surface] was struck within the flash window — wall clock, the
  /// same stamps the drawn kit flashes on.
  bool _recentlyStruck(int surface) {
    final at = struckMs[surface];
    return at != null && struckFlashIntensity(struckMs: at, nowMs: nowMs) > 0;
  }

  /// The bar's number as the paper writes it (1-based).
  int measureNumberAt(int slot) =>
      (slot >= 0 && slot < writtenMeasureOf.length
          ? writtenMeasureOf[slot]
          : slot) +
      1;

  /// The bar number, small, at the left margin of its own line: the surest
  /// way to make a bar line read as a BAR line rather than as one more rule —
  /// and it answers "where am I in the piece" at the same time.
  void _measureNumber(Canvas canvas, int number, double x, double y) {
    final tp = TextPainter(
      text: TextSpan(
        text: '$number',
        style: TextStyle(
          color: Colors.white.withValues(alpha: 0.55),
          fontSize: 13,
          fontWeight: FontWeight.w600,
          fontFamily: labelStyle?.fontFamily,
          fontFeatures: const [ui.FontFeature.tabularFigures()],
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(canvas, Offset(x, y));
  }

  /// Where the falling surface ends and the drawn kit begins — the instant
  /// "now", as a y. Named rather than inlined because the rails, the strokes
  /// and the tests all measure from it.
  double hitLineY(Size size) => size.height * 0.66;

  /// Radius of the disc drawn for one stroke.
  double strokeRadius(Size size) => math.min(
    (size.width / math.max(lanes.length, 1)) * 0.22,
    size.height * 0.030,
  );

  /// Centre of the disc drawn for [n]: on its lane's column, sitting on the
  /// line its onset falls on. Public because a pixel probe that recomputes
  /// this arithmetic drifts from the painter the first time the layout moves.
  Offset strokeCentre(TimedNote n, Size size) {
    final lane = laneIndexOf(lanes, n.pitch) ?? 0;
    final hit = hitLineY(size);
    final y = hit - (n.startMs - elapsedMs) * (hit / lookAheadMs);
    return Offset(kitArtFor(size).centreXOf(lane), y);
  }

  /// The kit's geometry for this surface — the screen strikes exactly what is
  /// drawn here.
  DrumKitArt kitArtFor(Size size) => DrumKitArt(
    lanes: lanes,
    size: size,
    top: size.height * 0.66,
    hasKick: hasKick,
  );

  @override
  bool shouldRepaint(DrumCascadePainter old) =>
      old.elapsedMs != elapsedMs ||
      old.notes != notes ||
      old.lanes != lanes ||
      old.multiVoice != multiVoice;
}
