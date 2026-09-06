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

/// EXPERIMENT (drum-highway) — the drawn kit, as ONE object: its layout, its
/// painting, and the hit areas the player strikes.
///
/// It replaces the strip of labelled rectangles under the play surface. That
/// strip said the same thing twice — a row of names under a row of names — and
/// it put the thing you strike somewhere other than the thing you read. Here
/// the drum you aim at, the rail the note falls down, and the name under it
/// are one object.
///
/// Geometry and painting live together on purpose: an input surface computed
/// anywhere but where the drawing happens is a hit area that drifts from what
/// the eye sees, which is the defect this file exists to prevent.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../state/drum_kit.dart';
import '../theme/cymbra_theme.dart';

/// One lane's neon hue, by POSITION: the eye learns "the second piece is the
/// snare" from the drawing, and a score with other pieces still reads the same.
const List<Color> kLaneHues = [
  Color(0xFF5BC8FF), // time keeper — cyan
  Color(0xFF7B6BFF), // snare — indigo
  Color(0xFFC061FF), // toms — violet
  Color(0xFFFF61C3), // cymbals — magenta
  Color(0xFF4DE0C0), // extras — teal
];

/// The feet's hue — the amber both drum surfaces already use.
const Color kKickHue = Color(0xFFFFA742);

/// Everything the kit needs to paint beyond its own geometry — the state of
/// the moment, in one value.
///
/// Bundled rather than passed as eight arguments: three surfaces build exactly
/// the same set (the two play surfaces and the staff), and a positional drift
/// between them would be silent.
class DrumKitPaint {
  const DrumKitPaint({
    required this.struck,
    required this.nowMs,
    this.expected = const {},
    this.waitPulse = 0,
    this.playable = const {},
    this.labels = const [],
    this.kickLabel = '',
    this.labelStyle,
  });

  /// Wall-clock stamps per surface (lane index or [kPedalSurface]).
  final Map<int, double> struck;
  final double nowMs;

  /// The surfaces Wait Mode is waiting for.
  final Set<int> expected;

  /// 0..1 breathing amplitude for the awaited pieces while the gate holds — a
  /// static outline would read as decoration rather than a demand.
  final double waitPulse;

  /// The surfaces this run can actually be played on: the pieces in focus that
  /// the connected kit also has. A surface outside it is drawn faded, because a
  /// kit presenting a live target the run neither awaits nor judges is lying
  /// about the exercise — whether it was left out of the practice selection or
  /// declared missing from the instrument (design D13). Faded rather than
  /// removed: the row keeps its order and its widths, so a player changing
  /// their selection finds every piece where they left it. An empty set means
  /// nothing is excluded at all.
  final Set<int> playable;

  final List<String> labels;
  final String kickLabel;
  final TextStyle? labelStyle;

  /// Flash intensity of [surface], 0 when it was not struck recently.
  double litOf(int surface) {
    final at = struck[surface];
    return at == null ? 0 : struckFlashIntensity(struckMs: at, nowMs: nowMs);
  }

  /// Opacity of [surface]: full when the selection can play it, faded when it
  /// has nothing to do in this exercise.
  double alphaOf(int surface) =>
      playable.isEmpty || playable.contains(surface) ? 1.0 : 0.28;
}

/// Paints a [DrumKitArt] filling its own band — the kit as a STANDALONE
/// surface, for a mode that draws no highway of its own (the staff).
///
/// The strike path uses `artFor(size)` from this same painter, so the notation
/// modes aim at exactly the drums they see, like the two play surfaces.
class DrumKitPainter extends CustomPainter {
  const DrumKitPainter({
    required this.lanes,
    required this.struckMs,
    required this.nowMs,
    this.hasKick = true,
    this.expectedSurfaces = const {},
    this.waitPulse = 0,
    this.playable = const {},
    this.labels = const [],
    this.kickLabel = '',
    this.labelStyle,
  });

