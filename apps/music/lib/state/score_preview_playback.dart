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

import 'package:flutter/foundation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/score_preview_service.dart';
import '../services/sound_clip_player.dart';
import '../services/wav_duration.dart';
import 'usage_tracking_notifier.dart';

part 'score_preview_playback.freezed.dart';
part 'score_preview_playback.g.dart';

/// Immutable state of the catalog audio-teaser audition (change:
/// add-score-daily-access-rewards, design D8): which piece is sounding, which is
/// being fetched, the pieces known to have no teaser (greyed controls), and a
/// failure cue for a dedicated listener.
@freezed
abstract class ScorePreviewPlaybackState with _$ScorePreviewPlaybackState {
  const factory ScorePreviewPlaybackState({
    /// The piece whose clip is sounding right now, or null.
    String? playingId,

    /// The piece whose clip is being fetched, or null.
    String? loadingId,

    /// Pieces the server said have no teaser (404) — their control is greyed.
    @Default(<String>{}) Set<String> missing,

    /// Increments once per fetch/playback failure (a listener shows a snackbar).
    @Default(0) int errorSeq,
  }) = _ScorePreviewPlaybackState;
}

/// Owns the teaser audition for the cards AND the unlock sheet: fetch (session
/// cache by id) → play through the clip player → stop after exactly ONE pass (the
/// engine's clip player loops by design for the 2.4 s SoundFont phrase, wrong for
/// a 30 s teaser) → one clip at a time; `stop()` on opening a piece / app pause.
/// Fire-and-observe: the UI reads [ScorePreviewPlaybackState], never awaits.
@Riverpod(keepAlive: true)
class ScorePreviewPlayback extends _$ScorePreviewPlayback {
  final Map<String, Uint8List?> _cache = {};
  Timer? _endTimer;
  int _generation = 0;

  @override
  ScorePreviewPlaybackState build() {
    ref.onDispose(() => _endTimer?.cancel());
    return const ScorePreviewPlaybackState();
  }

  /// Whether [catalogId] is the clip sounding now.
  bool isPlaying(String catalogId) => state.playingId == catalogId;

  /// Start [catalogId]'s teaser (stopping any other), or stop it if it is the
  /// one sounding.
  Future<void> toggle(String catalogId) async {
    if (state.playingId == catalogId || state.loadingId == catalogId) {
      await stop();
      return;
    }
    await stop();
    final gen = ++_generation;
    state = state.copyWith(loadingId: catalogId);
    Uint8List? bytes;
    try {
      bytes = _cache.containsKey(catalogId)
          ? _cache[catalogId]
          : await ref.read(scorePreviewServiceProvider).fetchClip(catalogId);
      _cache[catalogId] = bytes;
    } catch (e) {
      debugPrint('score preview fetch failed for $catalogId: $e');
      if (gen != _generation) return;
      state = state.copyWith(loadingId: null, errorSeq: state.errorSeq + 1);
      return;
    }
    // Superseded by another toggle/stop while fetching: drop silently.
    if (gen != _generation) return;
    if (bytes == null) {
      state = state.copyWith(
        loadingId: null,
        missing: {...state.missing, catalogId},
      );
      return;
    }
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(UsageActions.catalogPreviewAudition, subjectId: catalogId),
    );
    await ref.read(soundClipPlayerProvider).play(bytes);
    if (gen != _generation) return;
    state = state.copyWith(loadingId: null, playingId: catalogId);
    // ONE pass: the engine loops, so stop it at the clip's end ourselves.
    final length = wavDuration(bytes) ?? const Duration(seconds: 30);
    _endTimer = Timer(length, () {
      if (_generation != gen) return;
      unawaited(stop());
    });
  }

  /// Stop any teaser (fetch in flight or clip sounding).
  Future<void> stop() async {
    _generation++;
    _endTimer?.cancel();
    _endTimer = null;
    final wasActive = state.playingId != null || state.loadingId != null;
    if (state.playingId != null) {
      await ref.read(soundClipPlayerProvider).stop();
    }
    if (wasActive) {
      state = state.copyWith(playingId: null, loadingId: null);
    }
  }
}
