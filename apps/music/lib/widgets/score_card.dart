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

import '../l10n/gen/app_localizations.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';

/// The difficulty accent colour (green / teal / pink) shared by the badge + cover.
Color scoreLevelColor(PracticeLevel level) => switch (level) {
  PracticeLevel.beginner => CymbraColors.tertiary,
  PracticeLevel.intermediate => CymbraColors.secondary,
  PracticeLevel.advanced => CymbraColors.error,
};

/// A friendly name for a crawler source code, or `null` to omit (dev/unknown).
String? _sourceLabel(String? source) => switch (source) {
  'pdmx' => 'PDMX',
  'openscore' => 'OpenScore',
  'musetrainer' => 'MuseTrainer',
  'mutopia' => 'Mutopia',
  'eduardomourar' => 'GitHub · eduardomourar',
  null || '' || 'seed' => null,
  final s => s,
};

/// The attribution line for a catalog score: its origin (crawler source) plus the
/// arranger when known. `null` for bundled/contributed scores (no origin to show).
String? _attributionLine(CatalogEntry entry) {
  final parts = <String>[];
  final origin = _sourceLabel(entry.source);
  if (origin != null) parts.add('via $origin');
  final arranger = entry.arranger;
  if (arranger != null && arranger.isNotEmpty) parts.add('arr. $arranger');
  return parts.isEmpty ? null : parts.join(' · ');
}

/// A score tile used across the Score Hub and the library: a generated cover, a
/// difficulty badge, the title/composer, and an optional top-right [action]
/// (save / remove / delete — the caller decides, since each surface differs).
class ScoreCard extends StatelessWidget {
  const ScoreCard({
    super.key,
    required this.entry,
    required this.onTap,
    this.action,
  });

  final CatalogEntry entry;
  final VoidCallback onTap;

  /// Optional top-right overlay (e.g. a heart or delete button). `null` = none.
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Material(
      color: CymbraColors.surfaceContainerLow,
      borderRadius: BorderRadius.circular(18),
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            AspectRatio(
              aspectRatio: 16 / 11,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  _CoverArt(entry: entry),
                  Positioned(
                    top: 10,
                    left: 10,
                    child: _DifficultyBadge(level: entry.level, l10n: l10n),
                  ),
                  if (action != null)
                    Positioned(top: 4, right: 4, child: action!),
                ],
              ),
            ),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      entry.title,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: CymbraColors.onSurface,
                        fontSize: 16,
                        fontWeight: FontWeight.w700,
                        height: 1.15,
                      ),
                    ),
                    if (entry.composer.isNotEmpty) ...[
                      const SizedBox(height: 4),
                      Text(
                        entry.composer,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: CymbraColors.onSurfaceVariant,
                          fontSize: 13,
                        ),
                      ),
                    ],
                    if (_attributionLine(entry) case final attribution?) ...[
                      const SizedBox(height: 4),
                      Row(
                        children: [
                          const Icon(
                            Icons.cloud_outlined,
                            size: 12,
                            color: CymbraColors.outline,
                          ),
                          const SizedBox(width: 4),
                          Expanded(
                            child: Text(
                              attribution,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: const TextStyle(
                                color: CymbraColors.outline,
                                fontSize: 11,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DifficultyBadge extends StatelessWidget {
  const _DifficultyBadge({required this.level, required this.l10n});

  final PracticeLevel level;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final color = scoreLevelColor(level);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.22),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.6)),
      ),
      child: Text(
        level.localizedLabel(l10n).toUpperCase(),
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w800,
          letterSpacing: 0.5,
        ),
      ),
    );
  }
}

/// A deterministic generated cover for a score (no artwork exists). It is seeded
/// from the title+composer (stable across re-crawls) and, when the catalog has
/// backfilled facets, *reflects the piece*: the central glyph is its fastest note
/// value (♩/♪/♬), the sound-wave's frequency follows the tempo and its amplitude
/// the ambitus, and eighth/sixteenth glyphs appear only when the piece really has
/// them. The gradient is tinted toward the difficulty colour, and the time
/// signature / key accidentals sit as a watermark. Without facets it falls back
/// to the difficulty + seed.
class _CoverArt extends StatelessWidget {
  const _CoverArt({required this.entry});

