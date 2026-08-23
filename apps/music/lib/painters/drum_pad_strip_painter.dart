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

import '../state/drum_kit.dart';
import '../theme/cymbra_theme.dart';

/// The pad strip (change: add-drum-kit-view): the percussion controller
/// replacing the on-screen keyboard — one pad per lane **in the lane order**
/// (the same [DrumLane] list the cascade renders, so a player who learns
/// where a piece falls learns it once), with the kick as a single wide pedal
/// beneath the pads, never one pad among them.
///
/// Display-only in this change: pads produce no note and no feedback until
/// `add-drum-input-mapping`. The strip's height follows the keyboard's
/// viewport policy and is independent of the piece count — pads keep a usable
/// touch target however few there are, and a sparse kit must not steal height
/// from the cascade.
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

  final String? labelFontFamily;

  const DrumPadStripPainter({
    required this.lanes,
    required this.labels,
    required this.kickLabel,
    required this.hasKick,
    this.labelFontFamily,
  });

  /// The pedal band's share of the strip height when present.
  static const double pedalFraction = 0.30;

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
        canvas.drawRRect(
          rrect,
          Paint()..color = CymbraColors.surfaceContainerHigh,
        );
        canvas.drawRRect(
          rrect,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1
            ..color = CymbraColors.outlineVariant.withValues(alpha: 0.7),
        );
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
      // mirror of the cascade's full-width bar, in the foot colour.
      final rect = Rect.fromLTWH(
        3,
        padsHeight + 2,
        size.width - 6,
        pedalHeight - 6,
      );
      final rrect = RRect.fromRectAndRadius(rect, const Radius.circular(8));
      canvas.drawRRect(
        rrect,
        Paint()..color = CymbraColors.handLeft.withValues(alpha: 0.28),
      );
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = CymbraColors.handLeft.withValues(alpha: 0.7),
      );
      _label(canvas, kickLabel, rect, CymbraColors.onSurface);
    }
  }

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
      old.kickLabel != kickLabel;
}
