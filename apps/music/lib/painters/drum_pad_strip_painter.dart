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

import 'package:flutter/foundation.dart' show setEquals;
import 'package:flutter/material.dart';

import '../state/drum_kit.dart';
import '../theme/cymbra_theme.dart';

/// The pad strip (change: add-drum-kit-view): the percussion controller
/// replacing the on-screen keyboard — one pad per lane **in the lane order**
/// (the same [DrumLane] list the cascade renders, so a player who learns
/// where a piece falls learns it once), with the kick as a single wide pedal
/// beneath the pads, never one pad among them.
///
/// Playable since `add-drum-input-mapping`: a pointer-down anywhere in a
/// pad's horizontal span strikes it ([surfaceAt]), and the struck surface
/// shows a brief **struck** flash, decaying over [flashDurationMs] on its own
/// clock.
///
/// Since `add-drum-scoring` the strip also carries the gate's own indicator:
/// the [expectedSurfaces] the Wait Mode gate is waiting for are outlined, and
/// [waitPulse] breathes that outline while the gate is blocked — the pad
/// counterpart of the keyboard's pulsing expected keys, and the reason no
/// overlay or banner is ever drawn over the play surface. The pulse stops the
/// moment the gate releases (the controller is stopped at 0), leaving the
/// steady expected outline.
///
/// The strip's height follows the keyboard's viewport policy and is
/// independent of the piece count — pads keep a usable touch target however
/// few there are, and a sparse kit must not steal height from the cascade.
class DrumPadStripPainter extends CustomPainter {
  /// The lanes in presentation order ([PlayerData.presentedDrumLanes]).
  final List<DrumLane> lanes;

  /// Localised label per lane, aligned with [lanes] (resolved by the widget —
  /// a painter never reads localisation itself).
  final List<String> labels;

  /// Localised label of the kick pedal.
  final String kickLabel;

  /// Whether the score has any kick note (the pedal only shows when the foot
  /// actually plays).
  final bool hasKick;

  /// When each surface was last struck (the player's `struckSurfacesMs`, in
  /// wall-clock ms): pads by their index in [lanes], the pedal under
  /// [kPedalSurface].
  final Map<int, double> struckMs;

  /// The wall-clock instant this frame paints — the flash's age is measured
  /// against it, so the decay runs on real time even while playback is
  /// stopped (percussion strokes exist outside the playhead).
  final double nowMs;

  /// The surfaces the Wait Mode gate is waiting for (pads by their index in
  /// [lanes], the pedal under [kPedalSurface]) — `PlayerData.expectedDrumSurfaces`,
  /// so the strip names exactly the onset the gate and the scorer name.
  final Set<int> expectedSurfaces;

  /// Breathing phase 0→1→0 of the expected outline while the gate is blocked;
  /// 0 (steady) as soon as it releases.
  final double waitPulse;

  final String? labelFontFamily;

  const DrumPadStripPainter({
    required this.lanes,
    required this.labels,
    required this.kickLabel,
    required this.hasKick,
    this.struckMs = const {},
    this.nowMs = 0,
    this.expectedSurfaces = const {},
    this.waitPulse = 0,
    this.labelFontFamily,
  });

  /// The pedal band's share of the strip height when present.
  static const double pedalFraction = 0.30;

  /// How long a struck flash lasts, in milliseconds — one constant, the feel
  /// pass's value (change: add-drum-input-mapping, task 7.3). Short enough
  /// that a two-finger roll stays a sequence of distinct flashes rather than
  /// smearing into a solid glow, long enough to be seen under a fast groove.
  static const int flashDurationMs = 180;

  /// [flashDurationMs] as a [Duration], for the repaint clock the strip runs
  /// on while playback is stopped.
  static const Duration flashDuration = Duration(milliseconds: flashDurationMs);

  /// The flash intensity of a surface struck at [struckMs] as seen at
  /// [nowMs]: 1 at the attack, decaying linearly to 0 over
  /// [flashDurationMs]. Pure, so the decay is testable without a clock.
  ///
  /// Time-based, never hold-based: a percussion release arrives within
  /// milliseconds of its attack, so a highlight that lasted the hold would be
  /// an invisible flicker. Nothing here depends on the note-off, on the
  /// playhead, or on whether the stroke landed on an onset — one state,
  /// claiming nothing.
  static double flashIntensity({
    required double struckMs,
    required double nowMs,
  }) {
    final age = nowMs - struckMs;
    if (age < 0 || age >= flashDurationMs) return 0;
    return 1 - age / flashDurationMs;
  }

