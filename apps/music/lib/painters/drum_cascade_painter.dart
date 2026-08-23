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
  });

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

    final hitLineY = size.height;
    final pxPerMs = size.height / lookAheadMs;

    // Hit line (subtle) at the bottom, matching the keyboard cascade.
    canvas.drawLine(
      Offset(0, hitLineY - 1),
      Offset(size.width, hitLineY - 1),
      Paint()
        ..color = CymbraColors.outlineVariant.withValues(alpha: 0.6)
        ..strokeWidth = 1,
    );

    if (lanes.isNotEmpty) {
      // Faint lane separators, so the aim columns read without stealing
      // contrast from the notes.
      final laneWidth = size.width / lanes.length;
      final sep = Paint()
        ..color = CymbraColors.outlineVariant.withValues(alpha: 0.25)
        ..strokeWidth = 1;
      for (var i = 1; i < lanes.length; i++) {
        final x = laneWidth * i;
        canvas.drawLine(Offset(x, 0), Offset(x, hitLineY), sep);
      }
    }

    double bottomYOf(TimedNote n) =>
        hitLineY - (n.startMs - elapsedMs) * pxPerMs;

    // --- Pass 1: the foot bars, BENEATH the note layer -------------------
    // The kick (GM 35/36) always; any other foot event with no lane would be
    // a silent drop, so it can only be the pedal hi-hat 44 — which HAS a lane
    // (terminal bucket) and is drawn there as a note by pass 2 instead.
    const footColor = CymbraColors.handLeft; // the feet take the amber
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

    // --- Pass 2: the hand notes, per lane, on top ------------------------
    if (lanes.isEmpty) return;
    final laneWidth = size.width / lanes.length;
    for (final n in notes) {
      if (kKickGmNumbers.contains(n.pitch)) continue;
      final lane = laneIndexOf(lanes, n.pitch);
      if (lane == null) continue; // absent from the layout (defensive)

      final bottomY = bottomYOf(n);
      final height = (n.durationMs * pxPerMs).clamp(10.0, double.infinity);
      final topY = bottomY - height;
      if (bottomY < 0 || topY > hitLineY) continue;

      final left = laneWidth * lane;
      final inset = laneWidth * 0.14;
      final rect = Rect.fromLTRB(
        left + inset,
        topY,
        left + laneWidth - inset,
        bottomY,
      );

      final foot = isFootNote(n, multiVoice: multiVoice);
      final handColor = foot ? CymbraColors.handLeft : CymbraColors.handRight;
      final inHitZone =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      final base = inHitZone
          ? Color.lerp(handColor, const Color(0xFFFFFFFF), 0.4)!
          : handColor;

      final rrect = RRect.fromRectAndRadius(rect, const Radius.circular(4));
      if (isOpenHiHat(n.pitch)) {
        // The open hi-hat is a VARIANT of the note inside the hi-hat lane —
        // hollow where the closed stroke is filled — never a bar and never a
        // lane of its own: open-versus-closed is a different number on the
        // hand stroke; no foot note exists in the file.
        canvas.drawRRect(
          rrect.deflate(1.25),
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2.5
            ..color = base,
        );
      } else {
        // A hairline of background keeps the edge crisp over a foot bar.
        canvas.drawRRect(
          rrect.inflate(0.75),
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.5
            ..color = CymbraColors.background,
        );
        canvas.drawRRect(rrect, Paint()..color = base);
      }
    }
  }

  @override
  bool shouldRepaint(DrumCascadePainter old) =>
      old.elapsedMs != elapsedMs ||
      old.notes != notes ||
      old.lanes != lanes ||
      old.multiVoice != multiVoice;
}