  final List<DrumLane> lanes;
  final Map<int, double> struckMs;
  final double nowMs;
  final bool hasKick;
  final Set<int> expectedSurfaces;
  final double waitPulse;
  final Set<int> playable;
  final List<String> labels;
  final String kickLabel;
  final TextStyle? labelStyle;

  DrumKitArt artFor(Size size) =>
      DrumKitArt(lanes: lanes, size: size, top: 0, hasKick: hasKick);

  /// The state the kit is painted in, gathered once.
  DrumKitPaint get _paint => DrumKitPaint(
    struck: struckMs,
    nowMs: nowMs,
    expected: expectedSurfaces,
    waitPulse: waitPulse,
    playable: playable,
    labels: labels,
    kickLabel: kickLabel,
    labelStyle: labelStyle,
  );

  @override
  void paint(Canvas canvas, Size size) {
    // Its own dark ground: the kit is a neon drawing, and the app's light
    // surface underneath would leave it floating on paper.
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = CymbraColors.background,
    );
    artFor(size).paint(canvas, _paint);
  }

  @override
  bool shouldRepaint(covariant DrumKitPainter old) =>
      old.lanes != lanes ||
      old.struckMs != struckMs ||
      old.nowMs != nowMs ||
      old.expectedSurfaces != expectedSurfaces ||
      old.waitPulse != waitPulse ||
      old.playable != playable ||
      old.hasKick != hasKick ||
      old.labels != labels;
}

/// The drawn kit for [lanes], laid out in the band between [top] and the
/// bottom of [size].
class DrumKitArt {
  DrumKitArt({
    required this.lanes,
    required this.size,
    required this.top,
    this.hasKick = true,
  });

  final List<DrumLane> lanes;
  final Size size;

  /// Where the kit band starts (the play surface ends here).
  final double top;

  /// Whether the score uses the kick at all — a kick-less score draws no bass
  /// drum rather than an object nothing can strike.
  final bool hasKick;

  /// The band the PIECES live in: the space under [top], minus a row at the
  /// bottom for the bass drum's name. Without that reservation the kick sits
  /// low enough that its own label falls off the surface.
  double get _free => math.max(size.height - top - _nameRow, 0);

  /// Height reserved under the kit for the lowest name.
  static const double _nameRow = 16;

  /// Horizontal centre of the piece for [lane] — the same spread the rails and
  /// the falling strokes use, so a note lands exactly on its drum.
  ///
  /// With a kick and few pieces, the middle of the row is where the bass drum
  /// already is: a centred piece would be drawn ON it. The row therefore keeps
  /// its left-to-right order but straddles a central gap the kick's width,
  /// which is also where a real kit puts it — bass drum front and centre, the
  /// rest around it.
  double centreXOf(int lane) {
    final n = math.max(lanes.length, 1);
    final spread = size.width * 0.78;
    final centre = size.width / 2;
    final gap = _needsCentreGap ? _kickWidth * 1.15 : 0.0;
    if (gap <= 0) {
      final step = spread / n;
      return centre + (lane - (n - 1) / 2) * step;
    }
    // Half the pieces to each side of the gap (the extra one goes left, so a
    // three-piece kit reads hi-hat + snare | kick | cymbal).
    final leftCount = (n + 1) ~/ 2;
    final side = (spread - gap) / 2;
    if (lane < leftCount) {
      final step = side / leftCount;
      return centre - gap / 2 - side + step * (lane + 0.5);
    }
    final rightCount = n - leftCount;
    final step = side / math.max(rightCount, 1);
    return centre + gap / 2 + step * (lane - leftCount + 0.5);
  }

  /// The gap is only worth its cost while the row is sparse: past that the
  /// pieces are narrow enough to clear the kick on their own, and a hole in
  /// the middle of a wide kit reads as a missing drum.
  bool get _needsCentreGap => hasKick && lanes.length <= 4;

