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
/// sound — but with NO interaction, NO judging and NO scoring. It **auto-plays**
/// as soon as the card is shown (no Play button) and loops. Reuses the player's
/// [StaffPainter] and audio seam directly, driven by a local ticker, so nothing
/// about the scored player state is touched. Reports playback progress (0..1) via
/// [onProgress] so the deck can unlock rating only once enough has been heard.
class InCardPreview extends ConsumerStatefulWidget {
  const InCardPreview({super.key, required this.catalogId, this.onProgress});

  /// The catalog score to preview (its bytes are fetched + parsed on demand).
  final String catalogId;

  /// Reports the furthest-reached playback fraction (0..1) each frame, so the
  /// deck can gate rating on "listened to enough of the score".
  final ValueChanged<double>? onProgress;

  @override
  ConsumerState<InCardPreview> createState() => _InCardPreviewState();
}

class _InCardPreviewState extends ConsumerState<InCardPreview>
    with SingleTickerProviderStateMixin {
  /// How many measures to keep ahead of the playhead in the card. Small on
  /// purpose: the card reads ~1–2 measures at a time (a phrase), not the whole
  /// line, so the notation stays large and legible in the small preview.
  static const double _measuresAhead = 1.0;

  /// Shrinks the notation (notes, stems, glyphs) relative to the player's size,
  /// so the small card doesn't render oversized noteheads. Tune here (1.0 = the
  /// player's size).
  static const double _noteScale = 0.7;

  /// Rating unlocks at the SOONER of these two: a fraction of the piece, or a
  /// number of notes heard. 25% alone is too long on a long piece, so 25 notes
  /// caps the wait.
  static const double _unlockTimeFraction = 0.25;
  static const int _unlockNoteCount = 25;

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

  /// Notes whose onset the playhead has crossed (for the note-count unlock cap).
  int _notesPlayed = 0;

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
    final async = ref.read(cardPreviewScoreProvider(widget.catalogId));
    if (async.hasError) {
      // The preview couldn't load (bytes/parse failure): don't trap the user
      // behind the listen-to-rate gate — report it as fully "listened" so the
      // rating controls unlock (they can rate or skip). The error stays shown.
      widget.onProgress?.call(1.0);
      return;
    }
    final score = async.valueOrNull;
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
      _notesPlayed += edges.starts.length;
    }
    _reportUnlockProgress(score, next);
    setState(() => _elapsedMs = next);
  }

  /// The visible time window (ms) so the card shows ~[_measuresAhead] measures
  /// ahead of the playhead, derived from the piece's own tempo/metre so a slow
  /// and a fast piece show a comparable *number* of measures (not seconds).
  /// Clamped so an extreme tempo still reads sensibly.
  double _lookAheadFor(CardPreviewScore score) {
    final bpm = score.bpm <= 0 ? 90 : score.bpm;
    final beatType = score.beatType <= 0 ? 4 : score.beatType;
    final measureMs = (60000.0 / bpm) * score.beats * 4 / beatType;
    return (measureMs * _measuresAhead).clamp(1800.0, 5000.0);
  }

  /// Normalized progress toward unlocking rating (0..1, unlocked at 1). Reports the
  /// SOONER of the time and note-count thresholds, so a long piece unlocks after
  /// [_unlockNoteCount] notes rather than a long [_unlockTimeFraction] wait.
  void _reportUnlockProgress(CardPreviewScore score, double playhead) {
    final span = score.songEndMs - score.startMs;
    final byTime = span <= 0
        ? 0.0
        : ((playhead - score.startMs) / span) / _unlockTimeFraction;
    final byNotes = _notesPlayed / _unlockNoteCount;
    final progress = (byTime > byNotes ? byTime : byNotes).clamp(0.0, 1.0);
    widget.onProgress?.call(progress);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(cardPreviewScoreProvider(widget.catalogId));
    return ColoredBox(
      color: CymbraColors.surfaceContainerLow,
      child: async.when(
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
                  tieContinuations: score.tieContinuations,
                  elapsedMs: _elapsedMs,
                  activeNotes: const <int>{}, // read-only: nothing pressed
                  bpm: score.bpm,
                  songEndMs: score.songEndMs,
                  keyFifths: score.keyFifths,
                  measureKeyFifths: score.measureKeyFifths,
                  beats: score.beats,
                  beatType: score.beatType,
                  measureStartMs: score.measureStartMs,
                  lookAheadMs: _lookAheadFor(score),
                  noteScale: _noteScale,
                ),
                size: Size.infinite,
              ),
      ),
    );
  }
}
