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

import 'package:flutter/foundation.dart' show mapEquals;
import 'package:flutter/material.dart';

import '../state/note_label.dart';
import '../theme/cymbra_theme.dart';
import 'piano_layout.dart';
import 'smufl.dart';

/// How a note label is laid out inside a key: the font size to draw at, and
/// whether it has to be turned on its side to fit.
class KeyLabelFit {
  final double fontSize;

  /// True when the label is rotated a quarter turn, trading the key's narrow
  /// width budget for its far larger height budget.
  final bool vertical;

  const KeyLabelFit(this.fontSize, {required this.vertical});

  @override
  bool operator ==(Object other) =>
      other is KeyLabelFit &&
      other.fontSize == fontSize &&
      other.vertical == vertical;

  @override
  int get hashCode => Object.hash(fontSize, vertical);

  @override
  String toString() => 'KeyLabelFit($fontSize, vertical: $vertical)';
}

/// Fits a note label **strictly inside** a key box of [keyWidth] by
/// [availableHeight], or returns null when it cannot be done legibly.
///
/// A label centred on a key it overflows would spill onto the neighbouring
/// keys and point at the wrong note, so overflowing is never an option. On a
/// narrow keyboard the way out is the key's other dimension: a white key is
/// some 15 px wide but ~96 px tall, so a label that cannot fit across fits
/// easily along.
///
/// [widthPerFontUnit] is the text's width at font size 1 — the painter measures
/// once and divides, which keeps this pure and exactly testable. Text height is
/// taken as the font size, which holds because labels are drawn with
/// `height: 1.0`.
KeyLabelFit? fitKeyLabel({
  required double widthPerFontUnit,
  required double keyWidth,
  required double availableHeight,
  double maxFontSize = 15,
  double minFontSize = 7,
}) {
  if (widthPerFontUnit <= 0 || keyWidth <= 0 || availableHeight <= 0) {
    return null;
  }
  // Keep a hair of padding so a glyph never touches the key's edge.
  final w = keyWidth - 2;
  final h = availableHeight - 2;
  if (w <= 0 || h <= 0) return null;

  // Upright, the text runs across the key: its width is the binding constraint
  // and its height must clear the band.
  final upright = math.min(w / widthPerFontUnit, math.min(h, maxFontSize));
  // On its side, the text runs along the key: the key's *width* now caps the
  // font size and its height carries the text's length.
  final sideways = math.min(w, math.min(h / widthPerFontUnit, maxFontSize));

  // Whichever reads bigger wins, upright on a tie. On a wide key both peg at
  // the cap and the label stays upright; on a narrow one turning it over is
  // worth several font sizes, which is the whole point of not overflowing.
  final vertical = sideways > upright;
  final fontSize = vertical ? sideways : upright;
  return fontSize >= minFontSize
      ? KeyLabelFit(fontSize, vertical: vertical)
      : null;
}

/// Visual state of a key, by precedence.
enum _KeyState {
  /// Not pressed and not expected.
  idle,

  /// Required by the right hand but not held — "press this key" (blue).
  expectedRight,

  /// Required by the left hand but not held — "press this key" (amber).
  expectedLeft,

  /// Required and held — correctly played.
  correct,

  /// Held but not required.
  pressed,

  /// Toggled on in a selection exercise (a course chord-building step).
  selected,
}

/// Draws the piano keyboard at the bottom of the screen with feedback: keys the
/// player must press now ([requiredNotes]) glow in their hand's colour (right =
/// blue, left = amber — [leftHandNotes] marks the left-hand subset), turn green
/// once correctly held, while any other pressed key ([activeNotes]) glows purple.
class PianoKeyboardPainter extends CustomPainter {
  final PianoLayout layout;
  final Set<int> activeNotes;

  /// Notes expected at the current playhead (the Wait Mode gate).
  final Set<int> requiredNotes;

  /// The subset of [requiredNotes] belonging to the left hand (staff 2+); the
  /// rest are right-hand. Drives the per-hand expected colour.
  final Set<int> leftHandNotes;

