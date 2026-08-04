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
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../state/leaderboard.dart';
import '../state/my_standings_notifier.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import 'difficulty_badge.dart';
import 'leaderboard_view.dart';

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

/// The attribution line for a score card. For an upload: "{handle} · Cymbra".
/// For a catalog score: its origin (crawler source) plus the arranger when
/// known. `null` for bundled scores (no origin to show).
String? _attributionLine(CatalogEntry entry) {
  if (entry.uploaderHandle case final handle? when handle.isNotEmpty) {
    return '$handle · Cymbra';
  }
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
    this.statusTag,
  });

  final CatalogEntry entry;
  final VoidCallback onTap;

  /// Optional top-right overlay (e.g. a heart or delete button). `null` = none.
  final Widget? action;

  /// Optional bottom-left overlay tag (e.g. a proposal status pill). Kept out of the
  /// top row so a long label never collides with the difficulty badge or [action].
  final Widget? statusTag;

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
                    child: DifficultyBadge(level: entry.level, l10n: l10n),
                  ),
                  if (action != null)
                    Positioned(top: 4, right: 4, child: action!),
                  if (statusTag != null)
                    Positioned(bottom: 8, left: 10, child: statusTag!),
                  // Leaderboard badge — only for an accepted catalog score (the
                  // only kind with a shared board). Shows a bare trophy, plus the
                  // player's best rank across the two modes when they're ranked; a
                  // tap opens the same board widget as the pre-play view.
                  if (entry.catalogId != null)
                    Positioned(
                      bottom: 8,
                      right: 8,
                      child: _LeaderboardBadge(
                        catalogId: entry.catalogId!,
                        title: entry.title,
                      ),
                    ),
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