  double get _step => (size.width * 0.78) / math.max(lanes.length, 1);

  bool _isCymbal(DrumLane lane) =>
      lane.role == KitPieceRole.hiHat ||
      lane.role == KitPieceRole.ride ||
      lane.role == KitPieceRole.crash;

  /// The bounding box of the piece drawn for [lane] — also its hit area.
  Rect pieceRect(int lane) {
    final x = centreXOf(lane);
    if (_isCymbal(lanes[lane])) {
      final w = math.min(_step * 0.95, _free * 0.70);
      final h = math.min(_step * 0.24, _free * 0.18);
      // A cymbal is a thin ellipse, so its hit area is grown vertically to
      // something a finger can actually land on.
      return Rect.fromCenter(
        center: Offset(x, top + _free * 0.16),
        width: w,
        height: math.max(h * 3, _free * 0.26),
      );
    }
    final w = math.min(_step * 0.78, _free * 0.58);
    return Rect.fromCenter(
      center: Offset(x, top + _free * 0.40),
      width: w,
      height: _free * 0.30,
    );
  }

  double get _kickWidth => math.min(size.width * 0.22, _step * 1.6);

  Rect get kickRect => Rect.fromCenter(
    center: Offset(size.width / 2, top + _free * 0.72),
    width: _kickWidth,
    height: _free * 0.34,
  );

  /// Which surface a tap at [p] strikes: a lane index, [kPedalSurface] for the
  /// kick, or null outside the kit.
  ///
  /// Pieces are tested BEFORE the kick, and the kick's band spans the full
  /// width below them: the drums overlap the bass drum visually, so the
  /// nearer object has to win, and everything else in the band is still a
  /// kick rather than a dead zone.
  int? hitTest(Offset p) {
    if (p.dy < top || _free <= 0) return null;
    for (var i = 0; i < lanes.length; i++) {
      if (pieceRect(i).inflate(4).contains(p)) return i;
    }
    if (hasKick && p.dy >= top + _free * 0.52) return kPedalSurface;
    // Above the kick band and outside every piece: nearest piece by column, so
    // the gaps between drums are not dead.
    var best = 0;
    var bestDx = double.infinity;
    for (var i = 0; i < lanes.length; i++) {
      final dx = (p.dx - centreXOf(i)).abs();
      if (dx < bestDx) {
        bestDx = dx;
        best = i;
      }
    }
    if (lanes.isEmpty) return null;
    return bestDx <= _step * 0.75 ? best : null;
  }

  /// Paints the kit in the state [p] describes.
  void paint(Canvas canvas, DrumKitPaint p) {
    if (_free <= 24) return;
    _paintPieces(canvas, p);
    _paintNames(canvas, p);
  }

  /// The bass drum, then the drums, then the cymbals over them — how a kit
  /// stacks seen from the stool.
  void _paintPieces(Canvas canvas, DrumKitPaint p) {
    if (hasKick) {
      _drum(
        canvas,
        kickRect,
        kKickHue,
        p.litOf(kPedalSurface),
        awaited: p.expected.contains(kPedalSurface),
        pulse: p.waitPulse,
        fade: p.alphaOf(kPedalSurface),
      );
    }
    // Drums first, cymbals over them.
    for (var pass = 0; pass < 2; pass++) {
      for (var i = 0; i < lanes.length; i++) {
        final cymbal = _isCymbal(lanes[i]);
        if ((pass == 0) == cymbal) continue;
        final draw = cymbal ? _cymbal : _drum;
        draw(
          canvas,
          pieceRect(i),
          kLaneHues[i % kLaneHues.length],
          p.litOf(i),
          awaited: p.expected.contains(i),
          pulse: p.waitPulse,
          fade: p.alphaOf(i),
        );
      }
    }
  }