  /// The fixed-size window the chosen keyboard size represents (exactly N keys),
  /// or null in auto mode. When the drawn range ([layout]) is wider than this,
  /// a dashed boundary marks where the chosen size ends and the extra keys begin.
  final ({int low, int high})? chosenWindow;

  /// Reading-aid labels to draw on the awaited keys, by pitch. Empty when the
  /// aid is off or the gate is not holding. Each label is drawn strictly inside
  /// its own key (see [fitKeyLabel]) so it can never point at a neighbour.
  final Map<int, String> noteLabels;

  /// Naming convention for the octave anchors: letters (C4) or solfège (Do4).
  final bool solfege;
  final bool frenchRe;

  /// Font family for the key labels, so they follow the app's typography rather
  /// than resolving to whatever face the platform happens to default to. Null
  /// keeps the platform default.
  final String? labelFontFamily;

  /// Breathing phase (0..1) of the expected-key highlight while Wait Mode
  /// holds playback — the non-intrusive replacement for the old text banner.
  /// 0 (the default) renders the steady highlight.
  final double waitPulse;

  /// Keys toggled on in a selection exercise (change: add-notation-courses —
  /// the chord-building step). Persistent, unlike the momentary [activeNotes]
  /// flash; a held key still shows its press over the selection.
  final Set<int> selectedNotes;

  /// Whether to draw the C octave anchors (Do4/C4 + the middle-C dot). Lesson
  /// keyboards turn them off: under a highlighted key the middle-C puck reads
  /// as a second target, not as orientation.
  final bool showOctaveMarkers;

  const PianoKeyboardPainter({
    required this.layout,
    required this.activeNotes,
    this.requiredNotes = const {},
    this.leftHandNotes = const {},
    this.selectedNotes = const {},
    this.chosenWindow,
    this.noteLabels = const {},
    this.solfege = false,
    this.frenchRe = false,
    this.labelFontFamily,
    this.waitPulse = 0,
    this.showOctaveMarkers = true,
  });

  /// MIDI middle C (C4) — the anchor note the octave labels emphasise.
  static const int _middleC = 60;

  _KeyState _stateOf(int pitch) {
    final required = requiredNotes.contains(pitch);
    final active = activeNotes.contains(pitch);
    if (required && active) return _KeyState.correct;
    if (required) {
      return leftHandNotes.contains(pitch)
          ? _KeyState.expectedLeft
          : _KeyState.expectedRight;
    }
    if (active) return _KeyState.pressed;
    if (selectedNotes.contains(pitch)) return _KeyState.selected;
    return _KeyState.idle;
  }

  /// Brightens an expected key's colour by the current [waitPulse] phase, so
  /// awaited keys "breathe" while the Wait Mode gate is blocked.
  Color _pulsed(Color base) =>
      Color.lerp(base, const Color(0xFFFFFFFF), 0.35 * waitPulse)!;

  Color _fillFor(_KeyState state, {required bool isBlack}) => switch (state) {
    _KeyState.correct => CymbraColors.tertiary,
    _KeyState.expectedRight => _pulsed(CymbraColors.handRight),
    _KeyState.expectedLeft => _pulsed(CymbraColors.handLeft),
    _KeyState.pressed => CymbraColors.primaryContainer,
    // Light like the expected highlights, so key labels stay readable on it.
    _KeyState.selected => CymbraColors.secondary,
    _KeyState.idle =>
      isBlack ? CymbraColors.pianoBlack : CymbraColors.pianoWhite,
  };

  @override
  void paint(Canvas canvas, Size size) {
    final whiteH = size.height;
    final blackH = size.height * 0.62;

    // 1) White keys (background).
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (PianoLayout.isBlack(p)) continue;
      final r = layout.keyRect(p);
      final rect = Rect.fromLTWH(r.left, 0, r.width, whiteH);
      _drawKey(canvas, rect, _stateOf(p), isBlack: false);
    }