/// One sine layer of the generated cover wave (bundled so `_wave` stays within
/// the parameter budget).
typedef _WaveSpec = ({
  double baseY,
  double amp,
  double freq,
  double phase,
  double opacity,
  double stroke,
});

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
    final rng = math.Random(seed);
    _paintBackground(canvas, size, rng);
    final wave = _paintWaves(canvas, size, rng);
    _paintGlyphs(canvas, size, rng, wave.baseY, wave.amp);
    _paintWatermark(canvas, size);
  }

  /// Gradient fill + soft blurred blobs.
  void _paintBackground(Canvas canvas, Size size, math.Random rng) {
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
  }

  /// Sound-wave: frequency ← tempo, amplitude ← ambitus, plus a difficulty
  /// "energy" boost so harder pieces visibly read busier — taller, thicker, and
  /// with extra harmonics (idea B + the difficulty overlay). Falls back to the
  /// difficulty alone when facets are absent. Returns the base line + amplitude
  /// so the glyph band can sit above it.
  ({double baseY, double amp}) _paintWaves(
    Canvas canvas,
    Size size,
    math.Random rng,
  ) {
    final baseY = size.height * (0.55 + rng.nextDouble() * 0.18);
    final freq = tempoBpm != null
        ? (tempoBpm! / 40).clamp(1.5, 8.0)
        : 2.0 + intensity;
    final ampFrac = ambitus != null
        ? (0.04 + ambitus! / 520).clamp(0.04, 0.13)
        : 0.05 + 0.035 * intensity;
    // Difficulty overlay: ×1.0 / ×1.15 / ×1.3 for beginner / intermediate / advanced.
    final energy = 1.0 + 0.15 * intensity;
    final amp = size.height * ampFrac * energy;
    final phase = rng.nextDouble() * math.pi * 2;
    _wave(canvas, size, (
      baseY: baseY,
      amp: amp,
      freq: freq,
      phase: phase,
      opacity: 0.16 + 0.05 * intensity,
      stroke: 2.0 + 0.4 * intensity,
    ));
    // Second harmonic for advanced pieces (always) or dense ones.
    if (intensity >= 2 || (noteCount != null && noteCount! > 220)) {
      _wave(canvas, size, (
        baseY: baseY,
        amp: amp * 0.55,
        freq: freq * 2,
        phase: phase + 1.0,
        opacity: 0.11 + 0.03 * intensity,
        stroke: 1.3,
      ));
    }
    // A third, faint fast ripple for the busiest (advanced) pieces.
    if (intensity >= 2) {
      _wave(canvas, size, (
        baseY: baseY,
        amp: amp * 0.3,
        freq: freq * 3,
        phase: phase + 2.2,
        opacity: 0.09,
        stroke: 1.0,
      ));
    }
    return (baseY: baseY, amp: amp);
  }

  /// Fast-note glyphs, laid in evenly-spaced columns (small seeded jitter) so
  /// they never overlap; each sits in the band above the wave. With facets, only
  /// when the piece really has eighths (♫)/sixteenths (♬); else a difficulty proxy.
  void _paintGlyphs(
    Canvas canvas,
    Size size,
    math.Random rng,
    double baseY,
    double amp,
  ) {
    final (fastGlyph, noteGlyphs) = _glyphChoice();
    for (var i = 0; i < noteGlyphs; i++) {
      final t = noteGlyphs == 1 ? 0.5 : (i + 0.5) / noteGlyphs;
      final x =
          size.width * (0.14 + 0.72 * t) +
          (rng.nextDouble() - 0.5) * size.width * 0.05;
      final y = baseY - amp - size.height * (0.12 + rng.nextDouble() * 0.10);
      _glyph(
        canvas,
        Offset(x, y),
        14 + rng.nextDouble() * 5,
        fastGlyph,
        0.18 + rng.nextDouble() * 0.08,
      );
    }
  }

  /// The fast-note glyph + how many to draw, from the facets (or difficulty).
  (String, int) _glyphChoice() {
    if (minNoteValue == null) {
      return (intensity >= 1 ? '♫' : '♪', (1 + intensity).clamp(1, 4));
    }
    if (minNoteValue! < 8) return ('', 0); // only quarters/slower — no motif
    return (
      minNoteValue! >= 16 ? '♬' : '♫',
      noteCount != null
          ? (noteCount! / 160).clamp(1, 4).round()
          : 1 + intensity,
    );
  }

  /// Watermark: time signature + key accidentals bottom-right (idea 5).
  void _paintWatermark(Canvas canvas, Size size) {
    final marks = <String>[];
    if (timeSig.isNotEmpty && timeSig != '/') marks.add(timeSig);
    if (keyFifths != 0) {
      marks.add((keyFifths > 0 ? '♯' : '♭') * keyFifths.abs().clamp(1, 7));
    }
    if (marks.isEmpty) return;
    _text(
      canvas,
      marks.join('  '),
      Offset(size.width - 10, size.height - 8),
      11,
      0.28,
      TextAlign.right,
    );
  }

  void _wave(Canvas c, Size size, _WaveSpec w) {
    final path = Path()..moveTo(0, w.baseY);
    for (double x = 0; x <= size.width; x += 6) {
      final y =
          w.baseY +
          math.sin(x / size.width * math.pi * 2 * w.freq + w.phase) * w.amp;
      path.lineTo(x, y);
    }
    c.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = w.stroke
        ..color = Colors.white.withValues(alpha: w.opacity),
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

/// The compact leaderboard badge on a catalog score card (change: add-play-
/// leaderboards): a trophy, plus the player's best rank across the two modes when
/// they are ranked. Isolated as a `ConsumerWidget` so the card stays stateless; it
/// asks [MyStandings] for its piece (coalesced into one batch RPC per frame) and
/// reads only its own entry. A tap opens the shared board widget for the piece,
/// initialised on the mode that produced the best rank.
class _LeaderboardBadge extends ConsumerWidget {
  const _LeaderboardBadge({required this.catalogId, required this.title});

  final String catalogId;
  final String title;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Request this piece's standing (idempotent + batched); watch only our entry.
    ref.read(myStandingsProvider.notifier).request(catalogId);
    final standing = ref.watch(
      myStandingsProvider.select((m) => m[catalogId]),
    );
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => showLeaderboard(
        context,
        scoreId: catalogId,
        title: title,
        initialMode: standing?.mode ?? LeaderboardMode.tempo,
      ),
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: standing == null ? 6 : 8,
          vertical: 4,
        ),
        decoration: BoxDecoration(
          color: Colors.black.withValues(alpha: 0.5),
          borderRadius: BorderRadius.circular(10),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.emoji_events, size: 15, color: Color(0xFFF4C542)),
            if (standing != null) ...[
              const SizedBox(width: 3),
              Text(
                '#${standing.rank}',
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 12,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