  /// The names, each hung on its OWN piece — a few pixels from the shape it
  /// names, not on a shared baseline at the bottom of the band. A row of names
  /// lined up under drums of different heights makes the reader match them by
  /// column; on the drum, there is nothing to match.
  ///
  /// When the names are too long for the columns they sit in — "Tom médium
  /// grave" under a seven-piece kit — they are STAGGERED instead of being
  /// ellipsised into stumps: one row above its piece, one below, so each name
  /// has twice the room before it can touch its neighbour. Truncating is the
  /// worse answer, because two toms whose names differ only in their last word
  /// both end up as "Tom médium…".
  void _paintNames(Canvas canvas, DrumKitPaint p) {
    final stagger = _labelsCollide(p.labels, p.labelStyle);
    for (var i = 0; i < lanes.length && i < p.labels.length; i++) {
      _label(
        canvas,
        p.labels[i],
        Offset(centreXOf(i), _labelYOf(i, row: stagger && i.isOdd ? 1 : 0)),
        _nameRoom(i) * (stagger ? 2 : 1),
        kLaneHues[i % kLaneHues.length].withValues(alpha: p.alphaOf(i)),
        p.labelStyle,
      );
    }
    if (hasKick && p.kickLabel.isNotEmpty) {
      _label(
        canvas,
        p.kickLabel,
        Offset(size.width / 2, kickRect.bottom + 4),
        _kickWidth * 1.6,
        kKickHue.withValues(alpha: p.alphaOf(kPedalSurface)),
        p.labelStyle,
      );
    }
  }

  /// Horizontal room a name has before it reaches its neighbour's column.
  /// Derived from the ACTUAL centres — the row straddles the bass drum when it
  /// is sparse, so a width computed from the even spread would overrun.
  double _nameRoom(int lane) {
    var room = double.infinity;
    final x = centreXOf(lane);
    if (lane > 0) room = math.min(room, (x - centreXOf(lane - 1)).abs());
    if (lane < lanes.length - 1) {
      room = math.min(room, (centreXOf(lane + 1) - x).abs());
    }
    if (room == double.infinity) room = size.width * 0.6;
    return math.max(room - 8, 24); // a gutter, so two names never touch
  }

  /// Whether any name is wider than the column it would sit in.
  bool _labelsCollide(List<String> labels, TextStyle? style) {
    for (var i = 0; i < lanes.length && i < labels.length; i++) {
      if (_measure(labels[i], style) > _nameRoom(i)) return true;
    }
    return false;
  }