    // 2) Black keys (on top).
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (!PianoLayout.isBlack(p)) continue;
      final r = layout.keyRect(p);
      final rect = Rect.fromLTWH(r.left, 0, r.width, blackH);
      _drawKey(canvas, rect, _stateOf(p), isBlack: true);
    }

    // 3) Octave anchors: label each C (C3, C4…) at the bottom of its white key so
    // the player can orient their hands; middle C (C4) is emphasised.
    if (showOctaveMarkers) _drawOctaveLabels(canvas, whiteH);

    // 4) Reading-aid names, on the awaited keys themselves — no screen space of
    // their own, and anchored on the very key the finger is going to.
    _drawNoteLabels(canvas, whiteH, blackH);

    // 5) Chosen-size boundary: when the drawn range is wider than the chosen
    // keyboard size, a dashed line marks where the chosen size ends.
    _drawChosenSizeBoundary(canvas, whiteH);
  }

  /// Draws each awaited key's note name inside that key.
  ///
  /// White keys use the band below the black keys (the only part of a white key
  /// nothing is stacked on); black keys use their own lower half. A label that
  /// cannot be fitted legibly is dropped rather than drawn over its neighbours —
  /// the key still glows, so the player is never left without the cue.
  void _drawNoteLabels(Canvas canvas, double whiteH, double blackH) {
    if (noteLabels.isEmpty) return;
    for (final entry in noteLabels.entries) {
      final pitch = entry.key;
      if (!layout.contains(pitch)) continue;
      final isBlack = PianoLayout.isBlack(pitch);
      final r = layout.keyRect(pitch);

      // The vertical room this key offers, and where that room starts.
      final bandTop = isBlack ? blackH * 0.45 : blackH;
      final bandHeight = (isBlack ? blackH : whiteH) - bandTop;

      // Measure once at a reference size; width scales linearly with it.
      final probe = TextPainter(
        text: TextSpan(
          text: entry.value,
          style: TextStyle(
            fontFamily: labelFontFamily,
            fontSize: 100,
            height: 1.0,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      final fit = fitKeyLabel(
        widthPerFontUnit: probe.width / 100,
        keyWidth: r.width,
        availableHeight: bandHeight,
      );
      if (fit == null) continue;

      final tp = TextPainter(
        text: TextSpan(
          text: entry.value,
          style: TextStyle(
            fontFamily: labelFontFamily,
            // ♯/♭/♮ are not in every UI face; the bundled music font always has
            // them, so the alteration can never come out as a missing glyph.
            fontFamilyFallback: const [Smufl.fontFamily],
            fontSize: fit.fontSize,
            height: 1.0,
            fontWeight: FontWeight.w800,
            // Awaited keys are always filled with a light hand/correct colour,
            // so the label is dark whatever the state.
            color: CymbraColors.surfaceContainerLowest,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      canvas.save();
      if (fit.vertical) {
        // Quarter turn, reading bottom-to-top, centred in the key's band.
        canvas.translate(
          r.left + (r.width - tp.height) / 2,
          bandTop + (bandHeight + tp.width) / 2,
        );
        canvas.rotate(-math.pi / 2);
        tp.paint(canvas, Offset.zero);
      } else {
        tp.paint(
          canvas,
          Offset(
            r.left + (r.width - tp.width) / 2,
            bandTop + (bandHeight - tp.height) / 2,
          ),
        );
      }
      canvas.restore();
    }
  }

  /// Draws a "C{octave}" label under every C white key. Scientific pitch
  /// notation (MIDI 60 = C4). The label is skipped when it would not fit the key
  /// width, but middle C always keeps a small dot so it stays findable.
  void _drawOctaveLabels(Canvas canvas, double height) {
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (p % 12 != 0) continue; // C keys only
      // A key carrying a reading-aid name says it better; don't stack both.
      if (noteLabels.containsKey(p)) continue;
      final isMiddle = p == _middleC;
      final r = layout.keyRect(p);
      final fontSize = (r.width * 0.32).clamp(6.0, 11.0);
      final tp = TextPainter(
        text: TextSpan(
          text: octaveMarkerLabel(p, solfege: solfege, frenchRe: frenchRe),
          style: TextStyle(
            fontFamily: labelFontFamily,
            fontSize: fontSize,
            height: 1.0,
            fontWeight: isMiddle ? FontWeight.w800 : FontWeight.w500,
            color: isMiddle
                ? CymbraColors.primaryContainer
                : const Color(0xFF64748B),
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      // Middle C keeps an always-visible dot even if its label is too wide.
      if (isMiddle) {
        canvas.drawCircle(
          Offset(r.left + r.width / 2, height - fontSize - 8),
          math.max(2.0, r.width * 0.06),
          Paint()..color = CymbraColors.primaryContainer,
        );
      }

      if (tp.width > r.width - 2) continue; // too narrow: skip the text
      tp.paint(
        canvas,
        Offset(r.left + (r.width - tp.width) / 2, height - tp.height - 3),
      );
    }
  }

  /// Draws a dashed vertical line where the chosen keyboard size ends, on each
  /// side the drawn range was widened past it (see [chosenWindow]).
  void _drawChosenSizeBoundary(Canvas canvas, double height) {
    final window = chosenWindow;
    if (window == null) return;
    final paint = Paint()
      ..color = CymbraColors.outline
      ..strokeWidth = 1.5
      ..strokeCap = StrokeCap.round;
    // Extra keys below the chosen window → boundary at its low edge.
    if (layout.lowPitch < window.low) {
      _dashedVLine(canvas, layout.leftEdgeX(window.low), height, paint);
    }
    // Extra keys above the chosen window → boundary at its high edge.
    if (layout.highPitch > window.high) {
      _dashedVLine(canvas, layout.leftEdgeX(window.high + 1), height, paint);
    }
  }

  void _dashedVLine(Canvas canvas, double x, double height, Paint paint) {
    const dash = 5.0;
    const gap = 4.0;
    for (var y = 0.0; y < height; y += dash + gap) {
      canvas.drawLine(
        Offset(x, y),
        Offset(x, math.min(y + dash, height)),
        paint,
      );
    }
  }

  void _drawKey(
    Canvas canvas,
    Rect rect,
    _KeyState state, {
    required bool isBlack,
  }) {
    // Rounded bottom corners (4px) to mimic a physical key.
    final rrect = RRect.fromRectAndCorners(
      rect,
      bottomLeft: const Radius.circular(4),
      bottomRight: const Radius.circular(4),
    );

    final highlighted = state != _KeyState.idle;
    final fill = _fillFor(state, isBlack: isBlack);

    // Colored halo under any highlighted key.
    if (highlighted) {
      canvas.drawRRect(
        rrect,
        Paint()
          ..color = fill
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 12),
      );
    }

    canvas.drawRRect(rrect, Paint()..color = fill);

    // Border to separate white keys.
    if (!isBlack) {
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = const Color(0xFFE2E8F0),
      );
    } else if (highlighted) {
      // A highlighted black key is narrow; outline it so the state still reads.
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = CymbraColors.onSurface,
      );
    }
  }

  @override
  bool shouldRepaint(PianoKeyboardPainter old) =>
      old.activeNotes != activeNotes ||
      old.requiredNotes != requiredNotes ||
      old.leftHandNotes != leftHandNotes ||
      old.selectedNotes != selectedNotes ||
      old.chosenWindow != chosenWindow ||
      !mapEquals(old.noteLabels, noteLabels) ||
      old.solfege != solfege ||
      old.frenchRe != frenchRe ||
      old.labelFontFamily != labelFontFamily ||
      old.waitPulse != waitPulse ||
      old.showOctaveMarkers != showOctaveMarkers ||
      old.layout.width != layout.width ||
      old.layout.lowPitch != layout.lowPitch ||
      old.layout.highPitch != layout.highPitch;
}
