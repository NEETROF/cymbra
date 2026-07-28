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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../painters/staff_painter.dart';
import '../services/audio_service.dart';
import '../state/card_preview_notifier.dart';
import '../state/player_data.dart' show scoreNoteEdges;
import '../theme/cymbra_theme.dart';

/// The in-card **read-only** score preview (change: add-app-score-rating): the
/// same horizontal game-score render as play — the notation scrolls and the notes
/// sound — but with NO interaction, NO judging and NO scoring. It reuses the
/// player's [StaffPainter] and audio seam directly, driven by a local ticker, so
/// nothing about the scored player state is touched. Loops until dismissed.
class InCardPreview extends ConsumerStatefulWidget {
  const InCardPreview({
    super.key,
    required this.catalogId,
    required this.onClose,
  });

  /// The catalog score to preview (its bytes are fetched + parsed on demand).
  final String catalogId;

  /// Called when the user stops the preview (returns to the card face).
  final VoidCallback onClose;

  @override
  ConsumerState<InCardPreview> createState() => _InCardPreviewState();
}

class _InCardPreviewState extends ConsumerState<InCardPreview>
    with SingleTickerProviderStateMixin {
  late final Ticker _ticker;

  /// Captured in [initState] so [dispose] never reads a provider on a disposing
  /// container (mirrors the player notifier's captured audio reference).
  late final AudioService _audio;
  Duration _lastTick = Duration.zero;
  double _elapsedMs = 0;
  bool _seeded = false;

  /// Pitches currently sounding (audio-only), so each note is released when the
  /// playhead passes its end — mirrors the player's `_sounding` set.
  final Set<int> _sounding = <int>{};

  @override
  void initState() {
    super.initState();
    _audio = ref.read(audioServiceProvider);
    // Start the synth (idempotent, silent no-op on failure) so the preview sounds.
    unawaited(_audio.init());
    _ticker = createTicker(_onTick)..start();
  }

  @override
  void dispose() {
    _ticker.dispose();
    // Flush any ringing voices when leaving the preview (captured reference).
    _audio.allNotesOff();
    super.dispose();
  }

  void _onTick(Duration elapsed) {
    final score = ref.read(cardPreviewScoreProvider(widget.catalogId)).value;
    if (score == null || score.isEmpty) {
      _lastTick = elapsed;
      return;
    }
    if (!_seeded) {
      _elapsedMs = score.startMs;
      _seeded = true;
      _lastTick = elapsed;
      return;
    }
    final dtMs = (elapsed - _lastTick).inMicroseconds / 1000.0;
    _lastTick = elapsed;
    if (dtMs <= 0 || dtMs > 100) return; // skip a stalled/huge frame

    var next = _elapsedMs + dtMs;
    if (score.songEndMs > 0 && next >= score.songEndMs) {
      // Loop: silence across the seam and wrap to the trimmed start.
      _audio.allNotesOff();
      _sounding.clear();
      next = score.startMs;
    } else {
      // Sound onsets crossed, release notes whose end the playhead passed.
      final edges = scoreNoteEdges(
        visible: score.notes,
        from: _elapsedMs,
        to: next,
        sounding: _sounding,
      );
      for (final p in edges.stops) {
        _audio.noteOff(p);
        _sounding.remove(p);
      }
      for (final p in edges.starts) {
        _audio.noteOn(p);
        _sounding.add(p);
      }
    }
    setState(() => _elapsedMs = next);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(cardPreviewScoreProvider(widget.catalogId));
    return ColoredBox(
      color: CymbraColors.surfaceContainerLow,
      child: Stack(
        fit: StackFit.expand,
        children: [
          async.when(
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (_, _) => Center(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Text(
                  l10n.ratingPreviewError,
                  textAlign: TextAlign.center,
                  style: const TextStyle(color: CymbraColors.error),
                ),
              ),
            ),
            data: (score) => score.isEmpty
                ? const SizedBox.shrink()
                : CustomPaint(
                    painter: StaffPainter(
                      notes: score.notes,
                      rests: score.rests,
                      elapsedMs: _elapsedMs,
                      activeNotes: const <int>{}, // read-only: nothing pressed
                      bpm: score.bpm,
                      songEndMs: score.songEndMs,
                      keyFifths: score.keyFifths,
                      beats: score.beats,
                      beatType: score.beatType,
                      measureStartMs: score.measureStartMs,
                    ),
                    size: Size.infinite,
                  ),
          ),
          // Stop control, top-right — returns to the card face.
          Positioned(
            top: 4,
            right: 4,
            child: Material(
              color: Colors.black.withValues(alpha: 0.45),
              shape: const CircleBorder(),
              clipBehavior: Clip.antiAlias,
              child: IconButton(
                icon: const Icon(Icons.close, color: Colors.white),
                tooltip: l10n.ratingPreviewStop,
                onPressed: widget.onClose,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