  double _measure(String text, TextStyle? style) {
    if (text.isEmpty) return 0;
    final tp = TextPainter(
      text: TextSpan(text: text, style: _nameStyle(style, Colors.white)),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout();
    return tp.width;
  }

  TextStyle _nameStyle(TextStyle? style, Color color) =>
      (style ?? const TextStyle()).copyWith(
        color: color,
        fontSize: (style?.fontSize ?? 13).clamp(9.0, 13.0).toDouble(),
        fontWeight: FontWeight.w600,
      );

  /// Top of the name for [lane]: just outside the drawn shape, which for a
  /// cymbal is the thin ellipse inside its (taller) hit area.
  ///
  /// The side is chosen per piece by what is free in ITS column: under the
  /// piece unless the bass drum's band is there, over it otherwise — a name
  /// written across the bass drum reads as the bass drum's. [row] then picks
  /// one of two lines on that side, which is what makes a staggered row
  /// legible: two neighbours never share a line, so each has twice the width.
  double _labelYOf(int lane, {required int row}) {
    final r = pieceRect(lane);
    final cymbal = _isCymbal(lanes[lane]);
    final shapeTop = cymbal ? r.center.dy - r.height / 6 : r.top;
    final shapeBottom = cymbal ? r.center.dy + r.height / 6 : r.bottom;
    const line = labelFontSize + 4;
    final below = shapeBottom + 4;
    final fitsBelow =
        !hasKick || below + line <= kickRect.top || below >= kickRect.bottom;
    if (fitsBelow) {
      // Never past the band: a name pushed off the surface names nothing.
      return math.min(below + row * line, size.height - 15);
    }
    return math.max(shapeTop - 3 - line * (row + 1), top + 1);
  }

  /// The size a name is drawn at, before the style's own clamp.
  static const double labelFontSize = 13;

  void _label(
    Canvas canvas,
    String text,
    Offset centre,
    double maxWidth,
    Color color,
    TextStyle? style,
  ) {
    if (text.isEmpty || maxWidth < 24) return;
    final tp = TextPainter(
      text: TextSpan(
        text: text,
        style: _nameStyle(style, color.withValues(alpha: color.a * 0.92)),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
      ellipsis: '…',
      textAlign: TextAlign.center,
    )..layout(maxWidth: maxWidth);
    tp.paint(canvas, Offset(centre.dx - tp.width / 2, centre.dy));
  }

  void _cymbal(
    Canvas canvas,
    Rect box,
    Color color,
    double lit, {
    bool awaited = false,
    double pulse = 0,
    double fade = 1,
  }) {
    final r = Rect.fromCenter(
      center: box.center,
      width: box.width,
      height: box.height / 3,
    );
    canvas.drawOval(
      r.inflate(box.width * 0.06),
      Paint()
        ..color = color.withValues(alpha: (0.18 + 0.5 * lit) * fade)
        ..maskFilter = MaskFilter.blur(BlurStyle.normal, box.width * 0.10),
    );
    canvas.drawOval(
      r,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = (awaited ? 2.6 + 1.8 * pulse : 2.0) + 2 * lit
        ..color = color.withValues(alpha: (awaited ? 1.0 : 0.9) * fade),
    );
    canvas.drawLine(
      r.center,
      Offset(r.center.dx, box.bottom + box.height * 0.4),
      Paint()
        ..color = color.withValues(alpha: 0.35 * fade)
        ..strokeWidth = 1.5,
    );
  }

  void _drum(
    Canvas canvas,
    Rect box,
    Color color,
    double lit, {
    bool awaited = false,
    double pulse = 0,
    double fade = 1,
  }) {
    final w = box.width;
    // Ellipse depth follows the width, but never past the box: in a short
    // band (the staff's strip of a kit) a head sized from the width alone
    // overflows the drum and spills off the surface.
    final headH = math.min(w * 0.30, box.height * 0.5);
    final head = Rect.fromCenter(
      center: Offset(box.center.dx, box.top + headH / 2),
      width: w,
      height: headH,
    );
    final base = Rect.fromCenter(
      center: Offset(box.center.dx, box.bottom - headH / 2),
      width: w,
      height: headH,
    );
    final shell = Path()
      ..moveTo(head.left, head.center.dy)
      ..lineTo(base.left, base.center.dy)
      ..arcTo(base, math.pi, -math.pi, false)
      ..lineTo(head.right, head.center.dy)
      ..arcTo(head, 0, -math.pi, false)
      ..close();
    canvas.drawPath(
      shell,
      Paint()
        ..color = Color.lerp(
          const Color(0xFF0A0817),
          color,
          (0.16 + 0.3 * lit) * fade,
        )!,
    );
    canvas.drawPath(
      shell,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = (awaited ? 2.6 + 1.8 * pulse : 2.0) + 2 * lit
        ..color = color.withValues(alpha: (awaited ? 1.0 : 0.85) * fade),
    );
    canvas.drawOval(
      head,
      Paint()
        ..color = Color.lerp(
          const Color(0xFF0A0817),
          color,
          (0.30 + 0.5 * lit) * fade,
        )!,
    );
    canvas.drawOval(
      head,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6 + 2 * lit
        ..color = Color.lerp(
          color,
          Colors.white,
          0.25 + 0.5 * lit,
        )!.withValues(alpha: fade),
    );
  }
}