  final CatalogEntry entry;

  static const List<List<Color>> _gradients = [
    [Color(0xFF3B1E6E), Color(0xFF7C3AED)], // purple
    [Color(0xFF0B3D4A), Color(0xFF03C6B2)], // teal
    [Color(0xFF10233F), Color(0xFF5B9DFF)], // blue
    [Color(0xFF4A1D3F), Color(0xFFE05299)], // magenta
    [Color(0xFF123A2C), Color(0xFF4EDEA3)], // green
    [Color(0xFF3A2A12), Color(0xFFFFB454)], // amber
    [Color(0xFF241645), Color(0xFF44E2CD)], // indigo->teal
    [Color(0xFF3A1220), Color(0xFFFF6B6B)], // rose
  ];

  @override
  Widget build(BuildContext context) {
    // Seed on title+composer so the cover is stable even if a re-crawl changes
    // the row id (idea 1).
    final seed = '${entry.title}|${entry.composer}'.hashCode;
    final base = _gradients[seed.abs() % _gradients.length];
    // Tint the accent stop toward the difficulty colour (idea 3).
    final accent = Color.lerp(base[1], scoreLevelColor(entry.level), 0.35)!;
    final intensity = switch (entry.level) {
      PracticeLevel.beginner => 0,
      PracticeLevel.intermediate => 1,
      PracticeLevel.advanced => 2,
    };
    final ambitus = (entry.lowestMidi != null && entry.highestMidi != null)
        ? entry.highestMidi! - entry.lowestMidi!
        : null;
    return CustomPaint(
      painter: _CoverPainter(
        seed: seed,
        colors: [base[0], accent],
        intensity: intensity,
        minNoteValue: entry.minNoteValue,
        tempoBpm: entry.tempoBpm,
        noteCount: entry.noteCount,
        ambitus: ambitus,
        timeSig: entry.timeSig ?? '',
        keyFifths: entry.keyFifths ?? 0,
      ),
      // Central glyph = the fastest note value (idea 2); ♪ when unknown.
      child: Center(
        child: Text(
          _valueGlyph(entry.minNoteValue),
          style: TextStyle(
            fontSize: 60,
            color: Colors.white.withValues(alpha: 0.18),
            height: 1,
          ),
        ),
      ),
    );
  }
}

/// The note-value glyph for the fastest note (power-of-two denominator): ♩ for a
/// quarter or slower, ♪ for eighths, ♬ for sixteenths and faster; ♪ when unknown.
String _valueGlyph(int? minNoteValue) => switch (minNoteValue) {
  null => '♪',
  final v when v <= 4 => '♩',
  8 => '♪',
  _ => '♬',
};

class _CoverPainter extends CustomPainter {
  _CoverPainter({
    required this.seed,
    required this.colors,
    required this.intensity,
    required this.minNoteValue,
    required this.tempoBpm,
    required this.noteCount,
    required this.ambitus,
    required this.timeSig,
    required this.keyFifths,
  });

