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

import 'package:flutter/services.dart' show rootBundle;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/notation_engine.dart';
import 'card_preview_notifier.dart' show CardPreviewScore;
import 'notation_playback.dart';
import 'player_data.dart';

part 'sound_preview_sample.g.dart';

/// A short **bundled** score used to audition an instrument sound in the sound hub
/// (change: add-soundfont-moderation) — parsed through the SAME notation pipeline
/// the player uses, so tapping a sound plays a real embedded piece with that
/// SoundFont loaded. Kept small and recognisable (Ode to Joy).
const String soundPreviewSampleAsset = 'assets/scores/beginner/ode_to_joy.musicxml';

/// Loads + parses [soundPreviewSampleAsset] into a playback-ready [CardPreviewScore].
/// `keepAlive` because the sample never changes AND the hub only `ref.read`s it (no
/// widget `watch`es it): an autoDispose future would be disposed + re-created on every
/// audition tick, so `valueOrNull` would stay null and nothing would ever play. Reuses
/// the injectable [notationEngineProvider] so it is testable without the native library.
@Riverpod(keepAlive: true)
Future<CardPreviewScore> soundPreviewSample(Ref ref) async {
  final data = await rootBundle.load(soundPreviewSampleAsset);
  final bytes = data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes);
  final document = await ref.read(notationEngineProvider).parse(bytes);
  final derived = notationToTimedNotes(document);
  return CardPreviewScore(
    notes: derived.notes,
    rests: derived.rests,
    songEndMs: derived.songEndMs,
    bpm: derived.bpm,
    keyFifths: document.attributes.keyFifths,
    measureKeyFifths: derived.measureKeyFifths,
    beats: document.attributes.time.beats,
    beatType: document.attributes.time.beatType,
    measureStartMs: derived.measureStartMs,
    startMs: effectiveStartMs(derived.notes),
  );
}