  /// The controller surface under a pointer at [local] on a strip of [size]:
  /// the index of the pad whose horizontal span contains it, [kPedalSurface]
  /// in the pedal band, or null where the strip presents nothing.
  ///
  /// **The whole strip is live** (change: add-drum-input-mapping): the insets
  /// between the drawn pads are styling, not hit boundaries, and the pedal
  /// band is kick from edge to edge. A drummer at tempo aims at a region, not
  /// a rounded rectangle; a tap swallowed by a decorative gutter is a ghost
  /// stroke, and a ghost stroke reads as broken input.
  static int? surfaceAt(
    Offset local,
    Size size, {
    required int laneCount,
    required bool hasKick,
  }) {
    if (size.width <= 0 || size.height <= 0) return null;
    if (local.dx < 0 ||
        local.dy < 0 ||
        local.dx >= size.width ||
        local.dy >= size.height) {
      return null;
    }
    final pedalHeight = hasKick ? size.height * pedalFraction : 0.0;
    final padsHeight = size.height - pedalHeight;
    if (hasKick && local.dy >= padsHeight) return kPedalSurface;
    if (laneCount <= 0) return null;
    final index = (local.dx / (size.width / laneCount)).floor();
    return index.clamp(0, laneCount - 1);
  }

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = CymbraColors.surfaceContainerLow,
    );

    final pedalHeight = hasKick ? size.height * pedalFraction : 0.0;
    final padsHeight = size.height - pedalHeight;

    if (lanes.isNotEmpty) {
      final laneWidth = size.width / lanes.length;
      for (var i = 0; i < lanes.length; i++) {
        final rect = Rect.fromLTWH(
          laneWidth * i + 3,
          4,
          laneWidth - 6,
          padsHeight - 8,
        );
        final rrect = RRect.fromRectAndRadius(rect, const Radius.circular(8));
        final flash = _intensityOf(i);
        canvas.drawRRect(
          rrect,
          Paint()
            ..color = Color.lerp(
              CymbraColors.surfaceContainerHigh,
              CymbraColors.primary,
              flash * 0.75,
            )!,
        );
        canvas.drawRRect(
          rrect,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1 + 1.5 * flash
            ..color = Color.lerp(
              CymbraColors.outlineVariant.withValues(alpha: 0.7),
              CymbraColors.primary,
              flash,
            )!,
        );
        _expected(canvas, rrect, i);
        _label(
          canvas,
          labels.length > i ? labels[i] : '',
          rect,
          CymbraColors.onSurface,
        );
      }
    }

    if (hasKick) {
      // The kick pedal: one wide band beneath the pads — the controller's
      // mirror of the cascade's full-width bar, in the foot colour. It flashes
      // exactly like a pad (same state, same decay): a foot stroke is a stroke.
      final rect = Rect.fromLTWH(
        3,
        padsHeight + 2,
        size.width - 6,
        pedalHeight - 6,
      );
      final rrect = RRect.fromRectAndRadius(rect, const Radius.circular(8));
      final flash = _intensityOf(kPedalSurface);
      canvas.drawRRect(
        rrect,
        Paint()
          ..color = CymbraColors.handLeft.withValues(
            alpha: 0.28 + 0.52 * flash,
          ),
      );
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1 + 1.5 * flash
          ..color = CymbraColors.handLeft.withValues(alpha: 0.7 + 0.3 * flash),
      );
      _expected(canvas, rrect, kPedalSurface);
      _label(canvas, kickLabel, rect, CymbraColors.onSurface);
    }
  }

  /// The "play this now" outline of an expected surface, breathing with
  /// [waitPulse] while the gate holds. Drawn inside the pad's own rounded rect
  /// so it never bleeds into a neighbour, and skipped entirely for a surface
  /// the gate is not waiting for — the strip claims nothing about pieces the
  /// onset does not ask for.
  void _expected(Canvas canvas, RRect rrect, int surface) {
    if (!expectedSurfaces.contains(surface)) return;
    final pulse = waitPulse.clamp(0.0, 1.0);
    canvas.drawRRect(
      rrect.deflate(2),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2 + 1.5 * pulse
        ..color = CymbraColors.secondary.withValues(alpha: 0.55 + 0.45 * pulse),
    );
  }

  /// The struck-flash intensity of [surface] on this frame (0 when it was
  /// never struck, or its flash has decayed).
  double _intensityOf(int surface) {
    final struck = struckMs[surface];
    if (struck == null) return 0;
    return flashIntensity(struckMs: struck, nowMs: nowMs);
  }

  /// Whether any surface is mid-flash — the strip must keep repainting while
  /// one is, and may stop once none is.
  bool get _flashing =>
      struckMs.values.any((t) => nowMs - t >= 0 && nowMs - t < flashDurationMs);

  void _label(Canvas canvas, String text, Rect within, Color color) {
    if (text.isEmpty) return;
    final tp = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: color.withValues(alpha: 0.85),
          fontSize: 12,
          fontFamily: labelFontFamily,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 2,
      textAlign: TextAlign.center,
      ellipsis: '…',
    )..layout(maxWidth: within.width - 8);
    tp.paint(
      canvas,
      Offset(
        within.left + (within.width - tp.width) / 2,
        within.top + (within.height - tp.height) / 2,
      ),
    );
  }

  @override
  bool shouldRepaint(DrumPadStripPainter old) =>
      old.lanes != lanes ||
      old.labels != labels ||
      old.hasKick != hasKick ||
      old.kickLabel != kickLabel ||
      old.waitPulse != waitPulse ||
      !setEquals(old.expectedSurfaces, expectedSurfaces) ||
      // A flash in flight on either frame: the decay is time-based, so a new
      // `nowMs` alone changes the picture — but only while something is
      // actually flashing, so a still strip does not repaint on every frame.
      old._flashing ||
      _flashing;
}