  final int seed;
  final List<Color> colors;
  final int intensity;
  final int? minNoteValue;
  final int? tempoBpm;
  final int? noteCount;
  final int? ambitus;
  final String timeSig;
  final int keyFifths;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    canvas.drawRect(
      rect,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: colors,
        ).createShader(rect),
    );
    final rng = math.Random(seed);
    for (var i = 0; i < 4; i++) {
      final c = Offset(
        rng.nextDouble() * size.width,
        rng.nextDouble() * size.height,
      );
      final r = size.shortestSide * (0.18 + rng.nextDouble() * 0.22);
      canvas.drawCircle(
        c,
        r,
        Paint()
          ..color = Colors.white.withValues(
            alpha: 0.05 + rng.nextDouble() * 0.06,
          )
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 18),
      );
    }

    // Sound-wave: frequency ← tempo, amplitude ← ambitus, a second harmonic for
    // dense pieces (idea B). Falls back to the difficulty when facets are absent.
    final baseY = size.height * (0.55 + rng.nextDouble() * 0.18);
    final freq = tempoBpm != null
        ? (tempoBpm! / 40).clamp(1.5, 8.0)
        : 2.0 + intensity;
    final ampFrac = ambitus != null
        ? (0.04 + ambitus! / 520).clamp(0.04, 0.13)
        : 0.05 + 0.035 * intensity;
    final amp = size.height * ampFrac;
    final phase = rng.nextDouble() * math.pi * 2;
    _wave(canvas, size, baseY, amp, freq, phase, 0.16 + 0.04 * intensity, 2.0);
    final busy = noteCount != null ? noteCount! > 260 : intensity >= 2;
    if (busy) {
      _wave(canvas, size, baseY, amp * 0.5, freq * 2, phase + 1.0, 0.10, 1.2);
    }

    // Fast-note glyphs: with facets, only when the piece really has eighths
    // (♫) or sixteenths (♬); without facets, a difficulty proxy.
    final int noteGlyphs;
    final String fastGlyph;
    if (minNoteValue != null) {
      if (minNoteValue! >= 8) {
        fastGlyph = minNoteValue! >= 16 ? '♬' : '♫';
        noteGlyphs = noteCount != null
            ? (noteCount! / 140).clamp(1, 5).round()
            : 1 + intensity;
      } else {
        fastGlyph = '';
        noteGlyphs = 0; // only quarters/slower — no fast-note motif
      }
    } else {
      fastGlyph = intensity >= 1 ? '♫' : '♪';
      noteGlyphs = 1 + intensity * 2;
    }
    for (var i = 0; i < noteGlyphs; i++) {
      final x = size.width * (0.12 + rng.nextDouble() * 0.76);
      final y = baseY - amp - size.height * (0.06 + rng.nextDouble() * 0.12);
      _glyph(
        canvas,
        Offset(x, y),
        13 + rng.nextDouble() * 7,
        fastGlyph,
        0.16 + rng.nextDouble() * 0.10,
      );
    }

    // Watermark: time signature + key accidentals bottom-right (idea 5).
    final marks = <String>[];
    if (timeSig.isNotEmpty && timeSig != '/') marks.add(timeSig);
    if (keyFifths != 0) {
      marks.add((keyFifths > 0 ? '♯' : '♭') * keyFifths.abs().clamp(1, 7));
    }
    if (marks.isNotEmpty) {
      _text(
        canvas,
        marks.join('  '),
        Offset(size.width - 10, size.height - 8),
        11,
        0.28,
        TextAlign.right,
      );
    }
  }

  void _wave(
    Canvas c,
    Size size,
    double baseY,
    double amp,
    double freq,
    double phase,
    double opacity,
    double stroke,
  ) {
    final path = Path()..moveTo(0, baseY);
    for (double x = 0; x <= size.width; x += 6) {
      final y =
          baseY + math.sin(x / size.width * math.pi * 2 * freq + phase) * amp;
      path.lineTo(x, y);
    }
    c.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = stroke
        ..color = Colors.white.withValues(alpha: opacity),
    );
  }

  void _glyph(
    Canvas c,
    Offset pos,
    double fontSize,
    String glyph,
    double opacity,
  ) {
    if (glyph.isEmpty) return;
    final tp = TextPainter(
      text: TextSpan(
        text: glyph,
        style: TextStyle(
          fontSize: fontSize,
          color: Colors.white.withValues(alpha: opacity),
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(c, pos - Offset(tp.width / 2, tp.height / 2));
  }

  /// Bottom/right-anchored watermark text.
  void _text(
    Canvas c,
    String s,
    Offset anchor,
    double fontSize,
    double opacity,
    TextAlign align,
  ) {
    final tp = TextPainter(
      text: TextSpan(
        text: s,
        style: TextStyle(
          fontSize: fontSize,
          fontWeight: FontWeight.w700,
          color: Colors.white.withValues(alpha: opacity),
        ),
      ),
      textAlign: align,
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(c, anchor - Offset(tp.width, tp.height));
  }

  @override
  bool shouldRepaint(_CoverPainter old) =>
      old.seed != seed ||
      old.colors != colors ||
      old.intensity != intensity ||
      old.minNoteValue != minNoteValue ||
      old.tempoBpm != tempoBpm ||
      old.noteCount != noteCount ||
      old.ambitus != ambitus ||
      old.timeSig != timeSig ||
      old.keyFifths != keyFifths;
}
