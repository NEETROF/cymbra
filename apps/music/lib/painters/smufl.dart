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

/// SMuFL (Standard Music Font Layout) glyph rendering with the Bravura font.
///
/// Glyphs are positioned in *staff spaces* (the distance between two staff
/// lines). Bravura is designed so 1 em = 4 staff spaces, so a glyph drawn at
/// `fontSize = 4 * staffSpace` renders at the right size, and the font's
/// alphabetic baseline coincides with the SMuFL origin (y = 0). Metric/anchor
/// constants below are copied from `bravura_metadata.json`.
class Smufl {
  Smufl._();

  static const String fontFamily = 'Bravura';

  // --- Glyph codepoints (SMuFL standard) ---------------------------------
  static const String noteheadBlack = '\u{E0A4}';
  static const String noteheadHalf = '\u{E0A3}';
  static const String noteheadWhole = '\u{E0A2}';

  static const String gClef = '\u{E050}';
  static const String fClef = '\u{E062}';
  static const String cClef = '\u{E05C}';

  static const String flag8thUp = '\u{E240}';
  static const String flag8thDown = '\u{E241}';
  static const String flag16thUp = '\u{E242}';
  static const String flag16thDown = '\u{E243}';
  static const String flag32ndUp = '\u{E244}';
  static const String flag32ndDown = '\u{E245}';

  static const String accidentalFlat = '\u{E260}';
  static const String accidentalNatural = '\u{E261}';
  static const String accidentalSharp = '\u{E262}';
  static const String accidentalDoubleSharp = '\u{E263}';
  static const String accidentalDoubleFlat = '\u{E264}';

  static const String restWhole = '\u{E4E3}';
  static const String restHalf = '\u{E4E4}';
  static const String restQuarter = '\u{E4E5}';
  static const String rest8th = '\u{E4E6}';
  static const String rest16th = '\u{E4E7}';

  static const String augmentationDot = '\u{E1E7}';
  static const String brace = '\u{E000}';

  // --- Engraving defaults (staff spaces), from bravura_metadata.json ------
  static const double staffLineThickness = 0.13;
  static const double stemThickness = 0.12;
  static const double beamThickness = 0.5;
  static const double legerLineThickness = 0.16;
  static const double legerLineExtension = 0.4;
  static const double thinBarlineThickness = 0.16;

  // noteheadBlack: bbox width and stem-attachment anchors (staff spaces).
  static const double noteheadWidth = 1.18;
  static const double stemUpAnchorX = 1.18;
  static const double stemUpAnchorY = 0.168;
  static const double stemDownAnchorX = 0.0;
  static const double stemDownAnchorY = -0.168;

  /// Dynamics letter glyphs (p/m/f/r/s/z/n) for composing markings like "pp".
  static const Map<String, String> _dynLetters = {
    'p': '\u{E520}',
    'm': '\u{E521}',
    'f': '\u{E522}',
    'r': '\u{E523}',
    's': '\u{E524}',
    'z': '\u{E525}',
    'n': '\u{E526}',
  };

  /// Accidental glyph for a MusicXML accidental token, or null if unsupported.
  static String? accidental(String token) => switch (token) {
    'flat' => accidentalFlat,
    'natural' => accidentalNatural,
    'sharp' => accidentalSharp,
    'double-sharp' => accidentalDoubleSharp,
    'sharp-sharp' => accidentalDoubleSharp,
    'flat-flat' => accidentalDoubleFlat,
    _ => null,
  };

  /// Clef glyph for a MusicXML clef sign (G/F/C), defaulting to treble.
  static String clef(String sign) => switch (sign) {
    'F' => fClef,
    'C' => cClef,
    _ => gClef,
  };

  /// Composes a dynamics glyph string (e.g. "pp", "mf") from letter glyphs.
  static String dynamics(String token) {
    final buf = StringBuffer();
    for (final ch in token.toLowerCase().split('')) {
      final g = _dynLetters[ch];
      if (g != null) buf.write(g);
    }
    return buf.toString();
  }

  /// Builds a laid-out [TextPainter] for [glyph] at the given [staffSpace].
  static TextPainter painterFor(String glyph, double staffSpace, Color color) {
    return TextPainter(
      text: TextSpan(
        text: glyph,
        style: TextStyle(
          fontFamily: fontFamily,
          fontSize: staffSpace * 4,
          color: color,
          height: 1.0,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
  }

  /// Paints [glyph] so its SMuFL origin lands at ([x], [baselineY]).
  ///
  /// When [centerX] is true, [x] is the horizontal centre of the glyph's
  /// advance box instead of its left edge.
  static void draw(
    Canvas canvas,
    String glyph,
    double x,
    double baselineY,
    double staffSpace,
    Color color, {
    bool centerX = false,
  }) {
    if (glyph.isEmpty) return;
    final tp = painterFor(glyph, staffSpace, color);
    final baseline = tp.computeDistanceToActualBaseline(
      TextBaseline.alphabetic,
    );
    final dx = centerX ? x - tp.width / 2 : x;
    tp.paint(canvas, Offset(dx, baselineY - baseline));
  }
}
