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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/catalog_service.dart';
import '../services/notation_engine.dart';
import 'notation_playback.dart';
import 'player_data.dart';

part 'card_preview_notifier.g.dart';

/// The parsed, playback-ready form of a catalog score for the in-card read-only
/// preview (change: add-app-score-rating): the timed notes/rests and the timing
/// attributes the [StaffPainter] needs to render the horizontal game-score scroll.
/// Derived through the SAME notation pipeline the player uses (fetch bytes →
/// parse → [notationToTimedNotes]), never a second parser.
class CardPreviewScore {
  const CardPreviewScore({
    required this.notes,
    required this.rests,
    this.tieContinuations = const [],
    required this.songEndMs,
    required this.bpm,
    required this.keyFifths,
    this.measureKeyFifths = const [],
    required this.beats,
    required this.beatType,
    required this.measureStartMs,
    required this.startMs,
    this.isPercussion = false,
  });

  final List<TimedNote> notes;
  final List<TimedRest> rests;

  /// Render-only tie continuations (see [DerivedPlayback.tieContinuations]).
  final List<TimedNote> tieContinuations;
  final double songEndMs;
  final int bpm;
  final int keyFifths;
  final List<int> measureKeyFifths;
  final int beats;
  final int beatType;
  final List<int> measureStartMs;

  /// Playhead start (leading silence trimmed) — the preview loops back here.
  final double startMs;

  /// Whether the previewed score is percussion (change:
  /// add-drum-audio-channel): the card sounds it on the drum channel with a
  /// kit font, never as piano pitches.
  final bool isPercussion;

  bool get isEmpty => notes.isEmpty;
}

/// Loads and parses a catalog score into [CardPreviewScore] for the in-card
/// preview. A family keyed by `catalogId`; reuses the injectable
/// [catalogServiceProvider] (bytes) and [notationEngineProvider] (parse) seams,
/// so it is exercisable in tests without the native library or a live backend.
@riverpod
Future<CardPreviewScore> cardPreviewScore(Ref ref, String catalogId) async {
  // The deck previews `pending` candidates too, so it fetches through the
  // rating-preview path (not the accepted-only player-open `fetchBytes`) — change:
  // rate-pending-scores.
  final bytes = await ref
      .read(catalogServiceProvider)
      .ratingPreviewBytes(catalogId);
  final document = await ref.read(notationEngineProvider).parse(bytes);
  final derived = notationToTimedNotes(document);
  return CardPreviewScore(
    notes: derived.notes,
    rests: derived.rests,
    tieContinuations: derived.tieContinuations,
    songEndMs: derived.songEndMs,
    bpm: derived.bpm,
    keyFifths: document.attributes.keyFifths,
    measureKeyFifths: derived.measureKeyFifths,
    beats: document.attributes.time.beats,
    beatType: document.attributes.time.beatType,
    measureStartMs: derived.measureStartMs,
    startMs: effectiveStartMs(derived.notes),
    isPercussion: derived.isPercussion,
  );
}
