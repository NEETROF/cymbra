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

import 'dart:typed_data';

import 'package:audioplayers/audioplayers.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'sound_clip_player.g.dart';

/// Plays a short in-memory audio clip (a WAV byte buffer) — the SoundFont preview
/// audition path (change: add-soundfont-entitlement-previews). A seam over the audio
/// plugin so [SoundFontPreviewService](soundfont_preview_service.dart) can be driven
/// by a fake in unit/widget tests (which run on the Dart VM with no audio device).
abstract class SoundClipPlayer {
  /// Play [bytes] (a WAV clip), looping until [stop]; replaces any current clip.
  Future<void> play(Uint8List bytes);

  /// Stop any current clip.
  Future<void> stop();
}

/// Production [SoundClipPlayer] over `audioplayers`. Loops the short preview so the
/// audition continues until the user stops it (matching the previous synth audition).
class AudioPlayersClipPlayer implements SoundClipPlayer {
  final AudioPlayer _player = AudioPlayer();

  @override
  Future<void> play(Uint8List bytes) async {
    await _player.stop();
    await _player.setReleaseMode(ReleaseMode.loop);
    await _player.play(BytesSource(bytes, mimeType: 'audio/wav'));
  }

  @override
  Future<void> stop() => _player.stop();
}

/// Production clip-player provider. Override in tests with a recording fake.
@riverpod
SoundClipPlayer soundClipPlayer(Ref ref) {
  final player = AudioPlayersClipPlayer();
  ref.onDispose(() => player.stop());
  return player;
}
